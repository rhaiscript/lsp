//! The parser context is a separate module to limit
//! the API surface for the parser functions.
#![allow(dead_code)]
use rowan::{GreenNodeBuilder, TextRange, TextSize};

use crate::syntax::{
    Lexer,
    SyntaxKind::{self, *},
};

use super::{Parse, ParseError, ParseErrorKind};

pub(crate) struct Context<'src> {
    lexer: Lexer<'src>,
    current_token: Option<SyntaxKind>,
    last_token: Option<SyntaxKind>,
    green: GreenNodeBuilder<'static>,
    errors: Vec<ParseError>,

    /// Tracks statements being separated by ";".
    statement_closed: bool,
}

impl<'src> Context<'src> {
    pub(crate) fn new(source: &'src str) -> Self {
        Self {
            lexer: Lexer::new(source),
            current_token: None,
            last_token: None,
            green: GreenNodeBuilder::new(),
            errors: Vec::new(),

            statement_closed: true,
        }
    }

    pub(crate) fn token(&mut self) -> Option<SyntaxKind> {
        // Eat insignificant tokens
        loop {
            if self.current_token.is_none() {
                self.current_token = self.lexer.next();
            }

            match self.current_token {
                Some(COMMENT_BLOCK) | Some(COMMENT_LINE) | Some(WHITESPACE) => {
                    self.eat();
                    self.last_token = None // We aren't interested in these.
                }
                Some(ERROR) => {
                    self.eat_error(ParseErrorKind::InvalidInput);
                    self.last_token = None // We aren't interested in these.
                }
                _ => break,
            }
        }

        self.current_token
    }

    pub(crate) fn previous_token(&self) -> Option<SyntaxKind> {
        self.last_token
    }

    pub(crate) fn eat(&mut self) {
        if let Some(t) = self.current_token.take() {
            self.green.token(t.into(), self.lexer.slice());
            self.last_token = Some(t);
        }
        self.current_token = None;
    }

    pub(crate) fn eat_as(&mut self, kind: SyntaxKind) {
        if self.current_token.is_some() {
            self.green.token(kind.into(), self.lexer.slice());
            self.last_token = Some(kind);
        }
        self.current_token = None;
    }

    pub(crate) fn eat_error(&mut self, error: ParseErrorKind) {
        self.add_error(error);
        self.eat();
    }

    pub(crate) fn eat_error_as(&mut self, kind: SyntaxKind, error: ParseErrorKind) {
        self.add_error(error);
        self.eat_as(kind);
    }

    pub(crate) fn add_error(&mut self, error: ParseErrorKind) {
        let span = self.lexer.span();
        self.errors.push(ParseError::new(
            TextRange::new(
                TextSize::from(span.start as u32),
                TextSize::from(span.end as u32),
            ),
            error,
        ));
    }

    pub(crate) fn start_node(&mut self, kind: SyntaxKind) {
        self.green.start_node(kind.into());
    }

    pub(crate) fn finish_node(&mut self) {
        self.green.finish_node();
    }

    pub(crate) fn finish(self) -> Parse {
        Parse {
            errors: self.errors,
            green: self.green.finish(),
        }
    }

    pub(crate) fn statement_closed(&self) -> bool {
        self.statement_closed
    }

    pub(crate) fn set_statement_closed(&mut self, v: bool) {
        self.statement_closed = v;
    }
}
