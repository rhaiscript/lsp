use crate::{eval::Value, syntax::SyntaxInfo, HashSet, IndexMap, Module, Scope, Type};
use enum_as_inner::EnumAsInner;
use rhai_rowan::{syntax::SyntaxKind, TextRange};
use serde::{Deserialize, Serialize};
use strum_macros::IntoStaticStr;

slotmap::new_key_type! { pub struct Symbol; }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolData {
    pub syntax: Option<SyntaxInfo>,
    pub selection_syntax: Option<SyntaxInfo>,
    pub parent_scope: Scope,
    pub kind: SymbolKind,
}

impl SymbolData {
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        match &self.kind {
            SymbolKind::Fn(f) => Some(&f.name),
            SymbolKind::Decl(d) => Some(&d.name),
            SymbolKind::Reference(r) => Some(&r.name),
            _ => None,
        }
    }

    #[must_use]
    pub fn docs(&self) -> Option<&str> {
        match &self.kind {
            SymbolKind::Fn(f) => Some(&f.docs),
            SymbolKind::Decl(d) => Some(&d.docs),
            _ => None,
        }
    }

    /// Whether the given range is the identifier of the symbol.
    #[must_use]
    pub fn has_selection_range(&self, range: TextRange) -> bool {
        self.selection_syntax
            .and_then(|s| s.text_range.map(|r| r == range))
            .unwrap_or(false)
    }

    #[inline]
    #[must_use]
    pub fn text_range(&self) -> Option<TextRange> {
        self.syntax.and_then(|s| s.text_range)
    }

    #[inline]
    #[must_use]
    pub fn selection_range(&self) -> Option<TextRange> {
        self.selection_syntax.and_then(|s| s.text_range)
    }

    #[inline]
    #[must_use]
    pub fn selection_or_text_range(&self) -> Option<TextRange> {
        self.selection_range().or_else(|| self.text_range())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumAsInner, IntoStaticStr)]
pub enum SymbolKind {
    Block(BlockSymbol),
    Fn(FnSymbol),
    Decl(Box<DeclSymbol>),
    Reference(ReferenceSymbol),
    Path(PathSymbol),
    Lit(LitSymbol),
    Unary(UnarySymbol),
    Binary(BinarySymbol),
    Array(ArraySymbol),
    Index(IndexSymbol),
    Object(ObjectSymbol),
    Call(CallSymbol),
    Closure(ClosureSymbol),
    If(IfSymbol),
    Loop(LoopSymbol),
    For(ForSymbol),
    While(WhileSymbol),
    Break(BreakSymbol),
    Continue(ContinueSymbol),
    Return(ReturnSymbol),
    Switch(SwitchSymbol),
    Export(ExportSymbol),
    Try(TrySymbol),
    Import(ImportSymbol),
    Discard(DiscardSymbol),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockSymbol {
    pub scope: Scope,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FnSymbol {
    pub name: String,
    pub docs: String,
    pub scope: Scope,
    pub ty: Type,
    pub references: HashSet<Symbol>,
}

impl FnSymbol {
    pub fn params<'s>(&'s self, m: &'s Module) -> impl Iterator<Item = Symbol> + 's {
        m[self.scope]
            .symbols
            .iter()
            .take_while(|s| match &m[**s].kind {
                SymbolKind::Decl(d) => d.is_param,
                _ => false,
            })
            .copied()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DeclSymbol {
    pub name: String,
    pub docs: String,
    pub is_param: bool,
    pub is_const: bool,
    pub is_pat: bool,
    pub ty: Type,
    pub value: Option<Scope>,
    pub references: HashSet<Symbol>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReferenceSymbol {
    pub target: Option<ReferenceTarget>,
    pub part_of_path: bool,
    pub name: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PathSymbol {
    pub segments: Vec<Symbol>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LitSymbol {
    pub ty: Type,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnarySymbol {
    pub op: Option<SyntaxKind>,
    pub rhs: Option<Symbol>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinarySymbol {
    pub lhs: Option<Symbol>,
    pub op: Option<SyntaxKind>,
    pub rhs: Option<Symbol>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArraySymbol {
    pub values: Vec<Symbol>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSymbol {
    pub base: Option<Symbol>,
    pub index: Option<Symbol>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallSymbol {
    pub lhs: Option<Symbol>,
    pub arguments: Vec<Symbol>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectSymbol {
    pub fields: IndexMap<String, ObjectField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectField {
    pub property_syntax: Option<SyntaxInfo>,
    pub property_name: String,
    pub field_syntax: Option<SyntaxInfo>,
    pub value: Option<Symbol>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ClosureSymbol {
    pub scope: Scope,
    pub expr: Option<Symbol>,
}

impl ClosureSymbol {
    pub fn params<'s>(&'s self, m: &'s Module) -> impl Iterator<Item = Symbol> + 's {
        m[self.scope]
            .symbols
            .iter()
            .take_while(|s| match &m[**s].kind {
                SymbolKind::Decl(d) => d.is_param,
                _ => false,
            })
            .copied()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IfSymbol {
    /// Conditions and scopes for each branch.
    pub branches: Vec<(Option<Symbol>, Scope)>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LoopSymbol {
    pub scope: Scope,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ForSymbol {
    pub iterable: Option<Symbol>,
    pub scope: Scope,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WhileSymbol {
    pub condition: Option<Symbol>,
    pub scope: Scope,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BreakSymbol {
    pub expr: Option<Symbol>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ContinueSymbol {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReturnSymbol {
    pub expr: Option<Symbol>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SwitchSymbol {
    pub target: Option<Symbol>,
    pub arms: Vec<(Option<Symbol>, Option<Symbol>)>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ExportSymbol {
    pub target: Option<Symbol>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ImportSymbol {
    pub expr: Option<Symbol>,
    pub alias: Option<Symbol>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TrySymbol {
    pub try_scope: Scope,
    pub catch_scope: Scope,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DiscardSymbol {}

#[derive(Debug, Clone, Serialize, Deserialize, EnumAsInner)]
pub enum ReferenceTarget {
    Symbol(Symbol),
    Module(String),
}
