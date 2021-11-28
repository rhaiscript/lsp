mod def;
mod script;

use rhai_rowan::{
    ast::ExportTarget,
    syntax::{SyntaxKind, SyntaxToken},
};

use crate::{eval::Value, Type};

use super::*;

impl Module {
    pub(crate) fn add_source(&mut self, new_source: SourceData, syntax: &SyntaxNode) {
        if let Some(source) = self.sources.iter().find_map(|(src, source_data)| {
            if source_data.path == new_source.path {
                Some(src)
            } else {
                None
            }
        }) {
            self.remove_source(source);
        }

        let source = self.sources.insert(new_source);

        self.ensure_root_scope();
        match new_source.kind {
            crate::source::SourceKind::Script => self.add_source_script(source, syntax),
            crate::source::SourceKind::Def => self.add_source_def(source, syntax),
        }
    }

    #[allow(clippy::map_unwrap_or)]
    fn ensure_root_scope(&mut self) {
        self.root_scope = self
            .scopes
            .iter()
            .next()
            .map(|(scope, _)| scope)
            .unwrap_or_else(|| {
                let scope = self.create_scope(None, None);
                self.root_scope = scope;
                scope
            });
    }

    #[tracing::instrument(skip(self), level = "trace")]
    fn create_scope(&mut self, parent_symbol: Option<Symbol>, syntax: Option<SourceInfo>) -> Scope {
        let data = ScopeData {
            syntax,
            parent_symbol,
            ..ScopeData::default()
        };
        self.scopes.insert(data)
    }

    fn add_to_scope(&mut self, scope: Scope, symbol: Symbol, hoist: bool) {
        let s = self.scope_unchecked_mut(scope);
        debug_assert!(!s.symbols.contains(&symbol));
        debug_assert!(!s.hoisted_symbols.contains(&symbol));

        if hoist {
            s.hoisted_symbols.insert(symbol);
        } else {
            s.symbols.insert(symbol);
        }

        let sym_data = self.symbol_unchecked_mut(symbol);

        debug_assert!(sym_data.parent_scope == Scope::default());

        sym_data.parent_scope = scope;

        tracing::debug!(
            symbol_kind = Into::<&'static str>::into(&sym_data.kind),
            hoist,
            ?scope,
            ?symbol,
            "added symbol to scope"
        );
    }

    fn set_as_parent_symbol(&mut self, symbol: Symbol, scope: Scope) {
        let s = self.scope_unchecked_mut(scope);
        debug_assert!(s.parent_symbol.is_none());
        s.parent_symbol = Some(symbol);

        tracing::debug!(
            symbol_kind = Into::<&'static str>::into(&self.symbol_unchecked(symbol).kind),
            ?scope,
            ?symbol,
            "set parent symbol of scope"
        );
    }
}
