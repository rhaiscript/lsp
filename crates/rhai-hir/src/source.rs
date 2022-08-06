use rhai_rowan::TextRange;
use url::Url;

use crate::Module;

slotmap::new_key_type! { pub struct Source; }

#[derive(Debug, Clone)]
pub struct SourceData {
    pub url: Url,
    pub kind: SourceKind,
    pub module: Module,
}

#[derive(Debug, Copy, Clone)]
pub enum SourceKind {
    Script,
    Def,
}

impl SourceKind {
    /// Returns `true` if the source kind is [`Script`].
    ///
    /// [`Script`]: SourceKind::Script
    #[must_use]
    pub fn is_script(&self) -> bool {
        matches!(self, Self::Script)
    }

    /// Returns `true` if the source kind is [`Def`].
    ///
    /// [`Def`]: SourceKind::Def
    #[must_use]
    pub fn is_def(&self) -> bool {
        matches!(self, Self::Def)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SourceInfo {
    pub source: Option<Source>,
    pub text_range: Option<TextRange>,
    pub selection_text_range: Option<TextRange>,
}

impl SourceInfo {
    #[must_use]
    pub fn is(&self, source: Source) -> bool {
        self.source.map_or(false, |s| s == source)
    }
}
