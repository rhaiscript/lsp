#![allow(dead_code)]
use rhai_rowan::syntax::{SyntaxKind::*, SyntaxNode, SyntaxToken};
use rowan::Direction;

use crate::algorithm::{self, BeginToken, BreakToken, Breaks, Formatter};
use std::{
    io::{self, Write},
    mem,
};

impl<W: Write> Formatter<W> {
    pub(crate) fn ibox(&mut self, indent: isize) {
        self.scan_begin(BeginToken {
            offset: indent,
            breaks: Breaks::Inconsistent,
        });
    }

    pub(crate) fn cbox(&mut self, indent: isize) {
        self.scan_begin(BeginToken {
            offset: indent,
            breaks: Breaks::Consistent,
        });
    }

    pub(crate) fn end(&mut self) {
        self.scan_end();
    }

    pub(crate) fn word(&mut self, wrd: &'static str) -> io::Result<()> {
        self.scan_string(wrd)
    }

    fn spaces(&mut self, n: usize) {
        self.scan_break(BreakToken {
            blank_space: n,
            ..BreakToken::default()
        });
    }

    pub(crate) fn zerobreak(&mut self) {
        self.spaces(0);
    }

    pub(crate) fn space(&mut self) {
        self.spaces(1);
    }

    pub(crate) fn nbsp(&mut self) -> io::Result<()> {
        self.word(" ")
    }

    pub(crate) fn hardbreak(&mut self) {
        self.spaces(algorithm::SIZE_INFINITY as usize);
    }

    pub(crate) fn hardbreaks(&mut self, count: u64) {
        for _ in 0..count.min(self.options.max_empty_lines + 1) {
            self.spaces(algorithm::SIZE_INFINITY as usize);
        }
    }

    pub(crate) fn space_if_nonempty(&mut self) {
        self.scan_break(BreakToken {
            blank_space: 1,
            if_nonempty: true,
            ..BreakToken::default()
        });
    }

    pub(crate) fn hardbreak_if_nonempty(&mut self) {
        self.scan_break(BreakToken {
            blank_space: algorithm::SIZE_INFINITY as usize,
            if_nonempty: true,
            ..BreakToken::default()
        });
    }

    pub(crate) fn trailing_comma(&mut self, is_last: bool) -> io::Result<()> {
        if is_last {
            self.scan_break(BreakToken {
                pre_break: Some(','),
                ..BreakToken::default()
            });
        } else {
            self.word(",")?;
            self.space();
        }

        Ok(())
    }

    pub(crate) fn trailing_comma_or_space(&mut self, is_last: bool) -> io::Result<()> {
        if is_last {
            self.scan_break(BreakToken {
                blank_space: 1,
                pre_break: Some(','),
                ..BreakToken::default()
            });
        } else {
            self.word(",")?;
            self.space();
        }

        Ok(())
    }

    pub(crate) fn neverbreak(&mut self) {
        self.scan_break(BreakToken {
            never_break: true,
            ..BreakToken::default()
        });
    }
}

pub(crate) trait ScopedStatic {
    fn static_text(&self) -> &'static str;
}

impl ScopedStatic for SyntaxToken {
    fn static_text(&self) -> &'static str {
        // SAFETY: we guarantee that the syntax token
        // outlives the formatting process.
        unsafe { mem::transmute(self.text()) }
    }
}

pub(crate) fn breaks_before(node: &SyntaxNode) -> u64 {
    if let Some(elem) = node.siblings_with_tokens(Direction::Prev).nth(1) {
        if let Some(t) = elem.into_token() {
            if t.kind() == WHITESPACE {
                return break_count(&t);
            }
        }
    }

    0
}

pub(crate) fn break_count(t: &SyntaxToken) -> u64 {
    t.text().chars().filter(|c| *c == '\n').count() as u64
}

#[test]
fn lol() {
    let src = r#"
#{
    a: 2,

    b: 3,
}"#;

    let node = rhai_rowan::parser::Parser::new(src)
        .parse_script()
        .into_syntax();

    println!("{node:#?}");
}
