mod def;
mod script;

use super::*;
use crate::{module::ModuleKind, scope::ScopeParent, source::SourceKind, Type};
use rhai_rowan::ast::{AstNode, Rhai, RhaiDef};

impl Hir {
    pub fn add_source(&mut self, url: &Url, syntax: &SyntaxNode) {
        if let Some(s) = self.source_of(url) {
            self.remove_source(s);
        }

        if let Some(def) = Rhai::cast(syntax.clone()) {
            let source = self.sources.insert(SourceData {
                kind: SourceKind::Def,
                url: url.clone(),
                module: Module::default(),
            });

            let module = self.ensure_module(ModuleKind::Dynamic(url.clone()));
            self.source_mut(source).module = module;

            self.add_script(source, self[module].scope, &def);
        }

        if let Some(def) = RhaiDef::cast(syntax.clone()) {
            let source = self.sources.insert(SourceData {
                kind: SourceKind::Def,
                url: url.clone(),
                module: Module::default(),
            });

            // Here we don't know the module and the scope until
            // we parse the actual definition file.

            self.add_def(source, &def);
        }

        // TODO: error
    }
}

impl Hir {
    fn ensure_static_module(&mut self) {
        if self.static_module.is_null() {
            let scope = self.scopes.insert(ScopeData::default());
            self.static_module = self.modules.insert(ModuleData {
                kind: ModuleKind::Static,
                scope,
            });
        }
    }

    fn ensure_module(&mut self, kind: ModuleKind) -> Module {
        self.ensure_static_module();
        match &kind {
            ModuleKind::Static => self.static_module,
            _ => self
                .modules
                .iter()
                .find_map(|(m, data)| if data.kind == kind { Some(m) } else { None })
                .unwrap_or_else(|| {
                    let scope = self.scopes.insert(ScopeData {
                        parent: Some(ScopeParent::Scope(self[self.static_module].scope)),
                        ..ScopeData::default()
                    });
                    self.modules.insert(ModuleData { kind, scope })
                }),
        }
    }
}

impl Scope {
    pub(crate) fn add_symbol(self, hir: &mut Hir, symbol: Symbol, hoist: bool) {
        let s = hir.scope_mut(self);
        debug_assert!(!s.symbols.contains(&symbol));
        debug_assert!(!s.hoisted_symbols.contains(&symbol));

        if hoist {
            s.hoisted_symbols.insert(symbol);
        } else {
            s.symbols.insert(symbol);
        }

        let sym_data = hir.symbol_mut(symbol);

        debug_assert!(sym_data.parent_scope == Scope::default());

        sym_data.parent_scope = self;

        tracing::debug!(
            symbol_kind = Into::<&'static str>::into(&sym_data.kind),
            hoist,
            ?self,
            ?symbol,
            "added symbol to scope"
        );
    }

    pub(crate) fn set_parent(self, hir: &mut Hir, parent: impl Into<ScopeParent>) {
        let s = hir.scope_mut(self);
        debug_assert!(s.parent.is_none());
        s.parent = Some(parent.into());
    }
}
