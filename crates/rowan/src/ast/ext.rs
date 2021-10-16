//! Handwritten extensions for the generated AST.
//!
//! These can be gradually turned into code generation if similar
//! repetitive patterns are found and the effort is worth it.

use super::{AstNode, Expr, ObjectField, Param, ParamList, SwitchArm};
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
        let mut s = String::new();

        for doc in self.docs() {
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
}

impl super::ParamList {
    pub fn params(&self) -> impl Iterator<Item = Param> {
        self.syntax().descendants().filter_map(Param::cast)
    }
}

impl super::Path {
    pub fn segments(&self) -> impl Iterator<Item = SyntaxToken> {
        self.syntax()
            .descendants_with_tokens()
            .filter_map(SyntaxElement::into_token)
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