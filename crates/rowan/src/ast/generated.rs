//! This file was generated from ungrammar, do not touch it!
/// A macro for using tokens in a more humanly way, e.g. `T!["="]`.
# [macro_export] macro_rules ! T { ["lit_int"] => { $ crate :: syntax :: SyntaxKind :: LIT_INT } ; ["lit_float"] => { $ crate :: syntax :: SyntaxKind :: LIT_FLOAT } ; ["lit_str"] => { $ crate :: syntax :: SyntaxKind :: LIT_STR } ; ["lit_bool"] => { $ crate :: syntax :: SyntaxKind :: LIT_BOOL } ; ["lit_char"] => { $ crate :: syntax :: SyntaxKind :: LIT_CHAR } ; ["ident"] => { $ crate :: syntax :: SyntaxKind :: IDENT } ; ["::"] => { $ crate :: syntax :: SyntaxKind :: PUNCT_COLON2 } ; ["shebang"] => { $ crate :: syntax :: SyntaxKind :: SHEBANG } ; ["comment_line_doc"] => { $ crate :: syntax :: SyntaxKind :: COMMENT_LINE_DOC } ; ["comment_block_doc"] => { $ crate :: syntax :: SyntaxKind :: COMMENT_BLOCK_DOC } ; [";"] => { $ crate :: syntax :: SyntaxKind :: PUNCT_SEMI } ; ["let"] => { $ crate :: syntax :: SyntaxKind :: KW_LET } ; ["="] => { $ crate :: syntax :: SyntaxKind :: OP_ASSIGN } ; ["const"] => { $ crate :: syntax :: SyntaxKind :: KW_CONST } ; ["{"] => { $ crate :: syntax :: SyntaxKind :: PUNCT_BRACE_START } ; ["}"] => { $ crate :: syntax :: SyntaxKind :: PUNCT_BRACE_END } ; ["+"] => { $ crate :: syntax :: SyntaxKind :: OP_ADD } ; ["-"] => { $ crate :: syntax :: SyntaxKind :: OP_SUB } ; ["!"] => { $ crate :: syntax :: SyntaxKind :: OP_NOT } ; ["||"] => { $ crate :: syntax :: SyntaxKind :: OP_BOOL_OR } ; ["&&"] => { $ crate :: syntax :: SyntaxKind :: OP_BOOL_AND } ; ["=="] => { $ crate :: syntax :: SyntaxKind :: OP_EQ } ; ["!="] => { $ crate :: syntax :: SyntaxKind :: OP_NOT_EQ } ; ["<="] => { $ crate :: syntax :: SyntaxKind :: OP_LT_EQ } ; [">="] => { $ crate :: syntax :: SyntaxKind :: OP_GT_EQ } ; ["<"] => { $ crate :: syntax :: SyntaxKind :: OP_LT } ; [">"] => { $ crate :: syntax :: SyntaxKind :: OP_GT } ; ["*"] => { $ crate :: syntax :: SyntaxKind :: OP_MUL } ; ["**"] => { $ crate :: syntax :: SyntaxKind :: OP_POW } ; ["/"] => { $ crate :: syntax :: SyntaxKind :: OP_DIV } ; ["%"] => { $ crate :: syntax :: SyntaxKind :: OP_MOD } ; ["<<"] => { $ crate :: syntax :: SyntaxKind :: OP_SHIFT_LEFT } ; [">>"] => { $ crate :: syntax :: SyntaxKind :: OP_SHIFT_RIGHT } ; ["^"] => { $ crate :: syntax :: SyntaxKind :: OP_BIT_XOR } ; ["|"] => { $ crate :: syntax :: SyntaxKind :: OP_BIT_OR } ; ["&"] => { $ crate :: syntax :: SyntaxKind :: OP_BIT_AND } ; ["+="] => { $ crate :: syntax :: SyntaxKind :: OP_ADD_ASSIGN } ; ["/="] => { $ crate :: syntax :: SyntaxKind :: OP_DIV_ASSIGN } ; ["*="] => { $ crate :: syntax :: SyntaxKind :: OP_MUL_ASSIGN } ; ["**="] => { $ crate :: syntax :: SyntaxKind :: OP_POW_ASSIGN } ; ["%="] => { $ crate :: syntax :: SyntaxKind :: OP_MOD_ASSIGN } ; [">>="] => { $ crate :: syntax :: SyntaxKind :: OP_SHIFT_RIGHT_ASSIGN } ; ["<<="] => { $ crate :: syntax :: SyntaxKind :: OP_SHIFT_LEFT_ASSIGN } ; ["-="] => { $ crate :: syntax :: SyntaxKind :: OP_SUB_ASSIGN } ; ["|="] => { $ crate :: syntax :: SyntaxKind :: OP_OR_ASSIGN } ; ["&="] => { $ crate :: syntax :: SyntaxKind :: OP_AND_ASSIGN } ; ["^="] => { $ crate :: syntax :: SyntaxKind :: OP_XOR_ASSIGN } ; ["."] => { $ crate :: syntax :: SyntaxKind :: PUNCT_DOT } ; ["("] => { $ crate :: syntax :: SyntaxKind :: PUNCT_PAREN_START } ; [")"] => { $ crate :: syntax :: SyntaxKind :: PUNCT_PAREN_END } ; ["["] => { $ crate :: syntax :: SyntaxKind :: PUNCT_BRACKET_START } ; [","] => { $ crate :: syntax :: SyntaxKind :: PUNCT_COMMA } ; ["]"] => { $ crate :: syntax :: SyntaxKind :: PUNCT_BRACKET_END } ; ["#{"] => { $ crate :: syntax :: SyntaxKind :: PUNCT_MAP_START } ; [":"] => { $ crate :: syntax :: SyntaxKind :: PUNCT_COLON } ; ["if"] => { $ crate :: syntax :: SyntaxKind :: KW_IF } ; ["else"] => { $ crate :: syntax :: SyntaxKind :: KW_ELSE } ; ["loop"] => { $ crate :: syntax :: SyntaxKind :: KW_LOOP } ; ["for"] => { $ crate :: syntax :: SyntaxKind :: KW_FOR } ; ["in"] => { $ crate :: syntax :: SyntaxKind :: KW_IN } ; ["while"] => { $ crate :: syntax :: SyntaxKind :: KW_WHILE } ; ["break"] => { $ crate :: syntax :: SyntaxKind :: KW_BREAK } ; ["continue"] => { $ crate :: syntax :: SyntaxKind :: KW_CONTINUE } ; ["switch"] => { $ crate :: syntax :: SyntaxKind :: KW_SWITCH } ; ["_"] => { $ crate :: syntax :: SyntaxKind :: PUNCT_UNDERSCORE } ; ["=>"] => { $ crate :: syntax :: SyntaxKind :: PUNCT_ARROW_FAT } ; ["return"] => { $ crate :: syntax :: SyntaxKind :: KW_RETURN } ; ["fn"] => { $ crate :: syntax :: SyntaxKind :: KW_FN } ; ["import"] => { $ crate :: syntax :: SyntaxKind :: KW_IMPORT } ; ["as"] => { $ crate :: syntax :: SyntaxKind :: KW_AS } ; ["export"] => { $ crate :: syntax :: SyntaxKind :: KW_EXPORT } ; } pub use T ;
use crate::syntax::{SyntaxKind::*, SyntaxNode, SyntaxToken};
pub trait AstNode: Sized {
    fn can_cast(syntax: &SyntaxNode) -> bool;
    fn cast(syntax: SyntaxNode) -> Option<Self>;
    fn syntax(&self) -> SyntaxNode;
}
#[derive(Debug, Clone)]
pub struct Lit(SyntaxNode);
impl AstNode for Lit {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == LIT }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if !Self::can_cast(&syntax) {
            return None;
        }
        Some(Self(syntax))
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl Lit {
    pub fn token(&self) -> Option<SyntaxToken> {
        self.0.first_child_or_token().and_then(|e| e.into_token())
    }
}
#[derive(Debug, Clone)]
pub struct Path(SyntaxNode);
impl AstNode for Path {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == PATH }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl Path {
    pub fn root(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != IDENT {
                return None;
            }
            t.into_token()
        })
    }
}
#[derive(Debug, Clone)]
pub struct File(SyntaxNode);
impl AstNode for File {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == FILE }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl File {
    pub fn statements(&self) -> impl Iterator<Item = Stmt> {
        self.0.children().filter_map(Stmt::cast)
    }
}
#[derive(Debug, Clone)]
pub struct Stmt(SyntaxNode);
impl AstNode for Stmt {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == STMT }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
#[derive(Debug, Clone)]
pub struct Item(SyntaxNode);
impl AstNode for Item {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == ITEM }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl Item {
    pub fn docs(&self) -> impl Iterator<Item = Doc> { self.0.children().filter_map(Doc::cast) }
    pub fn expr(&self) -> Option<Expr> { self.0.children().find_map(Expr::cast) }
}
#[derive(Debug, Clone)]
pub struct Doc(SyntaxNode);
impl AstNode for Doc {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == DOC }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if !Self::can_cast(&syntax) {
            return None;
        }
        Some(Self(syntax))
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl Doc {
    pub fn token(&self) -> Option<SyntaxToken> {
        self.0.first_child_or_token().and_then(|e| e.into_token())
    }
}
#[derive(Debug, Clone)]
pub enum Expr {
    Ident(ExprIdent),
    Path(ExprPath),
    Lit(ExprLit),
    Let(ExprLet),
    Const(ExprConst),
    Block(ExprBlock),
    Unary(ExprUnary),
    Binary(ExprBinary),
    Paren(ExprParen),
    Array(ExprArray),
    Index(ExprIndex),
    Object(ExprObject),
    Call(ExprCall),
    Closure(ExprClosure),
    If(ExprIf),
    Loop(ExprLoop),
    For(ExprFor),
    While(ExprWhile),
    Break(ExprBreak),
    Continue(ExprContinue),
    Switch(ExprSwitch),
    Return(ExprReturn),
    Fn(ExprFn),
    Import(ExprImport),
}
impl AstNode for Expr {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if !Self::can_cast(&syntax) {
            return None;
        }
        let first_child = syntax.first_child()?;
        match first_child.kind() {
            EXPR_IDENT => Some(Self::Ident(ExprIdent::cast(first_child)?)),
            EXPR_PATH => Some(Self::Path(ExprPath::cast(first_child)?)),
            EXPR_LIT => Some(Self::Lit(ExprLit::cast(first_child)?)),
            EXPR_LET => Some(Self::Let(ExprLet::cast(first_child)?)),
            EXPR_CONST => Some(Self::Const(ExprConst::cast(first_child)?)),
            EXPR_BLOCK => Some(Self::Block(ExprBlock::cast(first_child)?)),
            EXPR_UNARY => Some(Self::Unary(ExprUnary::cast(first_child)?)),
            EXPR_BINARY => Some(Self::Binary(ExprBinary::cast(first_child)?)),
            EXPR_PAREN => Some(Self::Paren(ExprParen::cast(first_child)?)),
            EXPR_ARRAY => Some(Self::Array(ExprArray::cast(first_child)?)),
            EXPR_INDEX => Some(Self::Index(ExprIndex::cast(first_child)?)),
            EXPR_OBJECT => Some(Self::Object(ExprObject::cast(first_child)?)),
            EXPR_CALL => Some(Self::Call(ExprCall::cast(first_child)?)),
            EXPR_CLOSURE => Some(Self::Closure(ExprClosure::cast(first_child)?)),
            EXPR_IF => Some(Self::If(ExprIf::cast(first_child)?)),
            EXPR_LOOP => Some(Self::Loop(ExprLoop::cast(first_child)?)),
            EXPR_FOR => Some(Self::For(ExprFor::cast(first_child)?)),
            EXPR_WHILE => Some(Self::While(ExprWhile::cast(first_child)?)),
            EXPR_BREAK => Some(Self::Break(ExprBreak::cast(first_child)?)),
            EXPR_CONTINUE => Some(Self::Continue(ExprContinue::cast(first_child)?)),
            EXPR_SWITCH => Some(Self::Switch(ExprSwitch::cast(first_child)?)),
            EXPR_RETURN => Some(Self::Return(ExprReturn::cast(first_child)?)),
            EXPR_FN => Some(Self::Fn(ExprFn::cast(first_child)?)),
            EXPR_IMPORT => Some(Self::Import(ExprImport::cast(first_child)?)),
            _ => None,
        }
    }
    fn syntax(&self) -> SyntaxNode {
        match self {
            Self::Ident(t) => t.syntax().parent().unwrap(),
            Self::Path(t) => t.syntax().parent().unwrap(),
            Self::Lit(t) => t.syntax().parent().unwrap(),
            Self::Let(t) => t.syntax().parent().unwrap(),
            Self::Const(t) => t.syntax().parent().unwrap(),
            Self::Block(t) => t.syntax().parent().unwrap(),
            Self::Unary(t) => t.syntax().parent().unwrap(),
            Self::Binary(t) => t.syntax().parent().unwrap(),
            Self::Paren(t) => t.syntax().parent().unwrap(),
            Self::Array(t) => t.syntax().parent().unwrap(),
            Self::Index(t) => t.syntax().parent().unwrap(),
            Self::Object(t) => t.syntax().parent().unwrap(),
            Self::Call(t) => t.syntax().parent().unwrap(),
            Self::Closure(t) => t.syntax().parent().unwrap(),
            Self::If(t) => t.syntax().parent().unwrap(),
            Self::Loop(t) => t.syntax().parent().unwrap(),
            Self::For(t) => t.syntax().parent().unwrap(),
            Self::While(t) => t.syntax().parent().unwrap(),
            Self::Break(t) => t.syntax().parent().unwrap(),
            Self::Continue(t) => t.syntax().parent().unwrap(),
            Self::Switch(t) => t.syntax().parent().unwrap(),
            Self::Return(t) => t.syntax().parent().unwrap(),
            Self::Fn(t) => t.syntax().parent().unwrap(),
            Self::Import(t) => t.syntax().parent().unwrap(),
        }
    }
}
#[derive(Debug, Clone)]
pub struct ExprIdent(SyntaxNode);
impl AstNode for ExprIdent {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_IDENT }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprIdent {
    pub fn ident_token(&self) -> Option<SyntaxToken> {
        self.0.first_child_or_token().and_then(|e| e.into_token())
    }
}
#[derive(Debug, Clone)]
pub struct ExprPath(SyntaxNode);
impl AstNode for ExprPath {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_PATH }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprPath {
    pub fn path(&self) -> Option<Path> { self.0.first_child().and_then(Path::cast) }
}
#[derive(Debug, Clone)]
pub struct ExprLit(SyntaxNode);
impl AstNode for ExprLit {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_LIT }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprLit {
    pub fn lit(&self) -> Option<Lit> { self.0.first_child().and_then(Lit::cast) }
}
#[derive(Debug, Clone)]
pub struct ExprLet(SyntaxNode);
impl AstNode for ExprLet {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_LET }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprLet {
    pub fn kw_let_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != KW_LET {
                return None;
            }
            t.into_token()
        })
    }
    pub fn ident_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != IDENT {
                return None;
            }
            t.into_token()
        })
    }
}
#[derive(Debug, Clone)]
pub struct ExprConst(SyntaxNode);
impl AstNode for ExprConst {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_CONST }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprConst {
    pub fn kw_const_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != KW_CONST {
                return None;
            }
            t.into_token()
        })
    }
    pub fn ident_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != IDENT {
                return None;
            }
            t.into_token()
        })
    }
    pub fn op_assign_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != OP_ASSIGN {
                return None;
            }
            t.into_token()
        })
    }
    pub fn expr(&self) -> Option<Expr> { self.0.children().find_map(Expr::cast) }
}
#[derive(Debug, Clone)]
pub struct ExprBlock(SyntaxNode);
impl AstNode for ExprBlock {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_BLOCK }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprBlock {
    pub fn punct_brace_start_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != PUNCT_BRACE_START {
                return None;
            }
            t.into_token()
        })
    }
    pub fn statements(&self) -> impl Iterator<Item = Stmt> {
        self.0.children().filter_map(Stmt::cast)
    }
    pub fn punct_brace_end_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != PUNCT_BRACE_END {
                return None;
            }
            t.into_token()
        })
    }
}
#[derive(Debug, Clone)]
pub struct ExprUnary(SyntaxNode);
impl AstNode for ExprUnary {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_UNARY }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprUnary {
    pub fn expr(&self) -> Option<Expr> { self.0.children().find_map(Expr::cast) }
}
#[derive(Debug, Clone)]
pub struct ExprBinary(SyntaxNode);
impl AstNode for ExprBinary {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_BINARY }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprBinary {
    pub fn lhs(&self) -> Option<Expr> { self.0.children().find_map(Expr::cast) }
    pub fn rhs(&self) -> Option<Expr> { self.0.children().find_map(Expr::cast) }
}
#[derive(Debug, Clone)]
pub struct ExprParen(SyntaxNode);
impl AstNode for ExprParen {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_PAREN }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprParen {
    pub fn punct_paren_start_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != PUNCT_PAREN_START {
                return None;
            }
            t.into_token()
        })
    }
    pub fn expr(&self) -> Option<Expr> { self.0.children().find_map(Expr::cast) }
    pub fn punct_paren_end_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != PUNCT_PAREN_END {
                return None;
            }
            t.into_token()
        })
    }
}
#[derive(Debug, Clone)]
pub struct ExprArray(SyntaxNode);
impl AstNode for ExprArray {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_ARRAY }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprArray {
    pub fn punct_bracket_start_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != PUNCT_BRACKET_START {
                return None;
            }
            t.into_token()
        })
    }
    pub fn punct_bracket_end_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != PUNCT_BRACKET_END {
                return None;
            }
            t.into_token()
        })
    }
}
#[derive(Debug, Clone)]
pub struct ExprIndex(SyntaxNode);
impl AstNode for ExprIndex {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_INDEX }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprIndex {
    pub fn base(&self) -> Option<Expr> { self.0.children().find_map(Expr::cast) }
    pub fn punct_bracket_start_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != PUNCT_BRACKET_START {
                return None;
            }
            t.into_token()
        })
    }
    pub fn index(&self) -> Option<Expr> { self.0.children().find_map(Expr::cast) }
    pub fn punct_bracket_end_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != PUNCT_BRACKET_END {
                return None;
            }
            t.into_token()
        })
    }
}
#[derive(Debug, Clone)]
pub struct ExprObject(SyntaxNode);
impl AstNode for ExprObject {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_OBJECT }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprObject {
    pub fn punct_map_start_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != PUNCT_MAP_START {
                return None;
            }
            t.into_token()
        })
    }
    pub fn punct_brace_end_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != PUNCT_BRACE_END {
                return None;
            }
            t.into_token()
        })
    }
}
#[derive(Debug, Clone)]
pub struct ExprCall(SyntaxNode);
impl AstNode for ExprCall {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_CALL }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprCall {
    pub fn expr(&self) -> Option<Expr> { self.0.children().find_map(Expr::cast) }
    pub fn arg_list(&self) -> Option<ArgList> { self.0.children().find_map(ArgList::cast) }
}
#[derive(Debug, Clone)]
pub struct ExprClosure(SyntaxNode);
impl AstNode for ExprClosure {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_CLOSURE }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprClosure {
    pub fn param_list(&self) -> Option<ParamList> { self.0.children().find_map(ParamList::cast) }
    pub fn body(&self) -> Option<Expr> { self.0.children().find_map(Expr::cast) }
}
#[derive(Debug, Clone)]
pub struct ExprIf(SyntaxNode);
impl AstNode for ExprIf {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_IF }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprIf {
    pub fn kw_if_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != KW_IF {
                return None;
            }
            t.into_token()
        })
    }
    pub fn expr(&self) -> Option<Expr> { self.0.children().find_map(Expr::cast) }
    pub fn then_branch(&self) -> Option<ExprBlock> { self.0.children().find_map(ExprBlock::cast) }
}
#[derive(Debug, Clone)]
pub struct ExprLoop(SyntaxNode);
impl AstNode for ExprLoop {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_LOOP }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprLoop {
    pub fn kw_loop_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != KW_LOOP {
                return None;
            }
            t.into_token()
        })
    }
    pub fn loop_body(&self) -> Option<ExprBlock> { self.0.children().find_map(ExprBlock::cast) }
}
#[derive(Debug, Clone)]
pub struct ExprFor(SyntaxNode);
impl AstNode for ExprFor {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_FOR }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprFor {
    pub fn kw_for_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != KW_FOR {
                return None;
            }
            t.into_token()
        })
    }
    pub fn pat(&self) -> Option<Pat> { self.0.children().find_map(Pat::cast) }
    pub fn kw_in_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != KW_IN {
                return None;
            }
            t.into_token()
        })
    }
    pub fn iterable(&self) -> Option<Expr> { self.0.children().find_map(Expr::cast) }
    pub fn loop_body(&self) -> Option<ExprBlock> { self.0.children().find_map(ExprBlock::cast) }
}
#[derive(Debug, Clone)]
pub struct ExprWhile(SyntaxNode);
impl AstNode for ExprWhile {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_WHILE }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprWhile {
    pub fn kw_while_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != KW_WHILE {
                return None;
            }
            t.into_token()
        })
    }
    pub fn expr(&self) -> Option<Expr> { self.0.children().find_map(Expr::cast) }
    pub fn loop_body(&self) -> Option<ExprBlock> { self.0.children().find_map(ExprBlock::cast) }
}
#[derive(Debug, Clone)]
pub struct ExprBreak(SyntaxNode);
impl AstNode for ExprBreak {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_BREAK }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprBreak {
    pub fn kw_break_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != KW_BREAK {
                return None;
            }
            t.into_token()
        })
    }
}
#[derive(Debug, Clone)]
pub struct ExprContinue(SyntaxNode);
impl AstNode for ExprContinue {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_CONTINUE }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprContinue {
    pub fn kw_continue_token(&self) -> Option<SyntaxToken> {
        self.0.first_child_or_token().and_then(|e| e.into_token())
    }
}
#[derive(Debug, Clone)]
pub struct ExprSwitch(SyntaxNode);
impl AstNode for ExprSwitch {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_SWITCH }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprSwitch {
    pub fn kw_switch_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != KW_SWITCH {
                return None;
            }
            t.into_token()
        })
    }
    pub fn expr(&self) -> Option<Expr> { self.0.children().find_map(Expr::cast) }
    pub fn switch_arm_list(&self) -> Option<SwitchArmList> {
        self.0.children().find_map(SwitchArmList::cast)
    }
}
#[derive(Debug, Clone)]
pub struct ExprReturn(SyntaxNode);
impl AstNode for ExprReturn {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_RETURN }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprReturn {
    pub fn kw_return_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != KW_RETURN {
                return None;
            }
            t.into_token()
        })
    }
}
#[derive(Debug, Clone)]
pub struct ExprFn(SyntaxNode);
impl AstNode for ExprFn {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_FN }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprFn {
    pub fn kw_fn_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != KW_FN {
                return None;
            }
            t.into_token()
        })
    }
    pub fn ident_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != IDENT {
                return None;
            }
            t.into_token()
        })
    }
    pub fn param_list(&self) -> Option<ParamList> { self.0.children().find_map(ParamList::cast) }
    pub fn body(&self) -> Option<ExprBlock> { self.0.children().find_map(ExprBlock::cast) }
}
#[derive(Debug, Clone)]
pub struct ExprImport(SyntaxNode);
impl AstNode for ExprImport {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_IMPORT }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprImport {
    pub fn kw_import_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != KW_IMPORT {
                return None;
            }
            t.into_token()
        })
    }
    pub fn lit_str_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != LIT_STR {
                return None;
            }
            t.into_token()
        })
    }
}
#[derive(Debug, Clone)]
pub struct ObjectField(SyntaxNode);
impl AstNode for ObjectField {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == OBJECT_FIELD }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ObjectField {
    pub fn punct_colon_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != PUNCT_COLON {
                return None;
            }
            t.into_token()
        })
    }
    pub fn expr(&self) -> Option<Expr> { self.0.children().find_map(Expr::cast) }
}
#[derive(Debug, Clone)]
pub struct ArgList(SyntaxNode);
impl AstNode for ArgList {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == ARG_LIST }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ArgList {
    pub fn punct_paren_start_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != PUNCT_PAREN_START {
                return None;
            }
            t.into_token()
        })
    }
    pub fn punct_paren_end_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != PUNCT_PAREN_END {
                return None;
            }
            t.into_token()
        })
    }
}
#[derive(Debug, Clone)]
pub struct ParamList(SyntaxNode);
impl AstNode for ParamList {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == PARAM_LIST }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
#[derive(Debug, Clone)]
pub struct Param(SyntaxNode);
impl AstNode for Param {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == PARAM }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl Param {
    pub fn ident_token(&self) -> Option<SyntaxToken> {
        self.0.first_child_or_token().and_then(|e| e.into_token())
    }
}
#[derive(Debug, Clone)]
pub enum Pat {
    Tuple(PatTuple),
    Ident(PatIdent),
}
impl AstNode for Pat {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == PAT }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if !Self::can_cast(&syntax) {
            return None;
        }
        let first_child = syntax.first_child()?;
        match first_child.kind() {
            PAT_TUPLE => Some(Self::Tuple(PatTuple::cast(first_child)?)),
            PAT_IDENT => Some(Self::Ident(PatIdent::cast(first_child)?)),
            _ => None,
        }
    }
    fn syntax(&self) -> SyntaxNode {
        match self {
            Self::Tuple(t) => t.syntax().parent().unwrap(),
            Self::Ident(t) => t.syntax().parent().unwrap(),
        }
    }
}
#[derive(Debug, Clone)]
pub struct SwitchArmList(SyntaxNode);
impl AstNode for SwitchArmList {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == SWITCH_ARM_LIST }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl SwitchArmList {
    pub fn punct_brace_start_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != PUNCT_BRACE_START {
                return None;
            }
            t.into_token()
        })
    }
    pub fn punct_brace_end_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != PUNCT_BRACE_END {
                return None;
            }
            t.into_token()
        })
    }
}
#[derive(Debug, Clone)]
pub struct SwitchArm(SyntaxNode);
impl AstNode for SwitchArm {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == SWITCH_ARM }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl SwitchArm {
    pub fn punct_arrow_fat_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != PUNCT_ARROW_FAT {
                return None;
            }
            t.into_token()
        })
    }
    pub fn expr(&self) -> Option<Expr> { self.0.children().find_map(Expr::cast) }
}
#[derive(Debug, Clone)]
pub struct ExprExport(SyntaxNode);
impl AstNode for ExprExport {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPR_EXPORT }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExprExport {
    pub fn kw_export_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != KW_EXPORT {
                return None;
            }
            t.into_token()
        })
    }
    pub fn export_target(&self) -> Option<ExportTarget> {
        self.0.children().find_map(ExportTarget::cast)
    }
}
#[derive(Debug, Clone)]
pub enum ExportTarget {
    ExprLet(ExprLet),
    ExprConst(ExprConst),
    Ident(ExportIdent),
}
impl AstNode for ExportTarget {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPORT_TARGET }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if !Self::can_cast(&syntax) {
            return None;
        }
        let first_child = syntax.first_child()?;
        match first_child.kind() {
            EXPR_LET => Some(Self::ExprLet(ExprLet::cast(first_child)?)),
            EXPR_CONST => Some(Self::ExprConst(ExprConst::cast(first_child)?)),
            EXPORT_IDENT => Some(Self::Ident(ExportIdent::cast(first_child)?)),
            _ => None,
        }
    }
    fn syntax(&self) -> SyntaxNode {
        match self {
            Self::ExprLet(t) => t.syntax().parent().unwrap(),
            Self::ExprConst(t) => t.syntax().parent().unwrap(),
            Self::Ident(t) => t.syntax().parent().unwrap(),
        }
    }
}
#[derive(Debug, Clone)]
pub struct ExportIdent(SyntaxNode);
impl AstNode for ExportIdent {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == EXPORT_IDENT }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl ExportIdent {
    pub fn ident_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != IDENT {
                return None;
            }
            t.into_token()
        })
    }
}
#[derive(Debug, Clone)]
pub struct PatTuple(SyntaxNode);
impl AstNode for PatTuple {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == PAT_TUPLE }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl PatTuple {
    pub fn punct_paren_start_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != PUNCT_PAREN_START {
                return None;
            }
            t.into_token()
        })
    }
    pub fn punct_paren_end_token(&self) -> Option<SyntaxToken> {
        self.0.children_with_tokens().find_map(|t| {
            if t.kind() != PUNCT_PAREN_END {
                return None;
            }
            t.into_token()
        })
    }
}
#[derive(Debug, Clone)]
pub struct PatIdent(SyntaxNode);
impl AstNode for PatIdent {
    #[inline]
    fn can_cast(syntax: &SyntaxNode) -> bool { syntax.kind() == PAT_IDENT }
    #[inline]
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(&syntax) {
            Some(Self(syntax))
        } else {
            None
        }
    }
    fn syntax(&self) -> SyntaxNode { self.0.clone() }
}
impl PatIdent {
    pub fn ident_token(&self) -> Option<SyntaxToken> {
        self.0.first_child_or_token().and_then(|e| e.into_token())
    }
}
