use crate::{source::Source, IndexSet, Scope};
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
