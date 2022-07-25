//! Handwritten extensions for the generated AST.
//!
//! These can be gradually turned into code generation if similar
//! repetitive patterns are found and the effort is worth it.

use super::{
    AstNode, Expr, ObjectField, Param, ParamList, SwitchArm, SwitchArmCondition, TypedParam,
};
use super::{ExprBlock, ExprIf, T};
use crate::syntax::{SyntaxElement, SyntaxKind, SyntaxToken};

impl super::ExprLet {
    pub fn expr(&self) -> Option<Expr> {
        self.syntax().children().find_map(Expr::cast)
    }
}

impl super::ExprReturn {
    pub fn expr(&self) -> Option<Expr> {
        self.syntax().children().find_map(Expr::cast)
    }
}

impl super::ExprBreak {
    pub fn expr(&self) -> Option<Expr> {
        self.syntax().children().find_map(Expr::cast)
    }
}

impl super::Stmt {
    pub fn item(&self) -> Option<super::Item> {
        self.syntax().children().find_map(super::Item::cast)
    }
}

impl super::Item {
    #[must_use]
    pub fn docs_content(&self) -> String {
        docs_to_string(self.docs())
    }
}

impl super::ParamList {
    pub fn params(&self) -> impl Iterator<Item = Param> {
        self.syntax().descendants().filter_map(Param::cast)
    }
}

impl super::TypedParamList {
    pub fn params(&self) -> impl Iterator<Item = TypedParam> {
        self.syntax().descendants().filter_map(TypedParam::cast)
    }
}

impl super::Path {
    pub fn segments(&self) -> impl Iterator<Item = SyntaxToken> {
        self.syntax()
            .descendants_with_tokens()
            .filter(|t| {
                t.kind() != SyntaxKind::PUNCT_COLON2
                    && t.kind() != SyntaxKind::WHITESPACE
                    && t.kind() != SyntaxKind::COMMENT_BLOCK
                    && t.kind() != SyntaxKind::COMMENT_LINE
            })
            .filter_map(SyntaxElement::into_token)
    }
}

impl super::ExprFn {
    #[must_use]
    pub fn kw_private_token(&self) -> Option<SyntaxToken> {
        self.syntax().children_with_tokens().find_map(|t| {
            if t.kind() != SyntaxKind::KW_PRIVATE {
                return None;
            }
            t.into_token()
        })
    }
}

impl super::ExprBinary {
    pub fn op_token(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .find_map(SyntaxElement::into_token)
    }
}

impl super::ExprUnary {
    pub fn op_token(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .find_map(SyntaxElement::into_token)
    }
}

impl super::ExprArray {
    pub fn values(&self) -> impl Iterator<Item = Expr> {
        self.syntax().children().filter_map(Expr::cast)
    }
}

impl super::ExprObject {
    pub fn fields(&self) -> impl Iterator<Item = ObjectField> {
        self.syntax().children().filter_map(ObjectField::cast)
    }
}

impl super::ObjectField {
    #[must_use]
    pub fn property(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .find_map(|elem| match elem {
                rowan::NodeOrToken::Token(t)
                    if elem.kind() == T!["lit_str"] || elem.kind() == T!["ident"] =>
                {
                    Some(t)
                }
                _ => None,
            })
    }
}

impl super::ArgList {
    pub fn arguments(&self) -> impl Iterator<Item = Expr> {
        self.syntax().children().filter_map(Expr::cast)
    }
}

impl super::ExprIf {
    pub fn else_if_branch(&self) -> Option<ExprIf> {
        self.then_branch()
            .and_then(|t| t.syntax().next_sibling())
            .and_then(ExprIf::cast)
    }
    pub fn else_branch(&self) -> Option<ExprBlock> {
        self.then_branch()
            .and_then(|t| t.syntax().next_sibling())
            .and_then(ExprBlock::cast)
    }
}

impl super::Pat {
    pub fn idents(&self) -> impl Iterator<Item = SyntaxToken> {
        self.syntax()
            .descendants_with_tokens()
            .filter(|t| t.kind() == T!["ident"])
            .filter_map(SyntaxElement::into_token)
    }
}

impl super::SwitchArmList {
    pub fn arms(&self) -> impl Iterator<Item = SwitchArm> {
        self.syntax().children().filter_map(SwitchArm::cast)
    }
}

impl super::SwitchArm {
    #[must_use]
    pub fn pattern_expr(&self) -> Option<Expr> {
        let fat_arrow = self.punct_arrow_fat_token();

        fat_arrow
            .and_then(|arrow| arrow.prev_sibling_or_token())
            .and_then(SyntaxElement::into_node)
            .and_then(Expr::cast)
            .or_else(|| self.syntax().children().next().and_then(Expr::cast))
    }

    #[must_use]
    pub fn discard_token(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .next()
            .and_then(SyntaxElement::into_token)
            .and_then(|s| if s.kind() == T!["_"] { Some(s) } else { None })
    }

    pub fn condition(&self) -> Option<SwitchArmCondition> {
        self.syntax().children().find_map(SwitchArmCondition::cast)
    }

    #[must_use]
    pub fn value_expr(&self) -> Option<Expr> {
        let fat_arrow = self.punct_arrow_fat_token();

        fat_arrow
            .and_then(|arrow| arrow.next_sibling_or_token())
            .and_then(SyntaxElement::into_node)
            .and_then(Expr::cast)
    }
}

impl super::ExprImport {
    #[must_use]
    pub fn alias(&self) -> Option<SyntaxToken> {
        self.syntax()
            .last_child_or_token()
            .and_then(SyntaxElement::into_token)
            .and_then(|t| {
                if t.kind() == T!["ident"] {
                    Some(t)
                } else {
                    None
                }
            })
    }
}

impl super::ExprTry {
    pub fn catch_params(&self) -> Option<ParamList> {
        self.syntax().children().find_map(ParamList::cast)
    }
}

impl super::DefStmt {
    pub fn item(&self) -> Option<super::DefItem> {
        self.syntax().children().find_map(super::DefItem::cast)
    }
}

impl super::DefImport {
    #[must_use]
    pub fn alias(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(SyntaxElement::into_token)
            .find(|el| el.kind() == T!["ident"])
    }
}

impl super::DefItem {
    #[must_use]
    pub fn docs_content(&self) -> String {
        docs_to_string(self.docs())
    }
}

impl super::DefFn {
    #[must_use]
    #[inline]
    pub fn has_kw_get(&self) -> bool {
        let mut tokens = self.syntax().children_with_tokens().filter_map(|t| {
            if t.kind() != T!["ident"] {
                return None;
            }
            t.into_token()
        });

        let get = tokens.next();

        if let Some("get") = get.as_ref().map(SyntaxToken::text) {
            return tokens.next().is_some();
        }

        false
    }

    #[must_use]
    #[inline]
    pub fn has_kw_set(&self) -> bool {
        let mut tokens = self.syntax().children_with_tokens().filter_map(|t| {
            if t.kind() != T!["ident"] {
                return None;
            }
            t.into_token()
        });

        let get = tokens.next();

        if let Some("set") = get.as_ref().map(SyntaxToken::text) {
            return tokens.next().is_some();
        }

        false
    }

    #[must_use]
    pub fn get_token(&self) -> Option<SyntaxToken> {
        if !self.has_kw_get() {
            return None;
        }

        self.syntax().children_with_tokens().find_map(|t| {
            if t.kind() != T!["ident"] {
                return None;
            }
            t.into_token()
        })
    }

    #[must_use]
    pub fn ident_token(&self) -> Option<SyntaxToken> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|t| {
                if t.kind() != T!["ident"] {
                    return None;
                }
                t.into_token()
            })
            .nth(if self.has_kw_get() { 1 } else { 0 })
    }
}

impl super::DefModuleDecl {
    #[must_use]
    pub fn docs_content(&self) -> String {
        docs_to_string(self.docs())
    }
}

impl super::DefModule {
    #[must_use]
    pub fn kw_static_token(&self) -> Option<SyntaxToken> {
        self.syntax().children_with_tokens().find_map(|t| {
            if t.kind() != T!["static"] {
                return None;
            }
            t.into_token()
        })
    }

    #[must_use]
    pub fn ident_token(&self) -> Option<SyntaxToken> {
        self.syntax().children_with_tokens().find_map(|t| {
            if t.kind() != T!["ident"] {
                return None;
            }
            t.into_token()
        })
    }

    #[must_use]
    pub fn lit_str_token(&self) -> Option<SyntaxToken> {
        self.syntax().children_with_tokens().find_map(|t| {
            if t.kind() != T!["lit_str"] {
                return None;
            }
            t.into_token()
        })
    }
}

fn docs_to_string(docs: impl Iterator<Item = super::Doc>) -> String {
    let mut s = String::new();

    for doc in docs {
        if let Some(token) = doc.token() {
            match token.kind() {
                SyntaxKind::COMMENT_BLOCK_DOC => {
                    s += token
                        .text()
                        .strip_prefix("/**")
                        .unwrap_or_else(|| token.text())
                        .strip_suffix("*/")
                        .unwrap_or_else(|| token.text())
                        .trim();
                }
                SyntaxKind::COMMENT_LINE_DOC => {
                    let t = token
                        .text()
                        .strip_prefix("///")
                        .unwrap_or_else(|| token.text());
                    let t = t.strip_prefix(' ').unwrap_or(t);
                    let t = t.trim_end();
                    s += t;
                    s += "\n";
                }
                _ => unreachable!(),
            }
        }
    }

    s.truncate(s.trim_end().len());
    s
}
