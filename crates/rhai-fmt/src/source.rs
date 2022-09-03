use std::io::{self, Write};

use rhai_rowan::{
    ast::{AstNode, Item, Rhai},
    syntax::SyntaxKind::*,
    T,
};

use crate::{algorithm::Formatter, util::ScopedStatic};

impl<S: Write> Formatter<S> {
    pub(crate) fn fmt_rhai(&mut self, rhai: Rhai) -> io::Result<()> {
        self.cbox(0);

        if let Some(t) = rhai.shebang_token() {
            self.word(t.static_text())?;
            self.hardbreak();
        }

        self.leading_comments_in(&rhai.syntax())?;

        let count = rhai.statements().count();

        let mut first = true;
        let mut needs_sep = false;
        for (idx, stmt) in rhai.statements().enumerate() {
            let item = match stmt.item() {
                Some(item) => item,
                _ => continue,
            };
            let syntax = stmt.syntax();

            if !first {
                if needs_sep {
                    self.word(";")?;
                }

                self.comments_before_or_hardbreak(&syntax)?;
            }
            first = false;

            needs_sep = needs_stmt_separator(&item);
            self.fmt_item(item)?;

            let last = count == idx + 1;

            if last {
                let had_sep = stmt
                    .syntax()
                    .children_with_tokens()
                    .any(|c| c.kind() == T![";"]);

                if had_sep && needs_sep {
                    self.word(";")?;
                }
                self.trailing_comments_after(&syntax, false)?;
                self.hardbreak();
            }
        }

        self.end();

        Ok(())
    }
}

pub(crate) fn needs_stmt_separator(item: &Item) -> bool {
    match item
        .expr()
        .and_then(|e| e.syntax().first_child())
        .map(|e| e.kind())
    {
        Some(
            EXPR_IDENT | EXPR_BLOCK | EXPR_IF | EXPR_LOOP | EXPR_FOR | EXPR_WHILE | EXPR_SWITCH
            | EXPR_FN | EXPR_TRY,
        ) => false,
        _ => true,
    }
}
