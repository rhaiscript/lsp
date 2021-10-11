use crate::{scope::ScopeData, symbol::*, HashMap, Module, Scope};

use core::ops;
use rhai_rowan::syntax::SyntaxNode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Hir {
    modules: HashMap<String, Module>,
}

static_assertions::assert_impl_all!(Hir: Send, Sync);

impl ops::Index<Scope> for Hir {
    type Output = ScopeData;

    fn index(&self, index: Scope) -> &Self::Output {
        for (_, m) in self.modules() {
            if m.contains_scope(index) {
                return &m[index];
            }
        }

        panic!(
            r#"scope "{:?}" does not exist in any of the modules"#,
            index
        )
    }
}

impl ops::Index<Symbol> for Hir {
    type Output = SymbolData;

    fn index(&self, index: Symbol) -> &Self::Output {
        for (_, m) in self.modules() {
            if m.contains_symbol(index) {
                return &m[index];
            }
        }

        panic!(
            r#"symbol "{:?}" does not exist in any of the modules"#,
            index
        )
    }
}

impl Hir {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_module(&self, name: &str) -> Option<&Module> {
        self.modules.get(name)
    }

    pub fn modules(&self) -> impl Iterator<Item = (&String, &Module)> {
        self.modules.iter()
    }

    pub fn remove_module(&mut self, name: &str) {
        self.modules.remove(name);
        // TODO: module references
    }

    pub fn contains_module(&self, module: &str) -> bool {
        self.modules.contains_key(module)
    }

    pub fn module_count(&self) -> usize {
        self.modules.len()
    }

    pub fn contains_scope(&self, scope: Scope) -> bool {
        for (_, m) in self.modules() {
            if m.contains_scope(scope) {
                return true;
            }
        }

        false
    }

    pub fn scope_count(&self) -> usize {
        self.modules()
            .fold(0, |count, (_, m)| count + m.scope_count())
    }

    pub fn contains_symbol(&self, symbol: Symbol) -> bool {
        for (_, m) in self.modules() {
            if m.contains_symbol(symbol) {
                return true;
            }
        }

        false
    }

    pub fn symbol_count(&self) -> usize {
        self.modules()
            .fold(0, |count, (_, m)| count + m.symbol_count())
    }
}

impl Hir {
    pub fn add_module_from_syntax(&mut self, name: &str, syntax: &SyntaxNode) {
        self.remove_module(name);

        if let Some(m) = Module::new_from_syntax(name, syntax) {
            self.modules.insert(name.into(), m);
        }
    }

    pub fn resolve_references(&mut self) {
        for (_, m) in self.modules.iter_mut() {
            m.resolve_references();
        }
    }

    pub fn resolve_references_in_module(&mut self, name: &str) {
        if let Some(m) = self.modules.get_mut(name) {
            m.resolve_references();
        }
    }
}
