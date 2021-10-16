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
    // We don't do anything with these yet.
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

    #[token("(")]
    PUNCT_PAREN_START,
    #[token(")")]
    PUNCT_PAREN_END,

    #[token("[")]
    PUNCT_BRACKET_START,
    #[token("]")]
    PUNCT_BRACKET_END,

    #[token("#{")]
    PUNCT_MAP_START,
    #[token("{")]
    PUNCT_BRACE_START,
    #[token("}")]
    PUNCT_BRACE_END,

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
    #[token("!")]
    OP_NOT,
    // endregion

    // region: Literals
    #[regex(r"[0-9_]+", priority = 3)]
    #[regex(r"0x[0-9A-Fa-f_]+")]
    #[regex(r"0o[0-7_]+")]
    #[regex(r"0b(0|1|_)+")]
    LIT_INT,

    #[regex(r"([0-9_]+(\.[0-9_]+)?([eE][+-]?[0-9_]+)?)", priority = 2)]
    LIT_FLOAT,

    #[regex("true|false")]
    LIT_BOOL,

    #[token(r#"""#, |lex| {
        let mut escaped = false;

        for (i, b) in lex.remainder().bytes().enumerate() {
            if !escaped && b == b'"' {
                lex.bump(i + 1);
                return Some(());
            }
            escaped = b == b'\\';
        }

        None
    })]
    #[token("`", |lex| {
        let mut escaped = false;
        let mut last_char = 0_u8;
        let mut interpolation_level = 0;

        for (i, b) in lex.remainder().bytes().enumerate() {
            if b == b'{' && last_char == b'$' {
                interpolation_level += 1;
                continue;
            }

            if interpolation_level > 0 {
                if b == b'}' {
                    interpolation_level -= 1;
                }
                continue;
            }

            if !escaped && b == b'`' {
                lex.bump(i + 1);
                return Some(());
            }
            escaped = b == b'\\';
            last_char = b;
        }

        None
    })]
    LIT_STR,

    #[regex(r#"'\\.'|'.'|'\\x[A-Fa-f0-9][A-Fa-f0-9]'|'\\u[A-Fa-f0-9][A-Fa-f0-9][A-Fa-f0-9][A-Fa-f0-9]'|'\\U[A-Fa-f0-9][A-Fa-f0-9][A-Fa-f0-9][A-Fa-f0-9][A-Fa-f0-9][A-Fa-f0-9][A-Fa-f0-9][A-Fa-f0-9]'"#)]
    LIT_CHAR,
    // endregion

    // region: Other
    #[regex(r"#![^\n\r]*")]
    SHEBANG,

    #[regex("[A-Za-z_][0-9A-Za-z_]*")]
    IDENT,

    #[regex(r"//[^\n\r]*")]
    COMMENT_LINE,

    #[regex(r"///[^\n\r]*")]
    COMMENT_LINE_DOC,

    #[token("/*", lex_multi_line_comment)]
    COMMENT_BLOCK,

    #[token("/**", lex_multi_line_comment)]
    COMMENT_BLOCK_DOC,

    #[regex(r"[ \t\n\f]+")]
    WHITESPACE,
    #[error]
    ERROR,
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
    OBJECT_FIELD,
    ARG_LIST,
    PARAM_LIST,
    PARAM,
    PAT,
    SWITCH_ARM_LIST,
    SWITCH_ARM,
    EXPORT_TARGET,
    EXPORT_IDENT,
    PAT_TUPLE,
    PAT_IDENT,
    // endregion

    // A marker to safely cast between u16 and syntax kinds.
    #[doc(hidden)]
    __LAST,
}

impl SyntaxKind {
    /// Whether the syntax kind is a reserved keyword.
    #[must_use]
    pub fn is_reserved_keyword(&self) -> bool {
        self >= &SyntaxKind::KW_VAR && self < &SyntaxKind::KW_NIL
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
