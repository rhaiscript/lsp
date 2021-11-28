use super::module::Module;
use crate::{eval::Value, source::SourceInfo, HashSet, IndexMap, Scope, Type};
use enum_as_inner::EnumAsInner;
use rhai_rowan::{syntax::SyntaxKind, TextRange};
use strum_macros::IntoStaticStr;

slotmap::new_key_type! { pub struct Symbol; }

#[derive(Debug, Clone)]
pub struct SymbolData {
    pub source: SourceInfo,
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
        self.source
            .selection_text_range
            .map_or(false, |r| r == range)
    }

    #[inline]
    #[must_use]
    pub fn text_range(&self) -> Option<TextRange> {
        self.source.text_range
    }

    #[inline]
    #[must_use]
    pub fn selection_range(&self) -> Option<TextRange> {
        self.source.selection_text_range
    }

    #[inline]
    #[must_use]
    pub fn selection_or_text_range(&self) -> Option<TextRange> {
        self.selection_range().or_else(|| self.text_range())
    }
}

#[derive(Debug, Clone, EnumAsInner, IntoStaticStr)]
pub enum SymbolKind {
    Block(BlockSymbol),
    Fn(FnSymbol),
    Op(OpSymbol),
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

#[derive(Debug, Clone)]
pub struct BlockSymbol {
    pub scope: Scope,
}

#[derive(Debug, Default, Clone)]
pub struct FnSymbol {
    pub name: String,
    pub docs: String,
    pub scope: Scope,
    pub ty: Type,
    pub references: HashSet<Symbol>,
}

#[derive(Debug, Default, Clone)]
pub struct OpSymbol {
    pub name: String,
    pub docs: String,
    pub lhs_ty: Type,
    pub rhs_ty: Type,
    pub references: HashSet<Symbol>,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Default, Clone)]
pub struct DeclSymbol {
    pub name: String,
    pub docs: String,
    pub is_param: bool,
    pub is_const: bool,
    pub is_pat: bool,
    pub is_import: bool,
    pub ty: Type,
    pub value: Option<Symbol>,
    pub value_scope: Option<Scope>,
    pub references: HashSet<Symbol>,
}

#[derive(Debug, Default, Clone)]
pub struct ReferenceSymbol {
    pub target: Option<ReferenceTarget>,
    pub part_of_path: bool,
    pub name: String,
}

#[derive(Debug, Default, Clone)]
pub struct PathSymbol {
    pub segments: Vec<Symbol>,
}

#[derive(Debug, Default, Clone)]
pub struct LitSymbol {
    pub ty: Type,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub struct UnarySymbol {
    pub op: Option<SyntaxKind>,
    pub rhs: Option<Symbol>,
}

#[derive(Debug, Clone)]
pub struct BinarySymbol {
    pub lhs: Option<Symbol>,
    pub op: Option<SyntaxKind>,
    pub rhs: Option<Symbol>,
}

#[derive(Debug, Clone)]
pub struct ArraySymbol {
    pub values: Vec<Symbol>,
}

#[derive(Debug, Clone)]
pub struct IndexSymbol {
    pub base: Option<Symbol>,
    pub index: Option<Symbol>,
}

#[derive(Debug, Clone)]
pub struct CallSymbol {
    pub lhs: Option<Symbol>,
    pub arguments: Vec<Symbol>,
}

#[derive(Debug, Clone)]
pub struct ObjectSymbol {
    pub fields: IndexMap<String, ObjectField>,
}

#[derive(Debug, Clone)]
pub struct ObjectField {
    pub property_syntax: SourceInfo,
    pub property_name: String,
    pub field_syntax: SourceInfo,
    pub value: Option<Symbol>,
}

#[derive(Debug, Default, Clone)]
pub struct ClosureSymbol {
    pub scope: Scope,
    pub expr: Option<Symbol>,
}

impl ClosureSymbol {
    // pub fn params<'s>(&'s self, m: &'s Module) -> impl Iterator<Item = Symbol> + 's {
    //     m[self.scope]
    //         .symbols
    //         .iter()
    //         .take_while(|s| match &m[**s].kind {
    //             SymbolKind::Decl(d) => d.is_param,
    //             _ => false,
    //         })
    //         .copied()
    // }
}

#[derive(Debug, Default, Clone)]
pub struct IfSymbol {
    /// Conditions and scopes for each branch.
    pub branches: Vec<(Option<Symbol>, Scope)>,
}

#[derive(Debug, Default, Clone)]
pub struct LoopSymbol {
    pub scope: Scope,
}

#[derive(Debug, Default, Clone)]
pub struct ForSymbol {
    pub iterable: Option<Symbol>,
    pub scope: Scope,
}

#[derive(Debug, Default, Clone)]
pub struct WhileSymbol {
    pub condition: Option<Symbol>,
    pub scope: Scope,
}

#[derive(Debug, Default, Clone)]
pub struct BreakSymbol {
    pub expr: Option<Symbol>,
}

#[derive(Debug, Default, Clone)]
pub struct ContinueSymbol {}

#[derive(Debug, Default, Clone)]
pub struct ReturnSymbol {
    pub expr: Option<Symbol>,
}

#[derive(Debug, Default, Clone)]
pub struct SwitchSymbol {
    pub target: Option<Symbol>,
    pub arms: Vec<(Option<Symbol>, Option<Symbol>)>,
}

#[derive(Debug, Default, Clone)]
pub struct ExportSymbol {
    pub target: Option<Symbol>,
}

#[derive(Debug, Default, Clone)]
pub struct ImportSymbol {
    pub expr: Option<Symbol>,
    pub alias: Option<Symbol>,
}

#[derive(Debug, Default, Clone)]
pub struct TrySymbol {
    pub try_scope: Scope,
    pub catch_scope: Scope,
}

#[derive(Debug, Default, Clone)]
pub struct DiscardSymbol {}

#[derive(Debug, Clone, EnumAsInner)]
pub enum ReferenceTarget {
    Symbol(Symbol),
    Module(Module),
}
