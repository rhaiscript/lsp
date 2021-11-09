//! This module contains all parsing-related tools including a
//! recursive-descent parser.

use crate::syntax::{SyntaxKind, SyntaxNode};
use rowan::GreenNode;
use thiserror::Error;

use self::context::Context;

mod context;
pub mod parsers;

/// A flexible parser.
/// 
/// Parsing happens via given parser functions (also found in [`parsers`]).
/// 
/// # Example Uses
/// 
/// ```
/// use rhai_rowan::parser::Parser;
/// // Parse some Rhai code (file).
/// let parser = Parser::new(r#"print("hello rhai!")"#);
/// let parse = parser.parse();
/// 
/// assert!(parse.errors.is_empty());
/// ```
/// 
/// ```
/// use rhai_rowan::parser::Parser;
/// use rhai_rowan::parser::parsers::parse_expr;
///
/// // Parse an expression explicitly.
/// let mut parser = Parser::new(r#"print("hello rhai!")"#);
/// parser.execute(parse_expr);
/// let parse = parser.finish();
/// 
/// assert!(parse.errors.is_empty());
/// ```
pub struct Parser<'src> {
    context: Context<'src>,
}

impl<'src> Parser<'src> {
    /// Create a new parser with the given source.
    #[must_use]
    pub fn new(source: &'src str) -> Self {
        Self {
            context: Context::new(source),
        }
    }

    /// Finish parsing.
    /// 
    /// # Panics
    /// 
    /// If no parser function was called,
    /// or a parser was left in an invalid state (bug).
    #[must_use]
    pub fn finish(self) -> Parse {
        self.context.finish()
    }
}

impl<'src> Parser<'src> {
    /// Execute a parser function for the given parser.
    pub fn execute<F: FnOnce(&mut Context)>(&mut self, f: F) {
        f(&mut self.context);
    }
}

/// The result of parsing, containing all errors
/// and the final green tree.
#[derive(Debug, Clone)]
pub struct Parse {
    /// Parse errors.
    pub errors: Vec<ParseError>,
    /// Parsed green tree.
    pub green: GreenNode,
}

impl Parse {
    /// Turn the result green tree into a CST.
    /// *This ignores errors*, the resulting tree
    /// can be potentially syntactically invalid.
    #[must_use]
    pub fn clone_syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green.clone())
    }

    /// Turn the result green tree into a CST.
    /// *This ignores errors*, the resulting tree
    /// can be potentially syntactically invalid.
    #[must_use]
    pub fn into_syntax(self) -> SyntaxNode {
        SyntaxNode::new_root(self.green)
    }
}

/// A parse (syntax) error.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("parse error at {range:?}: {kind}")]
pub struct ParseError {
    /// The span of the error in the parsed source.
    pub range: rowan::TextRange,
    /// Error kind.
    pub kind: ParseErrorKind,
}

impl ParseError {
    fn new(range: rowan::TextRange, kind: ParseErrorKind) -> Self {
        Self { range, kind }
    }
}

/// All the non-fatal parsing errors that can occur.
#[derive(Debug, PartialEq, Eq, Clone, Error)]
pub enum ParseErrorKind {
    #[error("unexpected EOF")]
    UnexpectedEof,

    #[error(r#"invalid input"#)]
    InvalidInput,

    #[error(r#"unexpected token"#)]
    UnexpectedToken,

    #[error(r#"expected token "{0:?}""#)]
    ExpectedToken(SyntaxKind),

    #[error(
        r#"expected one of "{}""#,
        .0
            .iter()
            .map(|s|format!("{:?}", s))
            .collect::<Vec<String>>().join(",")
    )]
    ExpectedOneOfTokens(Vec<SyntaxKind>),
}
