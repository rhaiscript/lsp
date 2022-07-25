//! The parser context is a separate module to limit
//! the API surface for the parser functions.

use std::{collections::HashMap, iter::Iterator};

use rowan::{Checkpoint, GreenNodeBuilder, TextRange, TextSize};

use crate::syntax::{
    AmbiguousTokens, Lexer,
    SyntaxKind::{self, *},
};

use super::{Parse, ParseError, ParseErrorKind};

/// A parser context for parser functions.
///
/// It cannot be constructed and can only be obtained
/// via a [`super::Parser`].
#[derive(Debug)]
pub struct Context<'src> {
    lexer: Lexer<'src>,
    current_token: Option<SyntaxKind>,
    last_token: Option<SyntaxKind>,
    green: GreenNodeBuilder<'static>,
    errors: Vec<ParseError>,
    ambiguous_tokens: Option<AmbiguousTokens<'src>>,

    /// User-provided custom operators.
    ///
    /// tamasfe: originally I wanted to treat all identifiers
    /// as valid operators, but it introduced too much ambiguity,
    /// so the user has to provide them.
    custom_ops: HashMap<String, Operator>,

    /// Tracks statements being separated by ";".
    statement_closed: bool,
    /// We are parsing a switch pattern expression.
    switch_pat_expr: bool,
}

impl<'src> Context<'src> {
    pub(crate) fn new(source: &'src str) -> Self {
        Self {
            lexer: Lexer::new(source),
            current_token: None,
            last_token: None,
            green: GreenNodeBuilder::new(),
            errors: Vec::new(),
            custom_ops: HashMap::default(),
            ambiguous_tokens: None,

            statement_closed: true,
            switch_pat_expr: false,
        }
    }

    pub(super) fn custom_op(&mut self, ident: String, op: Operator) {
        self.custom_ops.insert(ident, op);
    }

    pub(crate) fn finish(self) -> Parse {
        Parse {
            errors: self.errors,
            green: self.green.finish(),
        }
    }

    /// Get the next token.
    pub fn token(&mut self) -> Option<SyntaxKind> {
        // Eat insignificant tokens
        loop {
            if self.current_token.is_none() {
                if let Some(token) = self.ambiguous_tokens.as_mut().and_then(Iterator::next) {
                    self.current_token = Some(token);
                } else {
                    self.ambiguous_tokens = None;
                    self.current_token = self.lexer.next();
                }
            }

            match self.current_token {
                Some(COMMENT_BLOCK | COMMENT_LINE | WHITESPACE) => {
                    self.eat();
                    self.last_token = None;
                }
                Some(ERROR) => {
                    self.eat_error(ParseErrorKind::InvalidInput);
                    self.last_token = None;
                }
                Some(t @ (__AMBIGUOUS_INTEGER_AND_RANGE | __AMBIGUOUS_INTEGER_AND_IDENT)) => {
                    self.ambiguous_tokens = Some(AmbiguousTokens::new(
                        t,
                        self.lexer.slice(),
                        self.lexer.span(),
                    ));
                    self.current_token = None;
                }
                _ => break,
            }
        }

        self.current_token
    }

    /// Get the previously added token.
    #[must_use]
    pub fn previous_token(&self) -> Option<SyntaxKind> {
        self.last_token
    }

    /// "Eat" the current token, add it to the tree inside the current node.
    pub fn eat(&mut self) {
        if let Some(t) = self.current_token.take() {
            self.green.token(
                t.into(),
                self.ambiguous_tokens
                    .as_ref()
                    .map_or_else(|| self.lexer.slice(), AmbiguousTokens::slice),
            );
            self.last_token = Some(t);
        }
        self.current_token = None;
    }

    /// Insert a token into the tree.
    pub fn insert_token(&mut self, kind: SyntaxKind, s: impl AsRef<str>) {
        self.green.token(kind.into(), s.as_ref());
    }

    /// Discard the current token (if any).
    ///
    /// This should only be used to split a token into more tokens,
    /// and source code should never be thrown away.
    ///
    /// If no token was lexed (by calling [`Self::token`]), this is a no-op.
    pub fn discard(&mut self) {
        self.current_token = None;
    }

    /// Eat the current token with the given kind.
    pub fn eat_as(&mut self, kind: SyntaxKind) {
        if self.current_token.is_some() {
            self.green.token(
                kind.into(),
                self.ambiguous_tokens
                    .as_ref()
                    .map_or_else(|| self.lexer.slice(), AmbiguousTokens::slice),
            );
            self.last_token = Some(kind);
        }
        self.current_token = None;
    }

    /// Eat the current token and add a parse error.
    pub fn eat_error(&mut self, error: ParseErrorKind) {
        self.add_error_inner(error, true);
    }

    /// Eat the current token with the given kind and add a parse error.
    pub fn eat_error_as(&mut self, kind: SyntaxKind, error: ParseErrorKind) {
        self.add_error(error);
        self.eat_as(kind);
    }

    /// Add a parse error without touching the token or the tree.
    pub fn add_error(&mut self, error: ParseErrorKind) {
        self.add_error_inner(error, false);
    }

    /// Start a new node in the tree.
    pub fn start_node(&mut self, kind: SyntaxKind) {
        self.green.start_node(kind.into());
    }

    /// Finish the current node.
    pub fn finish_node(&mut self) {
        self.green.finish_node();
    }

    /// Create a node checkpoint.
    pub fn checkpoint(&mut self) -> Checkpoint {
        self.green.checkpoint()
    }

    /// Start a new node at the given checkpoint.
    pub fn start_node_at(&mut self, checkpoint: Checkpoint, kind: SyntaxKind) {
        self.green.start_node_at(checkpoint, kind.into());
    }

    /// Check whether the last statement was closed with `;`.
    ///
    /// Block-like statements are also self-closing.
    #[must_use]
    pub fn statement_closed(&self) -> bool {
        self.statement_closed
    }

    /// Signal that the last parsed statement is considered closed.
    pub fn set_statement_closed(&mut self, v: bool) {
        self.statement_closed = v;
    }

    #[must_use]
    pub fn slice(&self) -> &str {
        self.ambiguous_tokens
            .as_ref()
            .map_or_else(|| self.lexer.slice(), AmbiguousTokens::slice)
    }

    #[must_use]
    pub fn switch_pat_expr(&self) -> bool {
        self.switch_pat_expr
    }

    pub fn set_switch_pat_expr(&mut self, switch_pat_expr: bool) {
        self.switch_pat_expr = switch_pat_expr;
    }

    /// The binding power of the current token.
    #[must_use]
    pub fn infix_binding_power(&self) -> Option<(u8, u8)> {
        if let Some(bp) = self.current_token.and_then(SyntaxKind::infix_binding_power) {
            return Some(bp);
        }

        self.custom_ops.get(self.slice()).map(|o| o.binding_power)
    }

    fn add_error_inner(&mut self, error: ParseErrorKind, eat: bool) {
        const MAX_SIMILAR_ERROR_COUNT: usize = 10;

        #[cfg(not(fuzzing))]
        {
            tracing::trace!(%error, "syntax error");
        }
        let span = self
            .ambiguous_tokens
            .as_ref()
            .map_or_else(|| self.lexer.span(), AmbiguousTokens::span);

        let err = ParseError::new(
            TextRange::new(
                TextSize::from(span.start as u32),
                TextSize::from(span.end as u32),
            ),
            error,
        );

        // Escape hatch in case of infinite loops or recursions.
        //
        // If an error happens at the same location at least MAX_SIMILAR_ERROR_COUNT times,
        // we will surely eat the current token.
        let same_error_count = self
            .errors
            .iter()
            .rev()
            .take_while(|e| err.range == e.range)
            .count();

        self.errors.push(err);

        let eat = eat || (same_error_count + 1) >= MAX_SIMILAR_ERROR_COUNT;

        if eat {
            self.eat();
        }
    }
}

/// A custom Operator.
#[derive(Debug)]
pub struct Operator {
    pub binding_power: (u8, u8),
}

impl Default for Operator {
    fn default() -> Self {
        // Lowest by default.
        Self {
            binding_power: (1, 2),
        }
    }
}
