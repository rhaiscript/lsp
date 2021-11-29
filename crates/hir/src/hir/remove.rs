use crate::{
    module::ModuleKind,
    scope::Scope,
    source::Source,
    symbol::{ReferenceTarget, Symbol, SymbolData, SymbolKind},
    Hir,
};

impl Hir {
    pub fn remove_source(&mut self, source: Source) {
        self.sources.remove(source);

        let symbols_to_remove = self
            .symbols
            .iter()
            .filter(|(_, symbol_data)| symbol_data.source.is(source))
            .map(|(s, _)| s)
            .collect::<Vec<_>>();

        for symbol in symbols_to_remove {
            self.remove_symbol(symbol);
        }

        self.modules.retain(|module, module_data| {
            module_data.kind == ModuleKind::Static
                || self
                    .sources
                    .iter()
                    .any(|(_, source_data)| source_data.module == module)
        });
    }

    fn remove_scope(&mut self, scope: Scope) {
        if let Some(s) = self.scopes.remove(scope) {
            for symbol in s.symbols {
                self.remove_symbol(symbol);
            }

            for symbol in s.hoisted_symbols {
                self.remove_symbol(symbol);
            }
        }
    }

    fn remove_symbol(&mut self, symbol: Symbol) {
        if let Some(s) = self.symbols.remove(symbol) {
            if self.scopes.contains_key(s.parent_scope) {
                self.scope_mut(s.parent_scope).symbols.shift_remove(&symbol);
                self.scope_mut(s.parent_scope)
                    .hoisted_symbols
                    .remove(&symbol);
            }
            self.remove_symbol_data(s);
        }
    }

    fn remove_symbol_data(&mut self, s: SymbolData) {
        match s.kind {
            SymbolKind::Block(block) => self.remove_scope(block.scope),
            SymbolKind::Fn(f) => self.remove_scope(f.scope),
            SymbolKind::Decl(decl) => {
                if let Some(v) = decl.value_scope {
                    self.remove_scope(v);
                }
            }
            SymbolKind::Reference(r) => {
                if let Some(ReferenceTarget::Symbol(r_target)) = r.target {
                    if let Some(target) = self.symbols.get_mut(r_target) {
                        match &mut target.kind {
                            SymbolKind::Fn(f) => {
                                f.references.remove(&r_target);
                            }
                            SymbolKind::Decl(decl) => {
                                decl.references.remove(&r_target);
                            }
                            _ => {}
                        }
                    }
                }
            }
            SymbolKind::Path(path) => {
                let symbols = path
                    .segments
                    .into_iter()
                    .filter_map(|s| self.symbols.remove(s))
                    .collect::<Vec<_>>();

                for symbol in symbols {
                    self.remove_symbol_data(symbol);
                }
            }
            SymbolKind::Unary(unary) => {
                if let Some(s) = unary.rhs {
                    self.remove_symbol(s);
                }
            }
            SymbolKind::Binary(binary) => {
                if let Some(s) = binary.lhs {
                    self.remove_symbol(s);
                }
                if let Some(s) = binary.rhs {
                    self.remove_symbol(s);
                }
            }
            SymbolKind::Array(array) => {
                let symbols = array
                    .values
                    .into_iter()
                    .filter_map(|s| self.symbols.remove(s))
                    .collect::<Vec<_>>();

                for symbol in symbols {
                    self.remove_symbol_data(symbol);
                }
            }
            SymbolKind::Index(index) => {
                if let Some(s) = index.base {
                    self.remove_symbol(s);
                }

                if let Some(s) = index.index {
                    self.remove_symbol(s);
                }
            }
            SymbolKind::Object(object) => {
                let symbols = object
                    .fields
                    .into_iter()
                    .filter_map(|(_, field)| field.value)
                    .collect::<Vec<_>>();

                for symbol in symbols {
                    self.remove_symbol(symbol);
                }
            }
            SymbolKind::Call(call) => {
                if let Some(s) = call.lhs {
                    self.remove_symbol(s);
                }

                let symbols = call
                    .arguments
                    .into_iter()
                    .filter_map(|s| self.symbols.remove(s))
                    .collect::<Vec<_>>();

                for symbol in symbols {
                    self.remove_symbol_data(symbol);
                }
            }
            SymbolKind::Closure(f) => self.remove_scope(f.scope),
            SymbolKind::If(if_sym) => {
                for (condition, scope) in if_sym.branches {
                    if let Some(s) = condition {
                        self.remove_symbol(s);
                    }

                    self.remove_scope(scope);
                }
            }
            SymbolKind::Loop(lp) => {
                self.remove_scope(lp.scope);
            }
            SymbolKind::For(fr) => {
                if let Some(s) = fr.iterable {
                    self.remove_symbol(s);
                }

                self.remove_scope(fr.scope);
            }
            SymbolKind::While(wle) => {
                if let Some(s) = wle.condition {
                    self.remove_symbol(s);
                }

                self.remove_scope(wle.scope);
            }
            SymbolKind::Break(brk) => {
                if let Some(s) = brk.expr {
                    self.remove_symbol(s);
                }
            }
            SymbolKind::Return(ret) => {
                if let Some(s) = ret.expr {
                    self.remove_symbol(s);
                }
            }
            SymbolKind::Switch(switch) => {
                for (pat, val) in switch.arms {
                    if let Some(s) = pat {
                        self.remove_symbol(s);
                    }

                    if let Some(s) = val {
                        self.remove_symbol(s);
                    }
                }
            }
            SymbolKind::Import(import) => {
                if let Some(s) = import.expr {
                    self.remove_symbol(s);
                }

                if let Some(s) = import.alias {
                    self.remove_symbol(s);
                }
            }
            SymbolKind::Lit(_) | SymbolKind::Continue(_) | SymbolKind::Discard(_) => {}
            SymbolKind::Export(e) => {
                if let Some(s) = e.target {
                    self.remove_symbol(s);
                }
            }
            SymbolKind::Try(t) => {
                self.remove_scope(t.try_scope);
                self.remove_scope(t.catch_scope);
            }
            SymbolKind::Op(_op) => {
                // TODO
            }
        }
    }
}
