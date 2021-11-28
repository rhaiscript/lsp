use super::Module;
use rhai_rowan::TextRange;
use url::Url;

slotmap::new_key_type! { pub struct Source; }

#[derive(Debug, Clone)]
pub struct SourceData {
    pub module: Module,
    pub url: Url,
    pub kind: SourceKind,
}

#[derive(Debug, Copy, Clone)]
pub enum SourceKind {
    Script,
    Def,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SourceInfo {
    pub source: Option<Source>,
    pub text_range: Option<TextRange>,
    pub selection_text_range: Option<TextRange>,
}

impl SourceInfo {
    #[must_use]
    pub fn is_part_of(&self, source: Source) -> bool {
        self.source.map_or(false, |s| s == source)
    }
}
