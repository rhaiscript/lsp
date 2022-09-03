use crate::{
    scope::Scope,
    source::Source,
    symbol::{ReferenceTarget, SwitchArm, Symbol, SymbolData, SymbolKind, VirtualSymbol},
    ty::Type,
    Hir, Module,
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

        let types_to_remove = self
            .types
            .iter()
            .filter(|(_, ty_data)| ty_data.source.is(source))
            .map(|(s, _)| s)
            .collect::<Vec<_>>();

        for symbol in symbols_to_remove {
            self.remove_symbol(symbol);
        }

        for ty in types_to_remove {
            self.remove_type(ty);
        }

        for m in self.modules.values_mut() {
            m.sources.remove(&source);
        }

        self.cleanup_modules();
    }

    /// Remove scopes and symbols of modules that
    /// are not protected and have no sources associated to them.
    fn cleanup_modules(&mut self) {
        let modules_to_remove = self
            .modules
            .keys()
            .filter(|m| {
                let m = &self[*m];

                !m.protected && m.sources.is_empty()
            })
            .collect::<Vec<_>>();

        for m in modules_to_remove {
            self.remove_module(m);
        }
    }

    fn remove_module(&mut self, module: Module) {
        if let Some(m) = self.modules.remove(module) {
            self.remove_scope(m.scope);

            let symbols_to_remove = self
                .symbols
                .keys()
                .filter(|sym| {
                    matches!(
                        &self[*sym].kind,
                        SymbolKind::Virtual(VirtualSymbol::Module(m))
                        if m.module == module
                    )
                })
                .collect::<Vec<_>>();

            for sym in symbols_to_remove {
                self.remove_symbol(sym);
            }
        }
    }

    pub(crate) fn remove_type(&mut self, ty: Type) {
        if let Some(ty) = self.types.get(ty) {
            if ty.protected {
                return;
            }
        }
        self.types.remove(ty);

        for symbol in self.symbols.values_mut() {
            if symbol.ty == ty {
                symbol.ty = self.builtin_types.unknown;
            }
        }
    }

    /// Recursively remove all descendant symbols and scopes,
    /// and then remove the scope itself.
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

    /// Recursively remove all descendant symbols and scopes,
    /// and then remove the symbol itself.
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
            SymbolKind::Ref(r) => {
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
                for symbol in path.segments {
                    self.remove_symbol(symbol);
                }
            }
            SymbolKind::Unary(unary) => {
                if let Some(s) = unary.rhs {
                    self.remove_symbol(s);
                }
            }
            SymbolKind::Binary(binary) => {
                self.remove_scope(binary.scope);
                if let Some(s) = binary.lhs {
                    self.remove_symbol(s);
                }
                if let Some(s) = binary.rhs {
                    self.remove_symbol(s);
                }
            }
            SymbolKind::Array(array) => {
                for symbol in array.values {
                    self.remove_symbol(symbol);
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

                for symbol in call.arguments {
                    self.remove_symbol(symbol);
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
                if let Some(s) = fr.cursor {
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
                for SwitchArm {
                    pat_expr,
                    condition_expr,
                    value_expr,
                } in switch.arms
                {
                    if let Some(s) = pat_expr {
                        self.remove_symbol(s);
                    }

                    if let Some(s) = condition_expr {
                        self.remove_symbol(s);
                    }

                    if let Some(s) = value_expr {
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
            SymbolKind::Lit(lit) => {
                for scope in lit.interpolated_scopes {
                    self.remove_scope(scope);
                }
            }
            SymbolKind::Continue(_)
            | SymbolKind::Discard(_)
            | SymbolKind::Op(_)
            | SymbolKind::TypeDecl(_) => {}
            SymbolKind::Export(e) => {
                if let Some(s) = e.target {
                    self.remove_symbol(s);
                }
            }
            SymbolKind::Try(t) => {
                self.remove_scope(t.try_scope);
                self.remove_scope(t.catch_scope);
            }
            SymbolKind::Throw(sym) => {
                if let Some(sym) = sym.expr {
                    self.remove_symbol(sym);
                }
            }
            SymbolKind::Virtual(virt) => {
                match virt {
                    VirtualSymbol::Module(_)
                    | VirtualSymbol::Proxy(_)
                    | VirtualSymbol::Alias(_) => {
                        // no cleanup needed.
                    }
                }
            }
        }
    }
}
