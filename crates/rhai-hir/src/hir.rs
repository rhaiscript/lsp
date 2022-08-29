mod add;
mod errors;
mod query;
mod remove;
mod resolve;

use core::ops;
use std::sync::Arc;

use crate::{
    module::{ModuleData, ModuleResolver, DefaultModuleResolver},
    scope::ScopeData,
    source::{Source, SourceData},
    symbol::*,
    ty::{Type, TypeData},
    Module, Scope,
};

use rhai_rowan::syntax::SyntaxNode;
use slotmap::{Key, SlotMap};
use url::Url;

#[derive(Clone)]
pub struct Hir {
    pub(crate) static_module: Module,
    pub(crate) virtual_source: Source,
    pub(crate) modules: SlotMap<Module, ModuleData>,
    pub(crate) scopes: SlotMap<Scope, ScopeData>,
    pub(crate) symbols: SlotMap<Symbol, SymbolData>,
    pub(crate) sources: SlotMap<Source, SourceData>,
    pub(crate) types: SlotMap<Type, TypeData>,
    pub(crate) builtin_types: BuiltinTypes,
    pub(crate) module_resolver: Arc<dyn ModuleResolver>
}

impl Default for Hir {
    fn default() -> Self {
        let mut this = Self {
            static_module: Default::default(),
            virtual_source: Default::default(),
            modules: Default::default(),
            scopes: Default::default(),
            symbols: Default::default(),
            sources: Default::default(),
            types: Default::default(),
            builtin_types: BuiltinTypes::uninit(),
            module_resolver: Arc::new(DefaultModuleResolver)
        };
        this.prepare();
        this
    }
}

static_assertions::assert_impl_all!(Hir: Send, Sync);

impl Hir {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_import_resolver(&mut self, resolver: impl ModuleResolver + 'static) {
        self.module_resolver = Arc::new(resolver);
    }
}

impl Hir {
    pub fn clear(&mut self) {
        self.symbols.clear();
        self.scopes.clear();
        self.modules.clear();
        self.sources.clear();
        self.types.clear();
        self.builtin_types = BuiltinTypes::uninit();
        self.static_module = Module::null();
        self.prepare();
    }

    #[must_use]
    #[inline]
    pub fn symbol(&self, symbol: Symbol) -> Option<&SymbolData> {
        self.symbols.get(symbol)
    }

    #[inline]
    pub fn symbols(&self) -> impl Iterator<Item = (Symbol, &SymbolData)> {
        self.symbols.iter()
    }

    #[must_use]
    #[inline]
    pub fn scope(&self, scope: Scope) -> Option<&ScopeData> {
        self.scopes.get(scope)
    }

    #[inline]
    pub fn scopes(&self) -> impl Iterator<Item = (Scope, &ScopeData)> {
        self.scopes.iter()
    }

    #[must_use]
    #[inline]
    pub const fn static_module(&self) -> Module {
        self.static_module
    }

    #[must_use]
    #[inline]
    pub fn module(&self, module: Module) -> Option<&ModuleData> {
        self.modules.get(module)
    }

    #[inline]
    pub fn modules(&self) -> impl Iterator<Item = (Module, &ModuleData)> {
        self.modules.iter()
    }

    #[inline]
    pub fn sources(&self) -> impl Iterator<Item = (Source, &SourceData)> {
        self.sources.iter()
    }

    #[must_use]
    pub fn source_of(&self, url: &Url) -> Option<Source> {
        self.sources()
            .find_map(|(s, data)| if data.url == *url { Some(s) } else { None })
    }

    #[inline]
    fn symbol_mut(&mut self, symbol: Symbol) -> &mut SymbolData {
        self.symbols.get_mut(symbol).unwrap()
    }

    #[inline]
    fn scope_mut(&mut self, scope: Scope) -> &mut ScopeData {
        self.scopes.get_mut(scope).unwrap()
    }

    #[inline]
    fn source_mut(&mut self, source: Source) -> &mut SourceData {
        self.sources.get_mut(source).unwrap()
    }

    #[inline]
    fn module_mut(&mut self, module: Module) -> &mut ModuleData {
        self.modules.get_mut(module).unwrap()
    }

    fn prepare(&mut self) {
        self.ensure_static_module();
        self.ensure_virtual_source();
        self.ensure_builtin_types();
    }
}

impl ops::Index<Scope> for Hir {
    type Output = ScopeData;

    fn index(&self, index: Scope) -> &Self::Output {
        assert!(!index.is_null(), "expected non-null scope");
        match self.scopes.get(index) {
            Some(s) => s,
            None => panic!("scope was not found {:?}", index),
        }
    }
}

impl ops::Index<Symbol> for Hir {
    type Output = SymbolData;

    fn index(&self, index: Symbol) -> &Self::Output {
        assert!(!index.is_null(), "expected non-null symbol");
        let sym = match self.symbols.get(index) {
            Some(s) => s,
            None => panic!("symbol was not found {:?}", index),
        };

        if let SymbolKind::Virtual(VirtualSymbol::Proxy(proxy)) = &sym.kind {
            return match self.symbols.get(proxy.target) {
                Some(s) => s,
                None => panic!("proxy target symbol was not found {:?}", proxy.target),
            };
        }

        sym
    }
}

impl ops::Index<Module> for Hir {
    type Output = ModuleData;

    fn index(&self, index: Module) -> &Self::Output {
        assert!(!index.is_null(), "expected non-null module");
        match self.modules.get(index) {
            Some(s) => s,
            None => panic!("module was not found {:?}", index),
        }
    }
}

impl ops::Index<Source> for Hir {
    type Output = SourceData;

    fn index(&self, index: Source) -> &Self::Output {
        assert!(!index.is_null(), "expected non-null source");
        match self.sources.get(index) {
            Some(s) => s,
            None => panic!("source was not found {:?}", index),
        }
    }
}

impl ops::Index<Type> for Hir {
    type Output = TypeData;

    fn index(&self, index: Type) -> &Self::Output {
        assert!(!index.is_null(), "expected non-null type");
        match self.types.get(index) {
            Some(s) => s,
            None => panic!("type was not found {:?}", index),
        }
    }
}

/// Built-in (primitive) types are treated as any other type
/// but always exist in the HIR and cannot be removed.
///
/// This struct keeps track of their keys.
#[derive(Debug, Clone, Copy)]
pub struct BuiltinTypes {
    pub module: Type,
    pub int: Type,
    pub float: Type,
    pub bool: Type,
    pub char: Type,
    pub string: Type,
    pub timestamp: Type,
    pub void: Type,
    pub unknown: Type,
    pub never: Type,
}

impl BuiltinTypes {
    fn uninit() -> Self {
        Self {
            module: Default::default(),
            int: Default::default(),
            float: Default::default(),
            bool: Default::default(),
            char: Default::default(),
            string: Default::default(),
            timestamp: Default::default(),
            void: Default::default(),
            unknown: Default::default(),
            never: Default::default(),
        }
    }

    #[must_use]
    fn is_uninit(&self) -> bool {
        // We don't check all of the fields,
        // as this is not exposed and we always
        // initialize all of them.
        self.module.is_null()
    }
}
