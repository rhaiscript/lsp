use std::io::{self, Write};

use rhai_rowan::ast::{AstNode, Item, Stmt};

use crate::{algorithm::Formatter, util::ScopedStatic};

impl<S: Write> Formatter<S> {
    pub(crate) fn fmt_stmt(&mut self, stmt: Stmt) -> io::Result<()> {
        if let Some(item) = stmt.item() {
            self.fmt_item(item)?;
        }
        Ok(())
    }

    pub(crate) fn fmt_item(&mut self, item: Item) -> io::Result<()> {
        self.cbox(0);

        for doc in item.docs() {
            self.fmt_doc(doc)?;
        }

        if let Some(expr) = item.expr() {
            self.fmt_expr(expr)?;
        }

        self.end();

        Ok(())
    }

    pub(crate) fn fmt_doc(&mut self, doc: rhai_rowan::ast::Doc) -> Result<(), io::Error> {
        let syntax = doc.syntax();
        if let Some(t) = doc.token() {
            self.word(t.static_text().trim_end())?;
            self.comment_same_line_after(&syntax)?;
            self.standalone_comments_after(&syntax)?;
            self.hardbreak();
        };
        Ok(())
    }
}
