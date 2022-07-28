use super::*;
use crate::{module::ModuleKind, source::SourceInfo, util::script_url, IndexSet};
use rhai_rowan::{
    ast::{AstNode, Def, DefStmt, RhaiDef},
    syntax::SyntaxElement,
    util::unescape,
    T,
};

impl Hir {
    pub(super) fn add_def(&mut self, source: Source, def: &RhaiDef) {
        let def_mod = match def.def_module_decl() {
            Some(d) => d,
            None => return,
        };

        let docs = def_mod.docs_content();

        let def_mod = match def_mod.def_module() {
            Some(d) => d,
            None => return,
        };

        let module_kind = if def_mod.kw_static_token().is_some() {
            ModuleKind::Static
        } else if let Some(name) = def_mod.lit_str_token() {
            let mut lit_str = name.text();
            lit_str = lit_str
                .strip_prefix('"')
                .unwrap_or(lit_str)
                .strip_suffix('"')
                .unwrap_or(lit_str);

            let import_url =
                self.resolve_import_url(Some(&self[source].url), &unescape(lit_str, '"').0);

            match import_url {
                Some(url) => ModuleKind::Url(url),
                None => {
                    tracing::debug!("failed to resolve import url");
                    return;
                }
            }
        } else if let Some(name) = def_mod.ident_token() {
            ModuleKind::Url(
                format!("{STATIC_URL_SCHEME}://{}", name.text())
                    .parse()
                    .unwrap(),
            )
        } else {
            ModuleKind::Url(
                script_url(&self[source].url).unwrap_or_else(|| self[source].url.clone()),
            )
        };

        let module = self.ensure_module(module_kind);
        self.module_mut(module).docs = docs;
        self.module_mut(module).sources.insert(source);

        self.source_mut(source).module = module;

        if let ModuleKind::Url(url) = &self[module].kind {
            if url.scheme() == STATIC_URL_SCHEME {
                self.add_module_to_static_scope(module);
            }
        }

        for stmt in def.statements() {
            self.add_def_statement(source, self[module].scope, &stmt);
        }
    }

    pub(super) fn add_def_statement(&mut self, source: Source, scope: Scope, stmt: &DefStmt) {
        let def = match stmt.item().and_then(|it| it.def()) {
            Some(d) => d,
            None => return,
        };

        let docs = stmt.item().map(|it| it.docs_content()).unwrap_or_default();

        match def {
            Def::Import(import_def) => {
                let import_scope = self.scopes.insert(ScopeData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: import_def.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    ..ScopeData::default()
                });

                let symbol_data = SymbolData {
                    export: true,
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: import_def.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Import(ImportSymbol {
                        target: None,
                        scope: import_scope,
                        alias: import_def.alias().map(|alias| {
                            let alias_symbol = self.add_symbol(SymbolData {
                                export: true,
                                source: SourceInfo {
                                    source: Some(source),
                                    text_range: alias.text_range().into(),
                                    selection_text_range: None,
                                },
                                kind: SymbolKind::Decl(Box::new(DeclSymbol {
                                    name: alias.text().into(),
                                    is_import: true,
                                    ..DeclSymbol::default()
                                })),
                                parent_scope: Scope::default(),
                            });

                            import_scope.add_symbol(self, alias_symbol, true);

                            alias_symbol
                        }),
                        expr: import_def.expr().and_then(|expr| {
                            self.add_expression(source, import_scope, false, expr)
                        }),
                    }),
                };

                let symbol = self.add_symbol(symbol_data);

                scope.add_symbol(self, symbol, true);
                import_scope.set_parent(self, symbol);
            }
            Def::Const(const_def) => {
                let ident_token = match const_def.ident_token() {
                    Some(s) => s,
                    None => return,
                };

                let symbol = self.symbols.insert(SymbolData {
                    export: true,
                    source: SourceInfo {
                        source: Some(source),
                        text_range: Some(const_def.syntax().text_range()),
                        selection_text_range: Some(ident_token.text_range()),
                    },
                    parent_scope: Scope::default(),
                    kind: SymbolKind::Decl(Box::new(DeclSymbol {
                        name: ident_token.text().into(),
                        is_const: true,
                        value: None,
                        value_scope: None,
                        docs,
                        ..DeclSymbol::default()
                    })),
                });

                scope.add_symbol(self, symbol, true);
            }
            Def::Let(let_def) => {
                let ident_token = match let_def.ident_token() {
                    Some(s) => s,
                    None => return,
                };

                let symbol = self.symbols.insert(SymbolData {
                    export: false,
                    source: SourceInfo {
                        source: Some(source),
                        text_range: Some(let_def.syntax().text_range()),
                        selection_text_range: Some(ident_token.text_range()),
                    },
                    parent_scope: Scope::default(),
                    kind: SymbolKind::Decl(Box::new(DeclSymbol {
                        name: ident_token.text().into(),
                        is_const: false,
                        value: None,
                        value_scope: None,
                        docs,
                        ..DeclSymbol::default()
                    })),
                });

                scope.add_symbol(self, symbol, true);
            }
            Def::Fn(expr) => {
                let fn_scope = self.scopes.insert(ScopeData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    ..ScopeData::default()
                });

                if let Some(param_list) = expr.typed_param_list() {
                    for param in param_list.params() {
                        let symbol = self.add_symbol(SymbolData {
                            export: false,
                            parent_scope: Scope::default(),
                            source: SourceInfo {
                                source: Some(source),
                                text_range: param.syntax().text_range().into(),
                                selection_text_range: param.ident_token().map(|t| t.text_range()),
                            },
                            kind: SymbolKind::Decl(Box::new(DeclSymbol {
                                name: param
                                    .ident_token()
                                    .map(|s| s.text().to_string())
                                    .unwrap_or_default(),
                                is_param: true,
                                ..DeclSymbol::default()
                            })),
                        });

                        fn_scope.add_symbol(self, symbol, false);
                    }
                }

                let symbol = self.add_symbol(SymbolData {
                    export: true,
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: expr.ident_token().map(|t| t.text_range()),
                    },
                    kind: SymbolKind::Fn(FnSymbol {
                        name: expr
                            .ident_token()
                            .map(|s| s.text().to_string())
                            .unwrap_or_default(),
                        docs,
                        scope: fn_scope,
                        getter: expr.has_kw_get(),
                        setter: expr.has_kw_set(),
                        ..FnSymbol::default()
                    }),
                });

                scope.add_symbol(self, symbol, true);
                fn_scope.set_parent(self, symbol);
            }
            Def::Op(f) => {
                let name_token = f
                    .syntax()
                    .children_with_tokens()
                    .filter_map(SyntaxElement::into_token)
                    .skip(1)
                    .find(|t| t.kind() == T!["ident"] || t.kind().infix_binding_power().is_some());

                let ident = match name_token {
                    Some(i) => i,
                    None => return,
                };

                let symbol = self.symbols.insert(SymbolData {
                    export: true,
                    source: SourceInfo {
                        source: Some(source),
                        text_range: Some(f.syntax().text_range()),
                        selection_text_range: Some(ident.text_range()),
                    },
                    parent_scope: Scope::default(),
                    kind: SymbolKind::Op(OpSymbol {
                        name: ident.text().into(),
                        docs,
                        binding_powers: f
                            .precedence()
                            .and_then(|precedence| {
                                let mut bps = precedence.binding_powers();

                                let bp_l: u8 = bps.next().and_then(|bp| bp.text().parse().ok())?;
                                let bp_r: u8 = bps
                                    .next()
                                    .and_then(|bp| bp.text().parse().ok())
                                    .unwrap_or_else(|| bp_l.saturating_add(1));

                                Some((bp_l, bp_r))
                            })
                            .unwrap_or((1, 2)),
                        ..OpSymbol::default()
                    }),
                });

                scope.add_symbol(self, symbol, true);
            }
            Def::ModuleInline(m) => {
                let ident = m.ident_token();

                let ident = match ident {
                    Some(ident) => ident,
                    None => return,
                };

                let module_scope = self.scopes.insert(ScopeData::default());

                module_scope.set_parent(self, scope);

                let module = self.modules.insert(ModuleData {
                    scope: module_scope,
                    kind: ModuleKind::Inline,
                    protected: false,
                    sources: IndexSet::from_iter([source]),
                    docs,
                });

                for statement in m.statements() {
                    self.add_def_statement(source, module_scope, &statement);
                }

                let virt_module_symbol = self.add_symbol(SymbolData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: Some(m.syntax().text_range()),
                        selection_text_range: Some(ident.text_range()),
                    },
                    parent_scope: Scope::default(),
                    kind: SymbolKind::Virtual(VirtualSymbol::Module(VirtualModuleSymbol {
                        name: ident.text().to_string(),
                        module,
                    })),
                    export: true,
                });

                scope.add_symbol(self, virt_module_symbol, true);
            }
            Def::Type(_) => {
                // TODO
            }
        }
    }
}
