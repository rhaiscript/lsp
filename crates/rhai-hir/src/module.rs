use std::{ffi::OsStr, path::Path};

use crate::{source::Source, Hir, IndexSet, Scope};
use url::Url;

slotmap::new_key_type! { pub struct Module; }

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ModuleKind {
    /// The static module is the root of every Rhai script,
    /// items and modules defined in the scope of the static module
    /// are available in every script.
    ///
    /// Other root modules themselves are always part of this module.
    Static,

    /// A module that is defined inline.
    Inline,

    /// A module identified by an URL.
    Url(Url),
}

impl core::fmt::Display for ModuleKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleKind::Static => "static".fmt(f),
            ModuleKind::Inline => "inline".fmt(f),
            ModuleKind::Url(u) => u.fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModuleData {
    pub scope: Scope,
    pub kind: ModuleKind,
    pub docs: String,
    /// Protected modules must not be removed,
    /// even if it has no sources associated.
    pub protected: bool,
    pub sources: IndexSet<Source>,
}

impl ModuleData {
    #[must_use]
    pub fn url(&self) -> Option<&Url> {
        match &self.kind {
            ModuleKind::Static | ModuleKind::Inline => None,
            ModuleKind::Url(u) => Some(u),
        }
    }
}

pub const STATIC_URL_SCHEME: &str = "rhai-static";

/// Used to resolve module URLs for import statements and definitions.
pub trait ModuleResolver: Send + Sync {
    /// Construct an URL for a module that should be imported
    /// based on the module where the import statement was found
    /// and the processed (e.g. unescaped) import path.
    ///
    /// This function is also called for `module "foo.rhai"` definitions.
    ///
    /// # Examples
    ///
    /// Take the following statement in a rhai script:
    ///
    /// ```rhai
    /// // file:///path/to/foo.rhai
    /// import "./bar.rhai"
    /// ```
    ///
    /// This function will be called with the `file:///path/to/foo.rhai` module
    /// as the base and `./bar.rhai` as the import path.
    /// Resolvers typically would return the `file:///path/to/bar.rhai` URL
    /// in this case.
    ///
    /// # Errors
    ///
    /// This function should only report errors if the import
    /// path is invalid, it does not need to do further checks
    /// regarding module imports.
    fn resolve_url(&self, from: &Url, path: &str) -> anyhow::Result<Url>;

    /// Same as `resolve_url`, but this function is called instead if
    /// the import statement is in a module.
    /// 
    /// # Errors
    ///
    /// This function should only report errors if the import
    /// path is invalid, it does not need to do further checks
    /// regarding module imports.
    fn resolve_url_from_module(&self, hir: &Hir, from: Module, path: &str) -> anyhow::Result<Url> {
        self.resolve_url(
            hir[from]
                .url()
                .ok_or_else(|| anyhow::anyhow!("could not determine base url"))?,
            path,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DefaultModuleResolver;

impl ModuleResolver for DefaultModuleResolver {
    fn resolve_url(&self, from: &Url, path: &str) -> anyhow::Result<Url> {
        if let Ok(url) = Url::parse(path) {
            return Ok(url);
        }

        if Path::new(path).extension() == Some(OsStr::new("rhai")) {
            Ok(from.join(path)?)
        } else {
            let path = format!("{path}.rhai");
            Ok(from.join(&path)?)
        }
    }
}
