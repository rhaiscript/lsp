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
//!
//! This file also contains utilities for comments in
//! other positions.

#![allow(dead_code)]
use rhai_rowan::syntax::{
    SyntaxElement,
    SyntaxKind::{self, *},
    SyntaxNode,
};
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

        if let Some(t) = ws_and_comments.next() {
            if t.kind() == WHITESPACE && break_count(&t) > 0 {
                return Ok(info);
            } else if t.kind() == WHITESPACE {
                if let Some(c) = ws_and_comments.next() {
                    if c.kind() != WHITESPACE {
                        self.nbsp()?;
                        self.word(c.static_text().trim())?;
                        info.comment_added = true;
                    }
                }
            } else {
                self.nbsp()?;
                self.word(t.static_text().trim())?;
                info.comment_added = true;
            }
        }

        Ok(info)
    }

    pub(crate) fn standalone_comments_after(
        &mut self,
        node: &SyntaxNode,
        trailing_newlines: bool,
    ) -> io::Result<CommentInfo> {
        let mut info = CommentInfo::default();

        let mut ws_and_comments = node
            .siblings_with_tokens(Direction::Next)
            .skip(1)
            .skip_while(|ws| {
                if let Some(token) = ws.as_token() {
                    if break_count(token) == 0 {
                        return true;
                    }
                }

                false
            })
            .take_while(|e| matches!(e.kind(), WHITESPACE | COMMENT_BLOCK | COMMENT_LINE))
            .filter_map(SyntaxElement::into_token)
            .collect::<Vec<_>>();

        if !trailing_newlines {
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

    /// Add comments that are before the expression.
    pub(crate) fn comments_in_expr_before(&mut self, expr: &SyntaxNode) -> io::Result<()> {
        let comments_before = expr
            .children_with_tokens()
            .take_while(|t| t.as_node().is_none())
            .filter_map(SyntaxElement::into_token)
            .filter(|t| matches!(t.kind(), COMMENT_LINE | COMMENT_BLOCK));

        let mut first = true;
        let mut hardbreak_last = false;
        for comment in comments_before {
            if !first {
                self.space();
            }
            first = false;
            match comment.kind() {
                COMMENT_LINE => {
                    self.word(comment.static_text().trim_end())?;
                    self.hardbreak();
                    hardbreak_last = true;
                }
                COMMENT_BLOCK => {
                    self.word(comment.static_text().trim_end())?;
                }
                _ => unreachable!(),
            }
        }

        if !first && !hardbreak_last {
            self.space();
        }

        Ok(())
    }

    /// Add comments that are before the expression.
    pub(crate) fn comments_in_expr_after(&mut self, expr: &SyntaxNode) -> io::Result<()> {
        let comments_before = expr
            .children_with_tokens()
            .skip_while(|t| t.as_token().is_some())
            .filter_map(SyntaxElement::into_token)
            .filter(|t| matches!(t.kind(), COMMENT_LINE | COMMENT_BLOCK));

        for comment in comments_before {
            self.space();
            match comment.kind() {
                COMMENT_LINE => {
                    self.word(comment.static_text().trim_end())?;
                    self.hardbreak();
                }
                COMMENT_BLOCK => {
                    self.word(comment.static_text().trim_end())?;
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    /// Adds the child comments after the given syntax kind,
    /// but before the next node after that.
    ///
    /// Returns the amount of comments added
    pub(crate) fn comments_after_child(
        &mut self,
        node: &SyntaxNode,
        kind: SyntaxKind,
    ) -> io::Result<usize> {
        let comments_after = node
            .children_with_tokens()
            .skip_while(|t| t.kind() != kind)
            .skip_while(|t| t.kind() == kind)
            .filter_map(SyntaxElement::into_token)
            .filter(|t| matches!(t.kind(), COMMENT_BLOCK | COMMENT_LINE));

        let mut count = 0;

        let mut first = true;
        for comment in comments_after {
            count += 1;
            if first {
                self.space();
            }
            first = false;
            match comment.kind() {
                COMMENT_LINE => {
                    self.word(comment.static_text().trim_end())?;
                    self.hardbreak();
                }
                COMMENT_BLOCK => {
                    self.word(comment.static_text().trim_end())?;
                    self.space();
                }
                _ => unreachable!(),
            }
        }

        Ok(count)
    }

    /// Add comments that are before the node until another node
    /// is encountered.
    ///
    /// Returns the amount of comments that were added.
    pub(crate) fn add_standalone_comments_before(
        &mut self,
        expr: &SyntaxNode,
    ) -> io::Result<usize> {
        let mut comments_before = expr
            .siblings_with_tokens(Direction::Prev)
            .skip(1)
            .take_while(|t| t.as_node().is_none())
            .filter_map(SyntaxElement::into_token)
            .filter(|t| matches!(t.kind(), COMMENT_LINE | COMMENT_BLOCK))
            .collect::<Vec<_>>();

        comments_before.reverse();

        let count = comments_before.len();

        for comment in comments_before {
            self.word(comment.static_text().trim())?;
            self.hardbreak();
        }

        Ok(count)
    }

    /// Add comments that are before the node until another node
    /// is encountered.
    ///
    /// Returns the amount of comments that were added.
    pub(crate) fn add_standalone_comments_after(&mut self, expr: &SyntaxNode) -> io::Result<usize> {
        let comments_after = expr
            .siblings_with_tokens(Direction::Next)
            .skip(1)
            .take_while(|t| t.as_node().is_none())
            .filter_map(SyntaxElement::into_token)
            .filter(|t| matches!(t.kind(), COMMENT_LINE | COMMENT_BLOCK))
            .collect::<Vec<_>>();

        let count = comments_after.len();

        for comment in comments_after {
            self.word(comment.static_text().trim())?;
            self.hardbreak();
        }

        Ok(count)
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

        if other.comment_added || other.hardbreak_added {
            self.hardbreak_end = other.hardbreak_end;
        }
    }
}

pub(crate) fn comments_in_expr(expr: &SyntaxNode) -> bool {
    expr.children_with_tokens()
        .any(|c| matches!(c.kind(), COMMENT_LINE | COMMENT_BLOCK))
}
