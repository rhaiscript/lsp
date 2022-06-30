mod add;
mod query;
mod remove;
mod resolve;
mod errors;

use core::ops;

use crate::{
    module::ModuleData,
    scope::ScopeData,
    source::{Source, SourceData},
    symbol::*,
    Module, Scope,
};

use rhai_rowan::syntax::SyntaxNode;
use slotmap::{Key, SlotMap};
use url::Url;

#[derive(Debug, Clone)]
pub struct Hir {
    static_module: Module,
    virtual_source: Source,
    modules: SlotMap<Module, ModuleData>,
    scopes: SlotMap<Scope, ScopeData>,
    symbols: SlotMap<Symbol, SymbolData>,
    sources: SlotMap<Source, SourceData>,
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
}

impl Hir {
    pub fn clear(&mut self) {
        self.symbols.clear();
        self.scopes.clear();
        self.modules.clear();
        self.sources.clear();
        self.static_module = Module::null();
        self.prepare();
    }

    #[must_use]
    pub fn symbol(&self, symbol: Symbol) -> Option<&SymbolData> {
        self.symbols.get(symbol)
    }

    pub fn symbols(&self) -> impl Iterator<Item = (Symbol, &SymbolData)> {
        self.symbols.iter()
    }

    #[must_use]
    pub fn scope(&self, scope: Scope) -> Option<&ScopeData> {
        self.scopes.get(scope)
    }

    pub fn scopes(&self) -> impl Iterator<Item = (Scope, &ScopeData)> {
        self.scopes.iter()
    }

    #[must_use]
    pub const fn static_module(&self) -> Module {
        self.static_module
    }

    #[must_use]
    pub fn module(&self, module: Module) -> Option<&ModuleData> {
        self.modules.get(module)
    }

    pub fn modules(&self) -> impl Iterator<Item = (Module, &ModuleData)> {
        self.modules.iter()
    }

    pub fn sources(&self) -> impl Iterator<Item = (Source, &SourceData)> {
        self.sources.iter()
    }

    #[must_use]
    pub fn source_of(&self, url: &Url) -> Option<Source> {
        self.sources()
            .find_map(|(s, data)| if data.url == *url { Some(s) } else { None })
    }

    fn symbol_mut(&mut self, symbol: Symbol) -> &mut SymbolData {
        self.symbols.get_mut(symbol).unwrap()
    }

    fn scope_mut(&mut self, scope: Scope) -> &mut ScopeData {
        self.scopes.get_mut(scope).unwrap()
    }

    fn source_mut(&mut self, source: Source) -> &mut SourceData {
        self.sources.get_mut(source).unwrap()
    }

    #[allow(dead_code)]
    fn module_mut(&mut self, module: Module) -> &mut ModuleData {
        self.modules.get_mut(module).unwrap()
    }

    fn prepare(&mut self) {
        self.ensure_static_module();
        self.ensure_virtual_source();
    }
}

impl ops::Index<Scope> for Hir {
    type Output = ScopeData;

    fn index(&self, index: Scope) -> &Self::Output {
        self.scopes.get(index).unwrap()
    }
}

impl ops::Index<Symbol> for Hir {
    type Output = SymbolData;

    fn index(&self, index: Symbol) -> &Self::Output {
        self.symbols.get(index).unwrap()
    }
}

impl ops::Index<Module> for Hir {
    type Output = ModuleData;

    fn index(&self, index: Module) -> &Self::Output {
        self.modules.get(index).unwrap()
    }
}

impl ops::Index<Source> for Hir {
    type Output = SourceData;

    fn index(&self, index: Source) -> &Self::Output {
        self.sources.get(index).unwrap()
    }
}
