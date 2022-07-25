use super::module::Module;
use crate::{eval::Value, source::SourceInfo, HashSet, Hir, IndexMap, Scope, Type};
use rhai_rowan::{syntax::SyntaxKind, TextRange};
use strum_macros::IntoStaticStr;

slotmap::new_key_type! { pub struct Symbol; }

#[derive(Debug, Clone)]
pub struct SymbolData {
    pub source: SourceInfo,
    pub parent_scope: Scope,
    pub kind: SymbolKind,
    pub export: bool,
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

    #[inline]
    #[must_use]
    pub fn value(&self) -> &Value {
        match &self.kind {
            SymbolKind::Lit(lit) => &lit.value,
            _ => &Value::Unknown,
        }
    }

    #[inline]
    #[must_use]
    pub fn is_param(&self) -> bool {
        match &self.kind {
            SymbolKind::Decl(d) => d.is_param,
            _ => false,
        }
    }

    #[inline]
    #[must_use]
    pub fn target(&self) -> Option<ReferenceTarget> {
        match &self.kind {
            SymbolKind::Reference(r) => r.target,
            SymbolKind::Decl(d) => d.target,
            SymbolKind::Import(i) => i.target.map(ReferenceTarget::Module),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, IntoStaticStr)]
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
    Throw(ThrowSymbol),
    Import(ImportSymbol),
    Discard(DiscardSymbol),
}

impl SymbolKind {
    /// Returns `true` if the symbol kind is [`Block`].
    ///
    /// [`Block`]: SymbolKind::Block
    #[must_use]
    pub fn is_block(&self) -> bool {
        matches!(self, Self::Block(..))
    }

    #[must_use]
    pub fn as_block(&self) -> Option<&BlockSymbol> {
        if let Self::Block(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Fn`].
    ///
    /// [`Fn`]: SymbolKind::Fn
    #[must_use]
    pub fn is_fn(&self) -> bool {
        matches!(self, Self::Fn(..))
    }

    #[must_use]
    pub fn as_fn(&self) -> Option<&FnSymbol> {
        if let Self::Fn(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Op`].
    ///
    /// [`Op`]: SymbolKind::Op
    #[must_use]
    pub fn is_op(&self) -> bool {
        matches!(self, Self::Op(..))
    }

    #[must_use]
    pub fn as_op(&self) -> Option<&OpSymbol> {
        if let Self::Op(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Decl`].
    ///
    /// [`Decl`]: SymbolKind::Decl
    #[must_use]
    pub fn is_decl(&self) -> bool {
        matches!(self, Self::Decl(..))
    }

    #[must_use]
    pub fn as_decl(&self) -> Option<&DeclSymbol> {
        if let Self::Decl(v) = self {
            Some(v)
        } else {
            None
        }
    }

    #[must_use]
    pub fn as_decl_mut(&mut self) -> Option<&mut DeclSymbol> {
        if let Self::Decl(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Reference`].
    ///
    /// [`Reference`]: SymbolKind::Reference
    #[must_use]
    pub fn is_reference(&self) -> bool {
        matches!(self, Self::Reference(..))
    }

    #[must_use]
    pub fn as_reference(&self) -> Option<&ReferenceSymbol> {
        if let Self::Reference(v) = self {
            Some(v)
        } else {
            None
        }
    }

    #[must_use]
    pub fn as_reference_mut(&mut self) -> Option<&mut ReferenceSymbol> {
        if let Self::Reference(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Path`].
    ///
    /// [`Path`]: SymbolKind::Path
    #[must_use]
    pub fn is_path(&self) -> bool {
        matches!(self, Self::Path(..))
    }

    #[must_use]
    pub fn as_path(&self) -> Option<&PathSymbol> {
        if let Self::Path(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Lit`].
    ///
    /// [`Lit`]: SymbolKind::Lit
    #[must_use]
    pub fn is_lit(&self) -> bool {
        matches!(self, Self::Lit(..))
    }

    #[must_use]
    pub fn as_lit(&self) -> Option<&LitSymbol> {
        if let Self::Lit(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Unary`].
    ///
    /// [`Unary`]: SymbolKind::Unary
    #[must_use]
    pub fn is_unary(&self) -> bool {
        matches!(self, Self::Unary(..))
    }

    #[must_use]
    pub fn as_unary(&self) -> Option<&UnarySymbol> {
        if let Self::Unary(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Binary`].
    ///
    /// [`Binary`]: SymbolKind::Binary
    #[must_use]
    pub fn is_binary(&self) -> bool {
        matches!(self, Self::Binary(..))
    }

    #[must_use]
    pub fn as_binary(&self) -> Option<&BinarySymbol> {
        if let Self::Binary(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Array`].
    ///
    /// [`Array`]: SymbolKind::Array
    #[must_use]
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(..))
    }

    #[must_use]
    pub fn as_array(&self) -> Option<&ArraySymbol> {
        if let Self::Array(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Index`].
    ///
    /// [`Index`]: SymbolKind::Index
    #[must_use]
    pub fn is_index(&self) -> bool {
        matches!(self, Self::Index(..))
    }

    #[must_use]
    pub fn as_index(&self) -> Option<&IndexSymbol> {
        if let Self::Index(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Object`].
    ///
    /// [`Object`]: SymbolKind::Object
    #[must_use]
    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(..))
    }

    #[must_use]
    pub fn as_object(&self) -> Option<&ObjectSymbol> {
        if let Self::Object(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Call`].
    ///
    /// [`Call`]: SymbolKind::Call
    #[must_use]
    pub fn is_call(&self) -> bool {
        matches!(self, Self::Call(..))
    }

    #[must_use]
    pub fn as_call(&self) -> Option<&CallSymbol> {
        if let Self::Call(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Closure`].
    ///
    /// [`Closure`]: SymbolKind::Closure
    #[must_use]
    pub fn is_closure(&self) -> bool {
        matches!(self, Self::Closure(..))
    }

    #[must_use]
    pub fn as_closure(&self) -> Option<&ClosureSymbol> {
        if let Self::Closure(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`If`].
    ///
    /// [`If`]: SymbolKind::If
    #[must_use]
    pub fn is_if(&self) -> bool {
        matches!(self, Self::If(..))
    }

    #[must_use]
    pub fn as_if(&self) -> Option<&IfSymbol> {
        if let Self::If(v) = self {
            Some(v)
        } else {
            None
        }
    }

    #[must_use]
    pub fn as_if_mut(&mut self) -> Option<&mut IfSymbol> {
        if let Self::If(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Loop`].
    ///
    /// [`Loop`]: SymbolKind::Loop
    #[must_use]
    pub fn is_loop(&self) -> bool {
        matches!(self, Self::Loop(..))
    }

    #[must_use]
    pub fn as_loop(&self) -> Option<&LoopSymbol> {
        if let Self::Loop(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`For`].
    ///
    /// [`For`]: SymbolKind::For
    #[must_use]
    pub fn is_for(&self) -> bool {
        matches!(self, Self::For(..))
    }

    #[must_use]
    pub fn as_for(&self) -> Option<&ForSymbol> {
        if let Self::For(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`While`].
    ///
    /// [`While`]: SymbolKind::While
    #[must_use]
    pub fn is_while(&self) -> bool {
        matches!(self, Self::While(..))
    }

    #[must_use]
    pub fn as_while(&self) -> Option<&WhileSymbol> {
        if let Self::While(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Break`].
    ///
    /// [`Break`]: SymbolKind::Break
    #[must_use]
    pub fn is_break(&self) -> bool {
        matches!(self, Self::Break(..))
    }

    #[must_use]
    pub fn as_break(&self) -> Option<&BreakSymbol> {
        if let Self::Break(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Continue`].
    ///
    /// [`Continue`]: SymbolKind::Continue
    #[must_use]
    pub fn is_continue(&self) -> bool {
        matches!(self, Self::Continue(..))
    }

    #[must_use]
    pub fn as_continue(&self) -> Option<&ContinueSymbol> {
        if let Self::Continue(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Return`].
    ///
    /// [`Return`]: SymbolKind::Return
    #[must_use]
    pub fn is_return(&self) -> bool {
        matches!(self, Self::Return(..))
    }

    #[must_use]
    pub fn as_return(&self) -> Option<&ReturnSymbol> {
        if let Self::Return(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Switch`].
    ///
    /// [`Switch`]: SymbolKind::Switch
    #[must_use]
    pub fn is_switch(&self) -> bool {
        matches!(self, Self::Switch(..))
    }

    #[must_use]
    pub fn as_switch(&self) -> Option<&SwitchSymbol> {
        if let Self::Switch(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Export`].
    ///
    /// [`Export`]: SymbolKind::Export
    #[must_use]
    pub fn is_export(&self) -> bool {
        matches!(self, Self::Export(..))
    }

    #[must_use]
    pub fn as_export(&self) -> Option<&ExportSymbol> {
        if let Self::Export(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Try`].
    ///
    /// [`Try`]: SymbolKind::Try
    #[must_use]
    pub fn is_try(&self) -> bool {
        matches!(self, Self::Try(..))
    }

    #[must_use]
    pub fn as_try(&self) -> Option<&TrySymbol> {
        if let Self::Try(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Import`].
    ///
    /// [`Import`]: SymbolKind::Import
    #[must_use]
    pub fn is_import(&self) -> bool {
        matches!(self, Self::Import(..))
    }

    #[must_use]
    pub fn as_import(&self) -> Option<&ImportSymbol> {
        if let Self::Import(v) = self {
            Some(v)
        } else {
            None
        }
    }

    #[must_use]
    pub fn as_import_mut(&mut self) -> Option<&mut ImportSymbol> {
        if let Self::Import(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Discard`].
    ///
    /// [`Discard`]: SymbolKind::Discard
    #[must_use]
    pub fn is_discard(&self) -> bool {
        matches!(self, Self::Discard(..))
    }

    #[must_use]
    pub fn as_discard(&self) -> Option<&DiscardSymbol> {
        if let Self::Discard(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the symbol kind is [`Throw`].
    ///
    /// [`Throw`]: SymbolKind::Throw
    #[must_use]
    pub fn is_throw(&self) -> bool {
        matches!(self, Self::Throw(..))
    }

    #[must_use]
    pub fn as_throw(&self) -> Option<&ThrowSymbol> {
        if let Self::Throw(v) = self {
            Some(v)
        } else {
            None
        }
    }
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
    pub getter: bool,
    pub setter: bool,
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
    /// Normally declarations are not references,
    /// however in some cases they can delegate the resolution
    /// to a target, e.g. in case of module aliases.
    pub target: Option<ReferenceTarget>,
}

#[derive(Debug, Default, Clone)]
pub struct ReferenceSymbol {
    pub target: Option<ReferenceTarget>,
    pub part_of_path: bool,
    pub field_access: bool,
    pub name: String,
}

#[derive(Debug, Default, Clone)]
pub struct PathSymbol {
    pub scope: Scope,
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

#[derive(Debug, Clone)]
pub struct ThrowSymbol {
    pub expr: Option<Symbol>,
}

#[derive(Debug, Default, Clone)]
pub struct ClosureSymbol {
    pub scope: Scope,
    pub expr: Option<Symbol>,
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
    pub arms: Vec<SwitchArm>,
}

#[derive(Debug, Default, Clone)]
pub struct SwitchArm {
    pub pat_expr: Option<Symbol>,
    pub condition_expr: Option<Symbol>,
    pub value_expr: Option<Symbol>,
}

#[derive(Debug, Default, Clone)]
pub struct ExportSymbol {
    pub target: Option<Symbol>,
}

#[derive(Debug, Default, Clone)]
pub struct ImportSymbol {
    pub scope: Scope,
    pub expr: Option<Symbol>,
    pub alias: Option<Symbol>,
    pub target: Option<Module>,
}

impl ImportSymbol {
    #[must_use]
    pub fn import_path<'h>(&'h self, hir: &'h Hir) -> Option<&'h str> {
        self.expr
            .and_then(|e| hir[e].value().as_string().map(String::as_str))
    }
}

#[derive(Debug, Default, Clone)]
pub struct TrySymbol {
    pub try_scope: Scope,
    pub catch_scope: Scope,
}

#[derive(Debug, Default, Clone)]
pub struct DiscardSymbol {}

#[derive(Debug, Clone, Copy)]
pub enum ReferenceTarget {
    Symbol(Symbol),
    Module(Module),
}
