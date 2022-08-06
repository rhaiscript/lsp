use crate::syntax::{
    SyntaxElement,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken,
};

pub(super) trait SyntaxExt {
    fn syntax_kind(&self) -> SyntaxKind;
    fn next_non_ws_sibling(&self) -> Option<SyntaxElement>;
    fn prev_non_ws_sibling(&self) -> Option<SyntaxElement>;
}

impl SyntaxExt for SyntaxElement {
    fn syntax_kind(&self) -> SyntaxKind {
        self.kind()
    }

    fn next_non_ws_sibling(&self) -> Option<SyntaxElement> {
        match self {
            rowan::NodeOrToken::Node(v) => v.next_non_ws_sibling(),
            rowan::NodeOrToken::Token(v) => v.next_non_ws_sibling(),
        }
    }

    fn prev_non_ws_sibling(&self) -> Option<SyntaxElement> {
        match self {
            rowan::NodeOrToken::Node(v) => v.prev_non_ws_sibling(),
            rowan::NodeOrToken::Token(v) => v.prev_non_ws_sibling(),
        }
    }
}

impl SyntaxExt for SyntaxToken {
    fn syntax_kind(&self) -> SyntaxKind {
        self.kind()
    }

    fn next_non_ws_sibling(&self) -> Option<SyntaxElement> {
        self.siblings_with_tokens(rowan::Direction::Next)
            .find(not_ws_or_comment)
    }

    fn prev_non_ws_sibling(&self) -> Option<SyntaxElement> {
        self.siblings_with_tokens(rowan::Direction::Prev)
            .find(not_ws_or_comment)
    }
}

impl SyntaxExt for SyntaxNode {
    fn syntax_kind(&self) -> SyntaxKind {
        self.kind()
    }

    fn next_non_ws_sibling(&self) -> Option<SyntaxElement> {
        self.siblings_with_tokens(rowan::Direction::Next)
            .find(not_ws_or_comment)
    }

    fn prev_non_ws_sibling(&self) -> Option<SyntaxElement> {
        self.siblings_with_tokens(rowan::Direction::Prev)
            .find(not_ws_or_comment)
    }
}

#[allow(dead_code)]
pub(super) fn not_ws_or_comment<T: SyntaxExt>(token: &T) -> bool {
    !matches!(
        token.syntax_kind(),
        WHITESPACE | COMMENT_BLOCK | COMMENT_LINE
    )
}
