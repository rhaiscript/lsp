//! Comments unfortunately require some special handling.
//!
//! Currently we take the following comments into account:
//!
//! ```rhai
//! {
//!   // 1. leading child comments
//!   
//!   // 1. with whitespace in between
//!
//!   let a = "foo"; // 2. comment between but still same line
//!
//!   // 2. standalone comments between nodes with white space
//!
//!   let b = "bar"; // 3. comments after last node
//!
//!   // 3. trailing standalone comments
//! }
//! ```

#![allow(dead_code)]
use rhai_rowan::syntax::{SyntaxElement, SyntaxKind::*, SyntaxNode};
use rowan::Direction;

use crate::{
    algorithm::Formatter,
    util::{break_count, ScopedStatic},
};
use std::io::{self, Write};

impl<W: Write> Formatter<W> {
    /// Comments in position 1.
    ///
    /// Returns true if ended with a hardbreak.
    pub(crate) fn leading_comments_in(&mut self, node: &SyntaxNode) -> io::Result<bool> {
        let ws_and_comments = node
            .children_with_tokens()
            .skip_while(|e| {
                e.as_token().is_some()
                    && !matches!(e.kind(), WHITESPACE | COMMENT_BLOCK | COMMENT_LINE)
            })
            .take_while(|e| matches!(e.kind(), WHITESPACE | COMMENT_BLOCK | COMMENT_LINE))
            .filter_map(SyntaxElement::into_token)
            .collect::<Vec<_>>();

        let mut hardbreak_last = false;
        for ws_or_comment in ws_and_comments {
            match ws_or_comment.kind() {
                COMMENT_BLOCK | COMMENT_LINE => {
                    self.word(ws_or_comment.static_text().trim_end())?;
                    hardbreak_last = false;
                }
                WHITESPACE => {
                    let breaks = break_count(&ws_or_comment);
                    self.hardbreaks(breaks);
                    if breaks > 0 {
                        hardbreak_last = true;
                    }
                }
                _ => unreachable!(),
            }
        }

        Ok(hardbreak_last)
    }

    /// Comments in position 2.
    pub(crate) fn comments_before_or_hardbreak(&mut self, node: &SyntaxNode) -> io::Result<()> {
        let mut ws_and_comments = node
            .siblings_with_tokens(Direction::Prev)
            .skip(1)
            .take_while(|e| matches!(e.kind(), WHITESPACE | COMMENT_BLOCK | COMMENT_LINE))
            .filter_map(SyntaxElement::into_token)
            .collect::<Vec<_>>();

        ws_and_comments.reverse();

        let mut hardbreak_last = false;

        let comment_at_p1 = ws_and_comments
            .get(1)
            .map(|e| e.kind() != WHITESPACE)
            .unwrap_or(false);

        for (idx, ws_or_comment) in ws_and_comments.into_iter().enumerate() {
            match ws_or_comment.kind() {
                COMMENT_BLOCK | COMMENT_LINE => {
                    if idx == 0 {
                        self.nbsp()?;
                    }
                    self.word(ws_or_comment.static_text().trim_end())?;
                    hardbreak_last = false;
                }
                WHITESPACE => {
                    let breaks = break_count(&ws_or_comment);
                    if idx == 0 && breaks == 0 && comment_at_p1 {
                        self.nbsp()?;
                    } else if breaks > 0 {
                        hardbreak_last = true;
                        self.hardbreaks(breaks)
                    }
                }
                _ => unreachable!(),
            }
        }

        if !hardbreak_last {
            self.hardbreak();
        }

        Ok(())
    }

    /// Comments in position 3.
    ///
    /// Always ends with a hardbreak unless
    /// specified **and** the last comment is a line comment.
    pub(crate) fn trailing_comments_after(
        &mut self,
        node: &SyntaxNode,
        hardbreak_after_last_line: bool,
    ) -> io::Result<()> {
        let mut ws_and_comments = node
            .siblings_with_tokens(Direction::Next)
            .skip(1)
            .take_while(|e| matches!(e.kind(), WHITESPACE | COMMENT_BLOCK | COMMENT_LINE))
            .filter_map(SyntaxElement::into_token)
            .enumerate()
            .collect::<Vec<_>>();

        let last_comment_position = ws_and_comments.iter().rev().find_map(|(idx, c)| {
            if c.kind() != WHITESPACE {
                Some(*idx)
            } else {
                None
            }
        });

        match last_comment_position {
            Some(p) => {
                ws_and_comments.truncate(p + 1);
            }
            None => return Ok(()),
        }

        let comment_at_p1 = ws_and_comments
            .get(1)
            .map(|(_, e)| e.kind() != WHITESPACE)
            .unwrap_or(false);

        let len = ws_and_comments.len();

        for (idx, ws_or_comment) in ws_and_comments {
            match ws_or_comment.kind() {
                COMMENT_BLOCK | COMMENT_LINE => {
                    if idx == 0 {
                        self.nbsp()?;
                    }
                    self.word(ws_or_comment.static_text().trim_end())?;

                    if hardbreak_after_last_line
                        && idx + 1 == len
                        && matches!(ws_or_comment.kind(), COMMENT_LINE)
                    {
                        self.hardbreak();
                    }
                }
                WHITESPACE => {
                    let breaks = break_count(&ws_or_comment);
                    if idx == 0 && breaks == 0 && comment_at_p1 {
                        self.nbsp()?;
                    } else if breaks != 0 {
                        self.hardbreaks(breaks);
                    }
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }
}
