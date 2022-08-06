use globset::{Glob, GlobSetBuilder};
use std::{
    mem,
    path::{Path, PathBuf}, borrow::Cow,
};
use url::Url;
use percent_encoding::percent_decode_str;

#[derive(Debug, Clone)]
pub struct GlobRule {
    include: globset::GlobSet,
    exclude: globset::GlobSet,
}

impl GlobRule {
    pub fn new(
        include: impl IntoIterator<Item = impl AsRef<str>>,
        exclude: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<Self, anyhow::Error> {
        let mut inc = GlobSetBuilder::new();
        for glob in include {
            inc.add(Glob::new(glob.as_ref())?);
        }

        let mut exc = GlobSetBuilder::new();
        for glob in exclude {
            exc.add(Glob::new(glob.as_ref())?);
        }

        Ok(Self {
            include: inc.build()?,
            exclude: exc.build()?,
        })
    }

    pub fn is_match(&self, text: impl AsRef<Path>) -> bool {
        if !self.include.is_match(text.as_ref()) {
            return false;
        }

        !self.exclude.is_match(text.as_ref())
    }
}

pub trait Normalize {
    /// Normalizing in the context of this library means the following:
    ///
    /// - replace `\` with `/` on windows
    /// - decode all percent-encoded characters
    #[must_use]
    fn normalize(self) -> Self;
}

impl Normalize for PathBuf {
    fn normalize(self) -> Self {
        match self.to_str() {
            Some(s) => (*normalize_str(s)).into(),
            None => self,
        }
    }
}

impl Normalize for Vec<PathBuf> {
    fn normalize(mut self) -> Self {
        for p in &mut self {
            *p = mem::take(p).normalize();
        }
        self
    }
}

impl Normalize for Url {
    fn normalize(self) -> Self {
        if self.scheme() != "file" {
            return self;
        }

        if let Ok(u) = normalize_str(self.as_str()).parse() {
            return u;
        }

        self
    }
}

pub(crate) fn normalize_str(s: &str) -> Cow<str> {
    let percent_decoded = match percent_decode_str(s).decode_utf8().ok() {
        Some(s) => s,
        None => return s.into(),
    };

    if cfg!(windows) {
        percent_decoded.replace('\\', "/").into()
    } else {
        percent_decoded
    }
}
