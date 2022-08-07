//! Cursor queries of a document purely based on syntax.

use rowan::{NodeOrToken, TextSize};

use crate::{
    ast::{AstNode, Path},
    syntax::{SyntaxKind::*, SyntaxNode, SyntaxToken},
    T,
};

use self::util::SyntaxExt;

mod util;

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
        let binary_expr = match self.binary_expr() {
            Some(expr) => expr,
            None => return false,
        };

        binary_expr
            .children_with_tokens()
            .any(|t| t.kind() == PUNCT_DOT)
    }

    #[must_use]
    pub fn ident(&self) -> Option<SyntaxToken> {
        self.before
            .as_ref()
            .and_then(|t| {
                if t.syntax.kind() == IDENT {
                    Some(t.syntax.clone())
                } else {
                    None
                }
            })
            .or_else(|| {
                self.after.as_ref().and_then(|t| {
                    if t.syntax.kind() == IDENT {
                        Some(t.syntax.clone())
                    } else {
                        None
                    }
                })
            })
    }

    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn binary_op_ident(&self) -> Option<SyntaxToken> {
        self.binary_expr().and_then(|expr| {
            expr.children_with_tokens().find_map(|t| {
                if t.kind() == IDENT {
                    let ident = t.into_token().unwrap();
                    if let Some(before) = &self.before {
                        if before.syntax == ident {
                            return Some(ident);
                        }
                    }

                    if let Some(after) = &self.after {
                        if after.syntax == ident {
                            return Some(ident);
                        }
                    }

                    None
                } else {
                    None
                }
            })
        })
    }

    #[must_use]
    pub fn binary_expr(&self) -> Option<SyntaxNode> {
        let before = match &self.before {
            Some(before) => before,
            None => return None,
        };

        let binary_expr = before
            .syntax
            .parent_ancestors()
            .find(|t| !matches!(t.kind(), EXPR_IDENT | EXPR_PAREN | EXPR));

        let binary_expr = match binary_expr {
            Some(p) => p,
            None => return None,
        };

        if binary_expr.kind() != EXPR_BINARY {
            return None;
        }

        Some(binary_expr)
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

    #[must_use]
    pub fn can_complete_ref(&self) -> bool {
        if self.can_complete_op() {
            return false;
        }

        #[allow(clippy::match_same_arms)]
        match (
            self.before.as_ref().and_then(|p| {
                let expr = p.expr()?;
                Some((expr.kind(), p, expr))
            }),
            self.after.as_ref().and_then(|p| {
                let expr = p.expr()?;
                Some((expr.kind(), p, expr))
            }),
        ) {
            (Some((EXPR_CONST | EXPR_LET, ..)), Some((EXPR_CONST | EXPR_LET, ..))) => false,
            (Some((EXPR_CONST | EXPR_LET, ..)), None) => self.after.is_some(),
            (Some((EXPR_BLOCK, ..)), Some((EXPR_BLOCK, ..))) => true,
            (Some((EXPR_IDENT, pos, ..)), _) | (_, Some((EXPR_IDENT, pos, ..))) => {
                pos.syntax.kind() != WHITESPACE
                    && pos
                        .syntax
                        .prev_sibling_or_token()
                        .map_or(true, |t| t.kind() == IDENT)
            }
            (Some((EXPR_BINARY, _, _)), Some((EXPR_BINARY, _, _))) => false,
            (Some((EXPR_LIT, pos, ..)), _) | (_, Some((EXPR_LIT, pos, ..))) => {
                pos.syntax.kind() != WHITESPACE
            }
            _ => !self.is_in_fn_signature(),
        }
    }

    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn can_complete_op(&self) -> bool {
        // Deal with empty expressions first.
        if let Some(before) = &self.before {
            if before.syntax.parent().map(|e| e.kind()) == Some(EXPR_BLOCK) {
                return false;
            }

            if let Some(exp_w) = before.expr_wrapper() {
                if let Some(binary_exp) = exp_w.parent() {
                    if binary_exp.kind() == EXPR_BINARY {
                        if let Some(op_token) = binary_exp
                            .children_with_tokens()
                            .find_map(NodeOrToken::into_token)
                        {
                            if !(op_token.kind() == T!["ident"]
                                || op_token.kind().is_reserved_keyword())
                            {
                                return false;
                            }
                        }
                    }
                }
            }
        }

        if let Some(after) = &self.after {
            if after.syntax.parent().map(|e| e.kind()) == Some(EXPR_BLOCK) {
                return false;
            }

            if after.syntax.kind() == PUNCT_BRACE_END
                && after.syntax.next_sibling_or_token().map(|t| t.kind()) == Some(LIT_STR)
                && after.syntax.prev_sibling_or_token().map(|t| t.kind())
                    == Some(LIT_STR_TEMPLATE_INTERPOLATION)
            {
                return true;
            }

            if let Some(exp_w) = after.expr_wrapper() {
                if let Some(binary_exp) = exp_w.parent() {
                    if binary_exp.kind() == EXPR_BINARY {
                        if let Some(op_token) = binary_exp
                            .children_with_tokens()
                            .find_map(NodeOrToken::into_token)
                        {
                            if !(op_token.kind() == T!["ident"]
                                || op_token.kind().is_reserved_keyword())
                            {
                                return false;
                            }
                        }
                    }
                }
            }
        }

        #[allow(clippy::match_same_arms)]
        match (
            self.before.as_ref().and_then(|p| {
                let expr = p.expr()?;
                Some((expr.kind(), p, expr))
            }),
            self.after.as_ref().and_then(|p| {
                let expr = p.expr()?;
                Some((expr.kind(), p, expr))
            }),
        ) {
            (Some((EXPR_BINARY, pos, _)), _) | (_, Some((EXPR_BINARY, pos, _))) => {
                pos.syntax.kind() != WHITESPACE
            }
            (Some((EXPR_UNARY | EXPR_BLOCK | EXPR_FN, _, _)), _)
            | (_, Some((EXPR_UNARY | EXPR_BLOCK | EXPR_FN, _, _))) => false,
            (Some((.., pos, _)), ..) | (.., Some((.., pos, _))) => {
                if let Some(parent) = self.binary_expr() {
                    if let Some(op_token) = parent
                        .children_with_tokens()
                        .find_map(NodeOrToken::into_token)
                    {
                        if !(op_token.kind() == T!["ident"]
                            || op_token.kind().is_reserved_keyword())
                        {
                            return false;
                        }
                    }

                    if let Some(lhs) = parent.first_child() {
                        if lhs.kind() == EXPR
                            && lhs.text_range().contains_range(pos.syntax.text_range())
                        {
                            return false;
                        }
                    }
                }

                pos.syntax.kind() == WHITESPACE
            }
            _ => false,
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

    fn is_in_fn_signature(&self) -> bool {
        let pos_info = match self.before.as_ref().or(self.after.as_ref()) {
            Some(before) => before,
            None => return false,
        };

        if let Some(EXPR_BLOCK) = pos_info.syntax.parent().map(|p| p.kind()) {
            if let Some(T!["{"]) = pos_info.syntax.next_non_ws_sibling().map(|t| t.kind()) {
                return true;
            }

            return false;
        }

        pos_info.expr().map_or(false, |c| c.kind() == EXPR_FN)
    }
}

#[derive(Debug, Clone)]
pub struct PositionInfo {
    /// The narrowest syntax element that contains the position.
    pub syntax: SyntaxToken,
}

impl PositionInfo {
    /// The first direct child of `EXPR`, always
    /// an `EXPR_*` node.
    #[must_use]
    pub fn expr(&self) -> Option<SyntaxNode> {
        self.syntax
            .parent_ancestors()
            .find(|t| t.kind() == EXPR)
            .and_then(|t| t.first_child())
    }

    /// The closest `EXPR` ancestor that might or might not
    /// have an `EXPR_*` child.
    #[must_use]
    pub fn expr_wrapper(&self) -> Option<SyntaxNode> {
        self.syntax.parent_ancestors().find(|t| t.kind() == EXPR)
    }
}

#[cfg(test)]
mod tests;
