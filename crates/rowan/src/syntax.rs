//! This module contains syntax kind declarations
//! and a Logos-based lexer implementation.

#![allow(clippy::manual_non_exhaustive)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use logos::{Lexer as LogosLexer, Logos};
use serde::{Deserialize, Serialize};
use std::ops::Range;

/// `SyntaxKind` represents all the node and token types (kinds) found in the grammar.
#[derive(
    Logos, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[repr(u16)]
pub enum SyntaxKind {
    // region: Keywords
    #[token("let")]
    KW_LET,
    #[token("const")]
    KW_CONST,
    #[token("for")]
    KW_FOR,
    #[token("do")]
    KW_DO,
    #[token("until")]
    KW_UNTIL,
    #[token("while")]
    KW_WHILE,
    #[token("in")]
    KW_IN,
    #[token("loop")]
    KW_LOOP,
    #[token("break")]
    KW_BREAK,
    #[token("continue")]
    KW_CONTINUE,
    #[token("if")]
    KW_IF,
    #[token("else")]
    KW_ELSE,
    #[token("switch")]
    KW_SWITCH,

    #[token("fn")]
    KW_FN,
    #[token("private")]
    KW_PRIVATE,
    #[token("return")]
    KW_RETURN,

    #[token("throw")]
    KW_THROW,
    #[token("try")]
    KW_TRY,
    #[token("catch")]
    KW_CATCH,

    #[token("import")]
    KW_IMPORT,
    #[token("export")]
    KW_EXPORT,
    #[token("as")]
    KW_AS,

    // The following are keywords,
    // but syntactically are just identifiers:

    // this
    // is_shared
    // Fn
    // call
    // curry
    // type_of
    // print
    // debug
    // eval
    // global

    // endregion

    // region: Reserved keywords
    // We might not do anything with these yet.
    #[token("var")]
    KW_VAR,
    #[token("static")]
    KW_STATIC,

    #[token("goto")]
    KW_GOTO,
    #[token("exit")]
    KW_EXIT,

    #[token("match")]
    KW_MATCH,
    #[token("case")]
    KW_CASE,

    #[token("public")]
    KW_PUBLIC,
    #[token("protected")]
    KW_PROTECTED,
    #[token("new")]
    KW_NEW,

    #[token("use")]
    KW_USE,
    #[token("with")]
    KW_WITH,
    #[token("module")]
    KW_MODULE,
    #[token("package")]
    KW_PACKAGE,
    #[token("super")]
    KW_SUPER,

    #[token("spawn")]
    KW_SPAWN,
    #[token("thread")]
    KW_THREAD,
    #[token("go")]
    KW_GO,
    #[token("sync")]
    KW_SYNC,
    #[token("async")]
    KW_ASYNC,
    #[token("await")]
    KW_AWAIT,
    #[token("yield")]
    KW_YIELD,

    #[token("default")]
    KW_DEFAULT,
    #[token("void")]
    KW_VOID,
    #[token("null")]
    KW_NULL,
    #[token("nil")]
    KW_NIL,
    // endregion

    // region: Punctuation and operators
    #[token(",")]
    PUNCT_COMMA,
    #[token(";")]
    PUNCT_SEMI,
    #[token(".")]
    PUNCT_DOT,
    #[token(":")]
    PUNCT_COLON,
    #[token("::")]
    PUNCT_COLON2,
    #[token("_")]
    PUNCT_UNDERSCORE,
    #[token("=>")]
    PUNCT_ARROW_FAT,
    #[token("->")]
    PUNCT_ARROW_THIN,

    #[token("(")]
    PUNCT_PAREN_START,
    #[token(")")]
    PUNCT_PAREN_END,

    #[token("[")]
    PUNCT_BRACKET_START,
    #[token("?[")]
    PUNCT_NULL_BRACKET_START,
    #[token("]")]
    PUNCT_BRACKET_END,

    #[token("#{")]
    PUNCT_MAP_START,
    #[token("{")]
    PUNCT_BRACE_START,
    #[token("}")]
    PUNCT_BRACE_END,

    #[token("?")]
    PUNCT_QUESTION_MARK, // Used only for types.

    #[token("+")]
    OP_ADD,
    #[token("-")]
    OP_SUB,
    #[token("*")]
    OP_MUL,
    #[token("/")]
    OP_DIV,
    #[token("%")]
    OP_MOD,
    #[token("**")]
    OP_POW,
    #[token(">>")]
    OP_SHIFT_RIGHT,
    #[token("<<")]
    OP_SHIFT_LEFT,
    #[token("&")]
    OP_BIT_AND,
    #[token("|")]
    OP_BIT_OR,
    #[token("^")]
    OP_BIT_XOR,

    #[token("..")]
    OP_RANGE,
    #[token("..=")]
    OP_RANGE_INCLUSIVE,

    #[token("?.")]
    OP_NULL_ACCESS,
    #[token("...")]
    OP_SPREAD,

    #[token("=")]
    OP_ASSIGN,
    #[token("+=")]
    OP_ADD_ASSIGN,
    #[token("-=")]
    OP_SUB_ASSIGN,
    #[token("*=")]
    OP_MUL_ASSIGN,
    #[token("/=")]
    OP_DIV_ASSIGN,
    #[token("%=")]
    OP_MOD_ASSIGN,
    #[token("**=")]
    OP_POW_ASSIGN,
    #[token(">>=")]
    OP_SHIFT_RIGHT_ASSIGN,
    #[token("<<=")]
    OP_SHIFT_LEFT_ASSIGN,
    #[token("&=")]
    OP_AND_ASSIGN,
    #[token("|=")]
    OP_OR_ASSIGN,
    #[token("^=")]
    OP_XOR_ASSIGN,

    #[token("==")]
    OP_EQ,
    #[token("!=")]
    OP_NOT_EQ,
    #[token(">")]
    OP_GT,
    #[token(">=")]
    OP_GT_EQ,
    #[token("<")]
    OP_LT,
    #[token("<=")]
    OP_LT_EQ,

    #[token("&&")]
    OP_BOOL_AND,
    #[token("||")]
    OP_BOOL_OR,
    #[token("??")]
    OP_NULL_OR,
    #[token("!")]
    OP_NOT,
    // endregion

    // region: Literals
    #[regex(r"[0-9][0-9_]*", priority = 3)]
    #[regex(r"0x[0-9A-Fa-f_]+")]
    #[regex(r"0o[0-7_]+")]
    #[regex(r"0b[01_]+")]
    LIT_INT,

    #[regex(
        r"([0-9][0-9_]*\.([0-9][0-9]*)(e([+-][0-9_]+|[0-9][0-9_]*))?)|([0-9][0-9_]*(\.([0-9][0-9]*)?))",
        priority = 2
    )]
    LIT_FLOAT,

    #[regex("true|false")]
    LIT_BOOL,

    #[token(r#"""#, |lex| {
        let mut escaped = false;
        let mut last_char = 0_u8;

        for (i, b) in lex.remainder().bytes().enumerate() {
            if !escaped && last_char == b'"' {
                if b != b'"' {
                    lex.bump(i);
                    return Some(());
                }
                last_char = 0_u8;
            } else {
                escaped = b == b'\\';
                last_char = b;
            }
        }

        if last_char == b'"' {
            lex.bump(lex.remainder().bytes().len());
            Some(())
        } else {
            None
        }
    })]
    #[token("`", |lex| {
        let mut escaped = false;
        let mut last_char = 0_u8;
        let mut interpolation_level = 0;

        for (i, b) in lex.remainder().bytes().enumerate() {
            if b == b'{' && last_char == b'$' {
                interpolation_level += 1;
                last_char = 0_u8;
                continue;
            }

            if interpolation_level > 0 {
                if last_char != b'\\' && b == b'}' {
                    interpolation_level -= 1;
                    last_char = 0_u8;
                } else {
                    last_char = b;
                }
                continue;
            }

            if !escaped && last_char == b'`' {
                if b != b'`' {
                    lex.bump(i);
                    return Some(());
                }
                last_char = 0_u8;
            } else {
                escaped = b == b'\\';
                last_char = b;
            }
        }

        if last_char == b'`' {
            lex.bump(lex.remainder().bytes().len());
            Some(())
        } else {
            None
        }
    })]
    LIT_STR,

    #[regex(r#"'\\.'|'.'|'\\x[A-Fa-f0-9][A-Fa-f0-9]'|'\\u[A-Fa-f0-9][A-Fa-f0-9][A-Fa-f0-9][A-Fa-f0-9]'|'\\U[A-Fa-f0-9][A-Fa-f0-9][A-Fa-f0-9][A-Fa-f0-9][A-Fa-f0-9][A-Fa-f0-9][A-Fa-f0-9][A-Fa-f0-9]'"#)]
    LIT_CHAR,
    // endregion

    // region: Other
    #[regex(r"#![^\n\r]*")]
    SHEBANG,

    #[regex("_*[A-Za-z][0-9A-Za-z_]*")]
    IDENT,

    // ///////... is not a block comment
    #[regex(r"(//|////)[^\n\r]*")]
    COMMENT_LINE,

    #[regex(r"///[^\n\r]*")]
    COMMENT_LINE_DOC,

    // /******... is not a block comment
    #[regex(r"/\*|/\*\*\*", lex_multi_line_comment)]
    COMMENT_BLOCK,

    #[token("/**", lex_multi_line_comment)]
    COMMENT_BLOCK_DOC,

    #[regex(r"[ \t\r\n\f]+")]
    WHITESPACE,
    #[error]
    ERROR,
    // endregion

    // region: ambiguous tokens
    /// The following is used to resolve ambiguity
    /// between floats and integers with ranges (#62).
    ///
    /// If this token is encountered it must be further processed
    /// with [`AmbiguousTokens`].
    #[regex(r#"[0-9][0-9_]*\.\.=?"#)]
    __AMBIGUOUS_INTEGER_AND_RANGE,

    // endregion

    // region: Nodes
    // This region is generated from ungrammar, do not touch it!
    LIT,
    PATH,
    RHAI,
    STMT,
    ITEM,
    DOC,
    EXPR,
    EXPR_IDENT,
    EXPR_PATH,
    EXPR_LIT,
    EXPR_LET,
    EXPR_CONST,
    EXPR_BLOCK,
    EXPR_UNARY,
    EXPR_BINARY,
    EXPR_PAREN,
    EXPR_ARRAY,
    EXPR_INDEX,
    EXPR_OBJECT,
    EXPR_CALL,
    EXPR_CLOSURE,
    EXPR_IF,
    EXPR_LOOP,
    EXPR_FOR,
    EXPR_WHILE,
    EXPR_BREAK,
    EXPR_CONTINUE,
    EXPR_SWITCH,
    EXPR_RETURN,
    EXPR_FN,
    EXPR_EXPORT,
    EXPR_IMPORT,
    EXPR_TRY,
    EXPR_THROW,
    OBJECT_FIELD,
    ARG_LIST,
    PARAM_LIST,
    PARAM,
    PAT,
    SWITCH_ARM_LIST,
    SWITCH_ARM,
    SWITCH_ARM_CONDITION,
    EXPORT_TARGET,
    EXPORT_IDENT,
    PAT_TUPLE,
    PAT_IDENT,
    RHAI_DEF,
    DEF_MODULE_DECL,
    DEF_STMT,
    DEF_ITEM,
    DEF,
    DEF_MODULE,
    DEF_IMPORT,
    DEF_CONST,
    DEF_FN,
    DEF_OP,
    DEF_TYPE,
    TYPE,
    DEF_LET,
    TYPE_LIST,
    TYPED_PARAM_LIST,
    TYPE_IDENT,
    TYPE_LIT,
    TYPE_OBJECT,
    TYPE_ARRAY,
    TYPE_TUPLE,
    TYPE_UNKNOWN,
    TYPE_GENERICS,
    TYPE_OBJECT_FIELD,
    TYPED_PARAM,
    TYPE_UNION,
    // endregion

    // A marker to safely cast between u16 and syntax kinds.
    #[doc(hidden)]
    __LAST,
}

impl SyntaxKind {
    /// Whether the syntax kind is a reserved keyword.
    #[must_use]
    pub fn is_reserved_keyword(&self) -> bool {
        self >= &SyntaxKind::KW_VAR && self <= &SyntaxKind::KW_NIL
    }

    /// Whether the syntax kind belongs in a definition file.
    #[must_use]
    pub fn is_def(&self) -> bool {
        self >= &SyntaxKind::RHAI_DEF && self <= &SyntaxKind::DEF_FN
    }
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lang {}
impl rowan::Language for Lang {
    type Kind = SyntaxKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= SyntaxKind::__LAST as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNode = rowan::SyntaxNode<Lang>;
pub type SyntaxToken = rowan::SyntaxToken<Lang>;
pub type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;

pub(crate) struct Lexer<'source> {
    lexer: LogosLexer<'source, SyntaxKind>,
    peeked: Option<Option<SyntaxKind>>,
}

impl core::fmt::Debug for Lexer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lexer")
            .field("peeked", &self.peeked)
            .finish()
    }
}

impl<'source> Lexer<'source> {
    pub(crate) fn new(source: &'source str) -> Self {
        Self {
            lexer: SyntaxKind::lexer(source),
            peeked: None,
        }
    }

    pub(crate) fn peek(&mut self) -> Option<SyntaxKind> {
        if self.peeked.is_none() {
            self.peeked = Some(self.lexer.next());
        }
        self.peeked.unwrap()
    }

    pub(crate) fn span(&self) -> Range<usize> {
        self.lexer.span()
    }

    pub(crate) fn slice(&self) -> &'source str {
        self.lexer.slice()
    }
}

impl<'source> Iterator for Lexer<'source> {
    type Item = SyntaxKind;

    fn next(&mut self) -> Option<SyntaxKind> {
        self.peeked
            .take()
            .map_or_else(|| self.lexer.next(), |peeked| peeked)
    }
}

// multi-line comments ending with "*/" have to be manually parsed
// to avoid yet another insane regex.
#[allow(clippy::unnecessary_wraps)]
fn lex_multi_line_comment(lex: &mut LogosLexer<SyntaxKind>) -> Option<()> {
    let mut start = 1;
    let mut to_bump = 0;

    let mut last_char = 0_u8;

    for c in lex.remainder().bytes() {
        to_bump += 1;

        match (last_char, c) {
            (b'/', b'*') => {
                start += 1;
            }
            (b'*', b'/') => {
                start -= 1;
            }
            _ => {}
        }

        last_char = c;

        if start == 0 {
            break;
        }
    }

    lex.bump(to_bump);

    Some(())
}

#[derive(Debug)]
pub struct AmbiguousTokens<'lexer> {
    last_slice: Option<&'lexer str>,
    last_span: Option<Range<usize>>,
    token: AmbiguousToken<'lexer>,
}

impl<'lexer> AmbiguousTokens<'lexer> {
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn new(token: SyntaxKind, slice: &'lexer str, span: Range<usize>) -> Self {
        match token {
            SyntaxKind::__AMBIGUOUS_INTEGER_AND_RANGE => {
                let range_idx = slice.as_bytes().iter().position(|b| *b == b'.').unwrap();
                let integer = &slice[..range_idx];
                let range = &slice[range_idx..];

                Self {
                    last_slice: None,
                    last_span: None,
                    token: AmbiguousToken::IntegerAndRange {
                        integer: Some((
                            SyntaxKind::LIT_INT,
                            integer,
                            span.start..(span.start + range_idx),
                        )),
                        range: Some((
                            if range.ends_with('=') {
                                SyntaxKind::OP_RANGE_INCLUSIVE
                            } else {
                                SyntaxKind::OP_RANGE
                            },
                            range,
                            (span.start + range_idx)..span.end,
                        )),
                    },
                }
            }
            _ => unreachable!("unambiguous token passed"),
        }
    }

    #[must_use]
    pub fn slice(&self) -> &str {
        self.last_slice.unwrap_or("")
    }

    #[must_use]
    pub fn span(&self) -> Range<usize> {
        self.last_span.clone().unwrap_or_default()
    }
}

impl Iterator for AmbiguousTokens<'_> {
    type Item = SyntaxKind;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.token {
            AmbiguousToken::IntegerAndRange { integer, range } => {
                if let Some((token, slice, span)) = integer.take() {
                    self.last_slice = Some(slice);
                    self.last_span = Some(span);
                    Some(token)
                } else if let Some((token, slice, span)) = range.take() {
                    self.last_slice = Some(slice);
                    self.last_span = Some(span);
                    Some(token)
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Debug)]
enum AmbiguousToken<'lexer> {
    IntegerAndRange {
        integer: Option<(SyntaxKind, &'lexer str, Range<usize>)>,
        range: Option<(SyntaxKind, &'lexer str, Range<usize>)>,
    },
}
