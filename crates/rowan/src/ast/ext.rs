//! Handwritten extensions for the generated AST.
//!
//! These can be gradually turned into code generation if similar
//! repetitive patterns are found and the effort is worth it.

use crate::syntax::{SyntaxElement, SyntaxKind, SyntaxToken};

use super::{AstNode, Expr, ObjectField, Param};
use super::{ExprBlock, ExprIf, T};

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
                        let t = token.text().strip_prefix("///").unwrap_or_else(|| token.text());
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
