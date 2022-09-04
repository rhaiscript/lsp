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
            // No hardbreak required here, as we should
            // already have whitespace in the file.
        }

        self.standalone_leading_comments_in(&rhai.syntax())?;

        let count = rhai.statements().count();

        for (idx, stmt) in rhai.statements().enumerate() {
            let item = match stmt.item() {
                Some(item) => item,
                _ => continue,
            };
            let stmt_syntax = stmt.syntax();
            let item_syntax = item.syntax();

            let last = count == idx + 1;
            let needs_sep = needs_stmt_separator(&item);

            self.ibox(0);

            self.fmt_item(item)?;

            let had_sep = stmt
                .syntax()
                .children_with_tokens()
                .any(|c| c.kind() == T![";"]);

            if last {
                if had_sep && needs_sep {
                    self.word(";")?;
                }
            } else if needs_sep {
                self.word(";")?;
            }

            self.comment_same_line_after(&item_syntax)?;
            self.comment_same_line_after(&stmt_syntax)?;

            self.end();

            let standalone_comments = if had_sep {
                self.standalone_comments_after(&stmt_syntax, !last)?
            } else {
                self.standalone_comments_after(&item_syntax, !last)?
            };

            if !standalone_comments.hardbreak_end {
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
            EXPR_BLOCK | EXPR_IF | EXPR_LOOP | EXPR_FOR | EXPR_WHILE | EXPR_SWITCH | EXPR_FN
            | EXPR_TRY,
        ) => false,
        _ => true,
    }
}
