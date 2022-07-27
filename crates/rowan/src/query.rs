//! Cursor queries of a document purely based on syntax.

use rowan::TextSize;

use crate::{
    ast::{AstNode, Path},
    syntax::{
        SyntaxElement,
        SyntaxKind::{self, *},
        SyntaxNode, SyntaxToken,
    },
};

#[derive(Debug, Default)]
pub struct Query {
    /// The offset the query was made for.
    pub offset: TextSize,
    /// Before the cursor.
    pub before: Option<PositionInfo>,
    /// After the cursor.
    pub after: Option<PositionInfo>,
}

impl Query {
    /// Query information about a cursor position in a syntax tree.
    #[must_use]
    pub fn at(root: &SyntaxNode, offset: TextSize) -> Self {
        Query {
            offset,
            before: offset
                .checked_sub(TextSize::from(1))
                .and_then(|offset| Self::position_info_at(root, offset)),
            after: if offset >= root.text_range().end() {
                None
            } else {
                Self::position_info_at(root, offset)
            },
        }
    }

    fn position_info_at(syntax: &SyntaxNode, offset: TextSize) -> Option<PositionInfo> {
        let syntax = match syntax.token_at_offset(offset) {
            rowan::TokenAtOffset::None => return None,
            rowan::TokenAtOffset::Single(s) => s,
            rowan::TokenAtOffset::Between(_, right) => right,
        };

        Some(PositionInfo { syntax })
    }
}

impl Query {
    #[must_use]
    pub fn is_field_access(&self) -> bool {
        let before = match &self.before {
            Some(before) => before,
            None => return false,
        };

        let binary_expr = before
            .syntax
            .parent_ancestors()
            .find(|t| !matches!(t.kind(), EXPR_IDENT | EXPR_PAREN | EXPR));

        let binary_expr = match binary_expr {
            Some(p) => p,
            None => return false,
        };

        if binary_expr.kind() != EXPR_BINARY {
            return false;
        }

        binary_expr
            .children_with_tokens()
            .any(|t| t.kind() == PUNCT_DOT)
    }

    #[must_use]
    pub fn is_path(&self) -> bool {
        self.path_node().is_some()
    }

    pub fn path(&self) -> Option<Path> {
        self.path_node().and_then(AstNode::cast)
    }

    /// # Panics
    ///
    /// Panics if the query is not a path.
    #[must_use]
    pub fn path_segment_index(&self) -> usize {
        let p = self.path().unwrap();

        p.segments()
            .enumerate()
            .find_map(|(i, p)| {
                if p.text_range().contains_inclusive(self.offset) {
                    Some(i)
                } else {
                    None
                }
            })
            // If a segment was not found,
            // we assume that we are at the end of the path.
            .unwrap_or_else(|| p.segments().count())
    }

    #[must_use]
    pub fn is_in_comment(&self) -> bool {
        match (&self.before, &self.after) {
            (None, Some(_) | None) => false,
            (Some(before), None) => matches!(before.syntax.kind(), COMMENT_LINE | COMMENT_LINE_DOC),
            (Some(before), Some(after)) => matches!(
                (before.syntax.kind(), after.syntax.kind()),
                (COMMENT_LINE | COMMENT_LINE_DOC, _)
                    | (
                        COMMENT_BLOCK | COMMENT_BLOCK_DOC,
                        COMMENT_BLOCK | COMMENT_BLOCK_DOC
                    )
            ),
        }
    }

    fn path_node(&self) -> Option<SyntaxNode> {
        let path_before = self.before.as_ref().and_then(|t| match t.syntax.parent() {
            Some(p) => {
                if p.kind() == PATH {
                    Some(p)
                } else {
                    None
                }
            }
            None => None,
        });

        if path_before.is_some() {
            return path_before;
        }

        let path_after = self.after.as_ref().and_then(|t| match t.syntax.parent() {
            Some(p) => {
                if p.kind() == PATH {
                    Some(p)
                } else {
                    None
                }
            }
            None => None,
        });

        path_after
    }
}

#[derive(Debug, Clone)]
pub struct PositionInfo {
    /// The narrowest syntax element that contains the position.
    pub syntax: SyntaxToken,
}

#[cfg(test)]
mod tests {
    use crate::{parser::Parser, util::src_cursor_offset};

    use super::*;

    #[test]
    fn test_query_field_access_empty() {
        let (offset, src) = src_cursor_offset(
            r#"
            foo.   $$
            "#,
        );

        let syntax = Parser::new(&src).parse_script().into_syntax();

        let q = Query::at(&syntax, offset);

        assert!(q.is_field_access());
    }

    #[test]
    fn test_query_field_access_ident() {
        let (offset, src) = src_cursor_offset(
            r#"
            foo.   id$$ent
            "#,
        );

        let syntax = Parser::new(&src).parse_script().into_syntax();

        let q = Query::at(&syntax, offset);

        assert!(q.is_field_access());
    }

    #[test]
    fn test_query_field_access_paren() {
        let (offset, src) = src_cursor_offset(
            r#"
            foo.   (id$$ent)
            "#,
        );

        let syntax = Parser::new(&src).parse_script().into_syntax();

        let q = Query::at(&syntax, offset);

        assert!(q.is_field_access());
    }

    #[test]
    fn test_query_not_field_access() {
        let (offset, src) = src_cursor_offset(
            r#"
            foo.   (a + id$$ent)
            "#,
        );

        let syntax = Parser::new(&src).parse_script().into_syntax();

        let q = Query::at(&syntax, offset);

        assert!(!q.is_field_access());
    }

    #[test]
    fn test_query_path_middle() {
        let (offset, src) = src_cursor_offset(
            r#"
            p$$ath::foo::bar
            "#,
        );

        let syntax = Parser::new(&src).parse_script().into_syntax();

        let q = Query::at(&syntax, offset);

        assert!(q.is_path());
    }

    #[test]
    fn test_query_path_before() {
        let (offset, src) = src_cursor_offset(
            r#"
            $$path::foo::bar
            "#,
        );

        let syntax = Parser::new(&src).parse_script().into_syntax();

        let q = Query::at(&syntax, offset);

        assert!(q.is_path());
    }

    #[test]
    fn test_query_path_after() {
        let (offset, src) = src_cursor_offset(
            r#"
            path::foo::bar$$
            "#,
        );

        let syntax = Parser::new(&src).parse_script().into_syntax();

        let q = Query::at(&syntax, offset);

        assert!(q.is_path());
    }

    #[test]
    fn test_query_path_segment_index() {
        let (offset, src) = src_cursor_offset(
            r#"
            path::foo::bar$$
            "#,
        );

        let syntax = Parser::new(&src).parse_script().into_syntax();

        let q = Query::at(&syntax, offset);

        assert_eq!(q.path_segment_index(), 2);

        let (offset, src) = src_cursor_offset(
            r#"
            path::f$$oo::bar
            "#,
        );

        let syntax = Parser::new(&src).parse_script().into_syntax();

        let q = Query::at(&syntax, offset);

        assert_eq!(q.path_segment_index(), 1);

        let (offset, src) = src_cursor_offset(
            r#"
            $$path::foo::bar
            "#,
        );

        let syntax = Parser::new(&src).parse_script().into_syntax();

        let q = Query::at(&syntax, offset);

        assert_eq!(q.path_segment_index(), 0);

        let (offset, src) = src_cursor_offset(
            r#"
            path$$::foo::bar
            "#,
        );

        let syntax = Parser::new(&src).parse_script().into_syntax();

        let q = Query::at(&syntax, offset);

        assert_eq!(q.path_segment_index(), 0);

        let (offset, src) = src_cursor_offset(
            r#"
            path::foo::$$
            "#,
        );

        let syntax = Parser::new(&src).parse_script().into_syntax();

        let q = Query::at(&syntax, offset);

        assert_eq!(q.path_segment_index(), 2);
    }
}

trait GetSyntaxKind {
    fn syntax_kind(&self) -> SyntaxKind;
}

impl GetSyntaxKind for SyntaxElement {
    fn syntax_kind(&self) -> SyntaxKind {
        self.kind()
    }
}

impl GetSyntaxKind for SyntaxToken {
    fn syntax_kind(&self) -> SyntaxKind {
        self.kind()
    }
}

impl GetSyntaxKind for SyntaxNode {
    fn syntax_kind(&self) -> SyntaxKind {
        self.kind()
    }
}

#[allow(dead_code)]
fn not_ws_or_comment<T: GetSyntaxKind>(token: &T) -> bool {
    !matches!(
        token.syntax_kind(),
        WHITESPACE | COMMENT_BLOCK | COMMENT_LINE
    )
}
