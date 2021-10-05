//! Handwritten extensions for the generated AST.

use super::AstNode;

impl super::Stmt {
    pub fn item(&self) -> Option<super::Item> {
        self.syntax().children().find_map(super::Item::cast)
    }
}

