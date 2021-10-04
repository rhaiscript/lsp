use crate::syntax::{SyntaxKind, SyntaxNode};
use rowan::GreenNode;
use thiserror::Error;

use self::context::Context;

mod context;
pub mod parsers;

pub struct Parser<'src> {
    context: Context<'src>,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            context: Context::new(source),
        }
    }

    pub fn finish(self) -> Parse {
        self.context.finish()
    }
}

impl<'src> Parser<'src> {
    pub fn execute<F: FnOnce(&mut Context)>(&mut self, f: F) {
        f(&mut self.context)
    }
}

/// The result of parsing.
#[derive(Debug, Clone)]
pub struct Parse {
    pub errors: Vec<ParseError>,
    pub green: GreenNode,
}

impl Parse {
    pub fn into_syntax(self) -> SyntaxNode {
        SyntaxNode::new_root(self.green)
    }
}

#[derive(Debug, Clone, Error)]
#[error("parse error at {range:?}: {kind}")]
pub struct ParseError {
    pub range: rowan::TextRange,
    pub kind: ParseErrorKind,
}

impl ParseError {
    pub fn new(range: rowan::TextRange, kind: ParseErrorKind) -> Self {
        Self { range, kind }
    }
}

#[derive(Debug, Clone, Error)]
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
            .intersperse(String::from(", "))
            .collect::<String>()
    )]
    ExpectedOneOfTokens(Vec<SyntaxKind>),
}
