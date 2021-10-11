use crate::{eval::Value, syntax::SyntaxInfo, HashSet, IndexMap, IndexSet, Scope, Type};
use enum_as_inner::EnumAsInner;
use rhai_rowan::{syntax::SyntaxKind, TextRange};
use serde::{Deserialize, Serialize};

slotmap::new_key_type! { pub struct Symbol; }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolData {
    pub syntax: Option<SyntaxInfo>,
    pub selection_syntax: Option<SyntaxInfo>,
    pub parent_scope: Scope,
    pub kind: SymbolKind,
}

impl SymbolData {
    pub fn name(&self) -> Option<&str> {
        match &self.kind {
            SymbolKind::Fn(f) => Some(&f.name),
            SymbolKind::Decl(d) => Some(&d.name),
            SymbolKind::Reference(r) => Some(&r.name),
            _ => None,
        }
    }

    pub fn docs(&self) -> Option<&str> {
        match &self.kind {
            SymbolKind::Fn(f) => Some(&f.docs),
            SymbolKind::Decl(d) => Some(&d.docs),
            _ => None,
        }
    }

    /// Whether the given range is the identifier of the symbol.
    pub fn has_selection_range(&self, range: TextRange) -> bool {
        self.selection_syntax
            .and_then(|s| s.text_range.map(|r| r == range))
            .unwrap_or(false)
    }

    #[inline]
    pub fn text_range(&self) -> Option<TextRange> {
        self.syntax.and_then(|s| s.text_range)
    }

    #[inline]
    pub fn selection_range(&self) -> Option<TextRange> {
        self.syntax.and_then(|s| s.text_range)
    }

    #[inline]
    pub fn selection_or_text_range(&self) -> Option<TextRange> {
        self.selection_range().or_else(|| self.text_range())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumAsInner)]
pub enum SymbolKind {
    Block(BlockSymbol),
    Fn(FnSymbol),
    Decl(DeclSymbol),
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
    // Switch(SwitchSymbol),
    // Import(ImportSymbol),
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
    pub references: HashSet<Symbol>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DeclSymbol {
    pub name: String,
    pub docs: String,
    pub is_param: bool,
    pub is_const: bool,
    pub ty: Type,
    pub inferred_ty: Option<Type>,
    pub value: Option<Scope>,
    pub references: HashSet<Symbol>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReferenceSymbol {
    pub target: Option<ReferenceTarget>,
    pub name: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PathSymbol {
    pub segments: IndexSet<Symbol>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LitSymbol {
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
    pub values: IndexSet<Symbol>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSymbol {
    pub base: Option<Symbol>,
    pub index: Option<Symbol>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallSymbol {
    pub lhs: Option<Symbol>,
    pub arguments: IndexSet<Symbol>,
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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IfSymbol {
    pub condition: Option<Symbol>,
    pub then_scope: Scope,
    pub branches: IndexSet<(Option<Symbol>, Scope)>,
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

#[derive(Debug, Clone, Serialize, Deserialize, EnumAsInner)]
pub enum ReferenceTarget {
    Symbol(Symbol),
    Module(String),
}
