use rhai_rowan::{TextRange, syntax::{SyntaxNode, SyntaxToken}};
use serde::{Deserialize, Serialize};

/// `SyntaxElement`s cannot be used because they're `!Send` and cannot be deserialized,
/// so we store a limited amount of syntax information only.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SyntaxInfo {
    pub text_range: Option<TextRange>,
}

impl From<SyntaxNode> for SyntaxInfo {
    fn from(syntax: SyntaxNode) -> Self {
        Self {
            text_range: Some(syntax.text_range()),
        }
    }
}

impl<'a> From<&'a SyntaxNode> for SyntaxInfo {
    fn from(syntax: &SyntaxNode) -> Self {
        Self {
            text_range: Some(syntax.text_range()),
        }
    }
}

impl From<SyntaxToken> for SyntaxInfo {
    fn from(syntax: SyntaxToken) -> Self {
        Self {
            text_range: Some(syntax.text_range()),
        }
    }
}


impl<'a> From<&'a SyntaxToken> for SyntaxInfo {
    fn from(syntax: &SyntaxToken) -> Self {
        Self {
            text_range: Some(syntax.text_range()),
        }
    }
}
