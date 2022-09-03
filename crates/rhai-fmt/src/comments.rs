//! Comments unfortunately require some special handling.
//!
//! Currently we take the following comments into account:
//!
//! ```rhai
//! {
//!   // 1. leading standalone comments
//!   
//!   // 1. with whitespace in between
//!
//!   let a = "foo"; // 2. comment on the same line.
//!
//!   // 3. trailing standalone comments
//!
//!   // 3. also with whitespace
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
    /// Add standalone comments and whitespace
    /// before the first child node.
    pub(crate) fn standalone_leading_comments_in(
        &mut self,
        node: &SyntaxNode,
    ) -> io::Result<CommentInfo> {
        let mut info = CommentInfo::default();

        let ws_and_comments = node
            .children_with_tokens()
            .skip_while(|e| {
                e.as_token().is_some()
                    && !matches!(e.kind(), WHITESPACE | COMMENT_BLOCK | COMMENT_LINE)
            })
            .take_while(|e| matches!(e.kind(), WHITESPACE | COMMENT_BLOCK | COMMENT_LINE))
            .filter_map(SyntaxElement::into_token)
            .collect::<Vec<_>>();

        let comment_count = ws_and_comments
            .iter()
            .filter(|c| c.kind() != WHITESPACE)
            .count();

        if comment_count == 0 {
            return Ok(info);
        }

        for ws_or_comment in ws_and_comments {
            match ws_or_comment.kind() {
                COMMENT_BLOCK | COMMENT_LINE => {
                    self.word(ws_or_comment.static_text().trim_end())?;
                    info.comment_added = true;
                    info.hardbreak_end = false;
                }
                WHITESPACE => {
                    let breaks = break_count(&ws_or_comment);
                    if breaks > 0 {
                        info.hardbreak_added = true;
                        info.hardbreak_end = true;
                    }
                    self.hardbreaks(breaks);
                }
                _ => unreachable!(),
            }
        }

        Ok(info)
    }

    pub(crate) fn comment_same_line_after(&mut self, node: &SyntaxNode) -> io::Result<CommentInfo> {
        let mut info = CommentInfo::default();

        let mut ws_and_comments = node
            .siblings_with_tokens(Direction::Next)
            .skip(1)
            .skip_while(|ws| {
                // Skip punctuation
                ws.as_token().is_some()
                    && !matches!(ws.kind(), WHITESPACE | COMMENT_BLOCK | COMMENT_LINE)
            })
            .take_while(|e| matches!(e.kind(), WHITESPACE | COMMENT_BLOCK | COMMENT_LINE))
            .filter_map(SyntaxElement::into_token);

        match ws_and_comments.next() {
            Some(t) => {
                if t.kind() == WHITESPACE && break_count(&t) > 0 {
                    return Ok(info);
                } else if t.kind() == WHITESPACE {
                    if let Some(c) = ws_and_comments.next() {
                        if c.kind() != WHITESPACE {
                            self.space();
                            self.word(c.static_text().trim())?;
                            info.comment_added = true;
                        }
                    }
                } else {
                    self.space();
                    self.word(t.static_text().trim())?;
                    info.comment_added = true;
                }
            }
            None => {}
        }

        Ok(info)
    }

    pub(crate) fn standalone_comments_after(
        &mut self,
        node: &SyntaxNode,
    ) -> io::Result<CommentInfo> {
        let mut info = CommentInfo::default();

        let mut ws_and_comments = node
            .siblings_with_tokens(Direction::Next)
            .skip(1)
            .skip_while(|ws| {
                // Skip punctuation
                if ws.as_token().is_some()
                    && !matches!(ws.kind(), WHITESPACE | COMMENT_BLOCK | COMMENT_LINE)
                {
                    return true;
                }

                if let Some(token) = ws.as_token() {
                    if break_count(token) != 0 {
                        return false;
                    }
                }

                true
            })
            .take_while(|e| matches!(e.kind(), WHITESPACE | COMMENT_BLOCK | COMMENT_LINE))
            .filter_map(SyntaxElement::into_token)
            .collect::<Vec<_>>();

        let last_comment_position =
            ws_and_comments
                .iter()
                .enumerate()
                .rev()
                .find_map(|(idx, t)| {
                    if t.kind() != WHITESPACE {
                        Some(idx)
                    } else {
                        None
                    }
                });

        match last_comment_position {
            Some(p) => {
                ws_and_comments.truncate(p + 1);
            }
            None => return Ok(info),
        }

        for ws_or_comment in ws_and_comments {
            match ws_or_comment.kind() {
                COMMENT_BLOCK | COMMENT_LINE => {
                    self.word(ws_or_comment.static_text().trim_end())?;
                    info.hardbreak_end = false;
                }
                WHITESPACE => {
                    let breaks = break_count(&ws_or_comment);
                    if breaks > 0 {
                        info.hardbreak_added = true;
                        info.hardbreak_end = true;
                    }
                    self.hardbreaks(breaks);
                }
                _ => unreachable!(),
            }
        }

        Ok(info)
    }
}

#[derive(Default)]
pub(crate) struct CommentInfo {
    pub(crate) comment_added: bool,
    pub(crate) hardbreak_added: bool,
    pub(crate) hardbreak_end: bool,
}

impl CommentInfo {
    pub(crate) fn update(&mut self, other: CommentInfo) {
        self.comment_added = self.comment_added || other.comment_added;
        self.hardbreak_added = self.hardbreak_added || other.hardbreak_added;
    }
}
