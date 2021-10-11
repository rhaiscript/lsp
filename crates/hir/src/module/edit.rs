use super::*;

impl Module {
    pub(crate) fn new_from_syntax(name: &str, syntax: &SyntaxNode) -> Option<Module> {
        Rhai::cast(syntax.clone()).map(|rhai| {
            let mut m = Module {
                name: name.into(),
                syntax: Some(syntax.into()),
                ..Default::default()
            };
            let root_scope = m.create_scope(None, Some(syntax.into()));
            m.root_scope = root_scope;
            m.extend_scope_from_statements(root_scope, rhai.statements());
            m
        })
    }

    fn create_scope(&mut self, parent_symbol: Option<Symbol>, syntax: Option<SyntaxInfo>) -> Scope {
        let data = ScopeData {
            parent_symbol,
            syntax,
            ..Default::default()
        };

        let scope = self.scopes.insert(data);

        scope
    }

    fn create_scope_with_statements(
        &mut self,
        parent_symbol: Option<Symbol>,
        syntax: Option<SyntaxInfo>,
        statements: impl Iterator<Item = Stmt>,
    ) -> Scope {
        let scope = self.create_scope(parent_symbol, syntax);
        self.extend_scope_from_statements(scope, statements);
        scope
    }

    fn extend_scope_from_statements(
        &mut self,
        scope: Scope,
        statements: impl Iterator<Item = Stmt>,
    ) {
        for stmt in statements {
            self.create_symbol_from_stmt(scope, stmt);
        }
    }

    fn add_symbol_from_expr(&mut self, parent_scope: Scope, expr: Expr) -> Option<Symbol> {
        match expr {
            Expr::Fn(expr_fn) => {
                let scope = self.scopes.insert(ScopeData {
                    syntax: Some(expr_fn.syntax().into()),
                    ..Default::default()
                });

                if let Some(param_list) = expr_fn.param_list() {
                    for param in param_list.params() {
                        let symbol = self.symbols.insert(SymbolData {
                            selection_syntax: Some(param.syntax().into()),
                            syntax: Some(param.syntax().into()),
                            parent_scope: scope,
                            kind: SymbolKind::Decl(DeclSymbol {
                                name: param
                                    .ident_token()
                                    .map(|s| s.text().to_string())
                                    .unwrap_or_default(),
                                is_param: true,
                                ..Default::default()
                            }),
                        });

                        // safety: we've just inserted it.
                        let scope_data = unsafe { self.scopes.get_unchecked_mut(scope) };
                        scope_data.symbols.insert(symbol);
                    }
                }

                if let Some(body) = expr_fn.body() {
                    self.extend_scope_from_statements(scope, body.statements())
                }

                let sym = SymbolData {
                    selection_syntax: expr_fn.ident_token().map(Into::into),
                    parent_scope,
                    syntax: Some(expr_fn.syntax().into()),
                    kind: SymbolKind::Fn(FnSymbol {
                        name: expr_fn
                            .ident_token()
                            .map(|s| s.text().to_string())
                            .unwrap_or_default(),
                        scope,
                        ..Default::default()
                    }),
                };

                let sym = self.symbols.insert(sym);
                // safety: we've just inserted it.
                unsafe {
                    self.scopes.get_unchecked_mut(scope).parent_symbol = Some(sym);
                }

                self.add_to_scope(parent_scope, sym, true);
                Some(sym)
            }
            Expr::Let(expr_let) => {
                let value = expr_let.expr().and_then(|expr| {
                    let scope = self.create_scope(None, Some(expr_let.syntax().into()));
                    self.add_symbol_from_expr(scope, expr);
                    Some(scope)
                });

                let sym = SymbolData {
                    selection_syntax: expr_let.ident_token().map(Into::into),
                    parent_scope,
                    syntax: Some(expr_let.syntax().into()),
                    kind: SymbolKind::Decl(DeclSymbol {
                        name: expr_let
                            .ident_token()
                            .map(|s| s.text().to_string())
                            .unwrap_or_default(),
                        value,
                        ..Default::default()
                    }),
                };

                let sym = self.symbols.insert(sym);

                if let Some(scope) = value {
                    // safety: guaranteed to exist
                    unsafe { self.scopes.get_unchecked_mut(scope).parent_symbol = Some(sym) }
                }

                self.add_to_scope(parent_scope, sym, false);
                Some(sym)
            }
            Expr::Const(expr_const) => {
                let value = expr_const.expr().and_then(|expr| {
                    let scope = self.create_scope(None, Some(expr_const.syntax().into()));
                    self.add_symbol_from_expr(scope, expr);
                    Some(scope)
                });

                let sym = SymbolData {
                    selection_syntax: expr_const.ident_token().map(Into::into),
                    parent_scope,
                    syntax: Some(expr_const.syntax().into()),
                    kind: SymbolKind::Decl(DeclSymbol {
                        name: expr_const
                            .ident_token()
                            .map(|s| s.text().to_string())
                            .unwrap_or_default(),
                        is_const: true,
                        value,
                        ..Default::default()
                    }),
                };

                let sym = self.symbols.insert(sym);

                if let Some(scope) = value {
                    // safety: guaranteed to exist
                    unsafe { self.scopes.get_unchecked_mut(scope).parent_symbol = Some(sym) }
                }

                self.add_to_scope(parent_scope, sym, false);
                Some(sym)
            }
            Expr::Ident(expr_ident) => {
                let sym = SymbolData {
                    selection_syntax: Some(
                        expr_ident
                            .ident_token()
                            .map(|t| t.into())
                            .unwrap_or_else(|| expr_ident.syntax().into()),
                    ),
                    parent_scope,
                    syntax: Some(expr_ident.syntax().into()),
                    kind: SymbolKind::Reference(ReferenceSymbol {
                        name: expr_ident
                            .ident_token()
                            .map(|s| s.text().to_string())
                            .unwrap_or_default(),
                        ..Default::default()
                    }),
                };

                let sym = self.symbols.insert(sym);
                self.add_to_scope(parent_scope, sym, false);
                Some(sym)
            }
            Expr::Path(expr_path) => {
                let segments = match expr_path.path() {
                    Some(p) => p.segments(),
                    None => return None,
                };

                let sym = SymbolData {
                    selection_syntax: None,
                    parent_scope,
                    syntax: Some(expr_path.syntax().into()),
                    kind: SymbolKind::Path(PathSymbol {
                        segments: segments
                            .map(|s| {
                                self.symbols.insert(SymbolData {
                                    selection_syntax: None,
                                    parent_scope,
                                    kind: SymbolKind::Reference(ReferenceSymbol {
                                        name: s.text().to_string(),
                                        ..Default::default()
                                    }),
                                    syntax: Some(s.into()),
                                })
                            })
                            .collect(),
                        ..Default::default()
                    }),
                };

                let sym = self.symbols.insert(sym);
                self.add_to_scope(parent_scope, sym, false);
                Some(sym)
            }
            Expr::Lit(expr_lit) => {
                let sym = self.symbols.insert(SymbolData {
                    selection_syntax: None,
                    parent_scope,
                    syntax: Some(expr_lit.syntax().into()),
                    kind: SymbolKind::Lit(LitSymbol {
                        ..Default::default()
                    }),
                });
                self.add_to_scope(parent_scope, sym, false);
                Some(sym)
            }
            Expr::Block(expr_block) => {
                let scope = self.create_scope_with_statements(
                    None,
                    Some(expr_block.syntax().into()),
                    expr_block.statements(),
                );

                let sym = SymbolData {
                    selection_syntax: None,
                    parent_scope,
                    syntax: Some(expr_block.syntax().into()),
                    kind: SymbolKind::Block(BlockSymbol { scope }),
                };

                let sym = self.symbols.insert(sym);

                // safety: we've just inserted it.
                unsafe { self.scopes.get_unchecked_mut(scope).parent_symbol = Some(sym) }

                self.add_to_scope(parent_scope, sym, false);
                Some(sym)
            }
            Expr::Unary(expr_unary) => {
                let rhs = expr_unary
                    .expr()
                    .and_then(|rhs| self.add_symbol_from_expr(parent_scope, rhs));

                let sym = SymbolData {
                    selection_syntax: None,
                    parent_scope,
                    syntax: Some(expr_unary.syntax().into()),
                    kind: SymbolKind::Unary(UnarySymbol {
                        op: expr_unary.op_token().map(|t| t.kind()),
                        rhs,
                    }),
                };

                let sym = self.symbols.insert(sym);
                self.add_to_scope(parent_scope, sym, false);
                Some(sym)
            }
            Expr::Binary(expr_binary) => {
                let lhs = expr_binary
                    .lhs()
                    .and_then(|lhs| self.add_symbol_from_expr(parent_scope, lhs));

                let rhs = expr_binary
                    .rhs()
                    .and_then(|rhs| self.add_symbol_from_expr(parent_scope, rhs));

                let sym = SymbolData {
                    selection_syntax: None,
                    parent_scope,
                    syntax: Some(expr_binary.syntax().into()),
                    kind: SymbolKind::Binary(BinarySymbol {
                        rhs,
                        op: expr_binary.op_token().map(|t| t.kind()),
                        lhs,
                    }),
                };

                let sym = self.symbols.insert(sym);
                self.add_to_scope(parent_scope, sym, false);
                Some(sym)
            }
            Expr::Paren(expr_paren) => match expr_paren.expr() {
                Some(expr) => self.add_symbol_from_expr(parent_scope, expr),
                None => None,
            },
            Expr::Array(expr_array) => {
                let sym = SymbolData {
                    selection_syntax: None,
                    parent_scope,
                    syntax: Some(expr_array.syntax().into()),
                    kind: SymbolKind::Array(ArraySymbol {
                        values: expr_array
                            .values()
                            .filter_map(|expr| self.add_symbol_from_expr(parent_scope, expr))
                            .collect(),
                    }),
                };

                Some(self.symbols.insert(sym))
            }
            Expr::Index(expr_index) => {
                let base = expr_index
                    .base()
                    .and_then(|base| self.add_symbol_from_expr(parent_scope, base));

                let index = expr_index
                    .index()
                    .and_then(|index| self.add_symbol_from_expr(parent_scope, index));

                let sym = SymbolData {
                    selection_syntax: None,
                    parent_scope,
                    syntax: Some(expr_index.syntax().into()),
                    kind: SymbolKind::Index(IndexSymbol { base, index }),
                };

                let sym = self.symbols.insert(sym);
                self.add_to_scope(parent_scope, sym, false);
                Some(sym)
            }
            Expr::Object(expr_object) => {
                let sym = SymbolData {
                    selection_syntax: None,
                    parent_scope,
                    syntax: Some(expr_object.syntax().into()),
                    kind: SymbolKind::Object(ObjectSymbol {
                        fields: expr_object
                            .fields()
                            .filter_map(|field| match (field.property(), field.expr()) {
                                (Some(name), Some(expr)) => Some((
                                    name.text().to_string(),
                                    ObjectField {
                                        property_name: name.text().to_string(),
                                        property_syntax: Some(name.into()),
                                        field_syntax: Some(field.syntax().into()),
                                        value: self.add_symbol_from_expr(parent_scope, expr),
                                    },
                                )),
                                _ => None,
                            })
                            .collect(),
                    }),
                };

                let sym = self.symbols.insert(sym);
                self.add_to_scope(parent_scope, sym, false);
                Some(sym)
            }
            Expr::Call(expr_call) => {
                let lhs = expr_call
                    .expr()
                    .and_then(|expr| self.add_symbol_from_expr(parent_scope, expr));

                let sym = SymbolData {
                    selection_syntax: None,
                    parent_scope,
                    syntax: Some(expr_call.syntax().into()),
                    kind: SymbolKind::Call(CallSymbol {
                        lhs,
                        arguments: match expr_call.arg_list() {
                            Some(arg_list) => arg_list
                                .arguments()
                                .filter_map(|expr| self.add_symbol_from_expr(parent_scope, expr))
                                .collect(),
                            None => Default::default(),
                        },
                    }),
                };

                let sym = self.symbols.insert(sym);
                self.add_to_scope(parent_scope, sym, false);
                Some(sym)
            }
            Expr::Closure(expr_closure) => {
                let scope = self.scopes.insert(ScopeData {
                    syntax: Some(expr_closure.syntax().into()),
                    ..Default::default()
                });

                if let Some(param_list) = expr_closure.param_list() {
                    for param in param_list.params() {
                        let symbol = self.symbols.insert(SymbolData {
                            selection_syntax: Some(param.syntax().into()),
                            syntax: Some(param.syntax().into()),
                            parent_scope: scope,
                            kind: SymbolKind::Decl(DeclSymbol {
                                name: param
                                    .ident_token()
                                    .map(|s| s.text().to_string())
                                    .unwrap_or_default(),
                                is_param: true,
                                ..Default::default()
                            }),
                        });

                        // safety: we've just inserted it.
                        let scope_data = unsafe { self.scopes.get_unchecked_mut(scope) };
                        scope_data.symbols.insert(symbol);
                    }
                }

                let expr = expr_closure
                    .body()
                    .and_then(|body| self.add_symbol_from_expr(scope, body));

                let sym = SymbolData {
                    selection_syntax: None,
                    parent_scope,
                    syntax: Some(expr_closure.syntax().into()),
                    kind: SymbolKind::Closure(ClosureSymbol { scope, expr }),
                };

                let sym = self.symbols.insert(sym);

                // safety: we've just inserted it.
                unsafe { self.scopes.get_unchecked_mut(scope).parent_symbol = Some(sym) }

                self.add_to_scope(parent_scope, sym, false);
                Some(sym)
            }
            Expr::If(expr_if) => {
                let condition = expr_if
                    .expr()
                    .and_then(|expr| self.add_symbol_from_expr(parent_scope, expr));

                let then_scope = match expr_if.then_branch() {
                    Some(body) => self.create_scope_with_statements(
                        None,
                        Some(body.syntax().into()),
                        body.statements(),
                    ),
                    None => return None,
                };

                let sym = self.symbols.insert(SymbolData {
                    selection_syntax: None,
                    parent_scope,
                    syntax: Some(expr_if.syntax().into()),
                    kind: SymbolKind::If(IfSymbol {
                        condition,
                        then_scope,
                        ..Default::default()
                    }),
                });

                self.add_to_scope(parent_scope, sym, false);

                fn get_if_symbol(module: &mut Module, symbol: Symbol) -> &mut IfSymbol {
                    // safety: we've just inserted it.
                    let symbol_data = unsafe { module.symbols.get_unchecked_mut(symbol) };
                    match &mut symbol_data.kind {
                        SymbolKind::If(k) => k,
                        _ => unreachable!(),
                    }
                }

                if let Some(else_branch) = expr_if.else_branch() {
                    let branch_scope = self.create_scope_with_statements(
                        Some(sym),
                        Some(else_branch.syntax().into()),
                        else_branch.statements(),
                    );

                    get_if_symbol(self, sym)
                        .branches
                        .insert((None, branch_scope));

                    return Some(sym);
                }

                let mut next_branch = expr_if.else_if_branch();

                while let Some(branch) = next_branch.take() {
                    let branch_condition = expr_if
                        .expr()
                        .and_then(|expr| self.add_symbol_from_expr(parent_scope, expr));

                    let then_scope = match branch.then_branch() {
                        Some(body) => self.create_scope_with_statements(
                            Some(sym),
                            Some(body.syntax().into()),
                            body.statements(),
                        ),
                        None => break,
                    };

                    get_if_symbol(self, sym)
                        .branches
                        .insert((branch_condition, then_scope));

                    if let Some(else_branch) = expr_if.else_branch() {
                        let branch_scope = self.create_scope_with_statements(
                            Some(sym),
                            Some(else_branch.syntax().into()),
                            else_branch.statements(),
                        );

                        get_if_symbol(self, sym)
                            .branches
                            .insert((None, branch_scope));

                        break;
                    }

                    next_branch = branch.else_if_branch();
                }

                Some(sym)
            }
            Expr::Loop(expr_loop) => {
                let scope = match expr_loop.loop_body() {
                    Some(body) => self.create_scope_with_statements(
                        None,
                        Some(body.syntax().into()),
                        body.statements(),
                    ),
                    None => return None,
                };

                let sym = SymbolData {
                    selection_syntax: None,
                    parent_scope,
                    syntax: Some(expr_loop.syntax().into()),
                    kind: SymbolKind::Loop(LoopSymbol { scope }),
                };

                let sym = self.symbols.insert(sym);
                // safety: we've just inserted it.
                unsafe {
                    self.scopes.get_unchecked_mut(scope).parent_symbol = Some(sym);
                }
                self.add_to_scope(parent_scope, sym, false);
                Some(sym)
            }
            Expr::For(expr_for) => {
                let scope = match expr_for.loop_body() {
                    Some(body) => self.create_scope_with_statements(
                        None,
                        Some(body.syntax().into()),
                        body.statements(),
                    ),
                    None => return None,
                };

                let sym = SymbolData {
                    selection_syntax: None,
                    parent_scope,
                    syntax: Some(expr_for.syntax().into()),
                    kind: SymbolKind::For(ForSymbol {
                        iterable: expr_for
                            .iterable()
                            .and_then(|expr| self.add_symbol_from_expr(parent_scope, expr)),
                        scope,
                    }),
                };

                let sym = self.symbols.insert(sym);
                // safety: we've just inserted it.
                unsafe {
                    self.scopes.get_unchecked_mut(scope).parent_symbol = Some(sym);
                }
                self.add_to_scope(parent_scope, sym, false);
                Some(sym)
            }
            Expr::While(expr_while) => {
                let scope = match expr_while.loop_body() {
                    Some(body) => self.create_scope_with_statements(
                        None,
                        Some(body.syntax().into()),
                        body.statements(),
                    ),
                    None => return None,
                };

                let sym = SymbolData {
                    selection_syntax: None,
                    parent_scope,
                    syntax: Some(expr_while.syntax().into()),
                    kind: SymbolKind::While(WhileSymbol {
                        condition: expr_while
                            .expr()
                            .and_then(|expr| self.add_symbol_from_expr(parent_scope, expr)),
                        scope,
                    }),
                };

                let sym = self.symbols.insert(sym);
                // safety: we've just inserted it.
                unsafe {
                    self.scopes.get_unchecked_mut(scope).parent_symbol = Some(sym);
                }
                self.add_to_scope(parent_scope, sym, false);
                Some(sym)
            }
            Expr::Break(expr_break) => {
                let sym = SymbolData {
                    selection_syntax: None,
                    parent_scope,
                    syntax: Some(expr_break.syntax().into()),
                    kind: SymbolKind::Break(BreakSymbol {
                        expr: expr_break
                            .expr()
                            .and_then(|expr| self.add_symbol_from_expr(parent_scope, expr)),
                    }),
                };

                let sym = self.symbols.insert(sym);
                self.add_to_scope(parent_scope, sym, false);
                Some(sym)
            }
            Expr::Continue(expr_continue) => {
                let sym = SymbolData {
                    selection_syntax: None,
                    parent_scope,
                    syntax: Some(expr_continue.syntax().into()),
                    kind: SymbolKind::Continue(ContinueSymbol {}),
                };

                let sym = self.symbols.insert(sym);
                self.add_to_scope(parent_scope, sym, false);
                Some(sym)
            }
            Expr::Return(expr_return) => {
                let sym = SymbolData {
                    selection_syntax: None,
                    parent_scope,
                    syntax: Some(expr_return.syntax().into()),
                    kind: SymbolKind::Return(ReturnSymbol {
                        expr: expr_return
                            .expr()
                            .and_then(|expr| self.add_symbol_from_expr(parent_scope, expr)),
                    }),
                };

                let sym = self.symbols.insert(sym);
                self.add_to_scope(parent_scope, sym, false);
                Some(sym)
            }
            Expr::Switch(_) => None,
            Expr::Import(_) => None,
        }
    }

    fn add_to_scope(&mut self, scope: Scope, symbol: Symbol, hoist: bool) {
        // safety: guaranteed to exist.
        unsafe {
            let scope_data = self.scopes.get_unchecked_mut(scope);
            if hoist {
                let inserted = scope_data.hoisted_symbols.insert(symbol);
                debug_assert!(inserted);
            } else {
                let inserted = scope_data.symbols.insert(symbol);
                debug_assert!(inserted);
            }
        }
    }

    fn create_symbol_from_stmt(&mut self, parent_scope: Scope, stmt: Stmt) -> Option<Symbol> {
        stmt.item().and_then(|item| {
            item.expr().and_then(|expr| {
                match self.add_symbol_from_expr(parent_scope, expr) {
                    Some(symbol) => {
                        // safety: we've just inserted it.
                        let sym = unsafe { self.symbols.get_unchecked_mut(symbol) };
                        match &mut sym.kind {
                            SymbolKind::Fn(f) => f.docs = item.docs_content(),
                            SymbolKind::Decl(decl) => decl.docs = item.docs_content(),
                            _ => {}
                        };

                        Some(symbol)
                    }
                    None => None,
                }
            })
        })
    }
}

impl Module {
    pub(crate) fn resolve_references(&mut self) {
        let self_ptr = self as *mut Module;

        for (symbol, ref_kind) in self
            .symbols
            .iter_mut()
            .filter_map(|(s, d)| match &mut d.kind {
                SymbolKind::Reference(r) => Some((s, r)),
                _ => None,
            })
        {
            ref_kind.target = None;

            // safety: This is safe because we only operate
            //  on separate elements (declarations and refs)
            //  and we don't touch the map itself.
            //
            // Without this unsafe block, we'd have to unnecessarily
            // allocate a vector of symbols.
            unsafe {
                for vis_symbol in (&*self_ptr).visible_symbols_from_symbol(symbol) {
                    let vis_symbol_data = (&mut *self_ptr).symbols.get_unchecked_mut(vis_symbol);
                    if let Some(n) = vis_symbol_data.name() {
                        if n != ref_kind.name {
                            continue;
                        }
                    }

                    match &mut vis_symbol_data.kind {
                        SymbolKind::Fn(target) => {
                            target.references.insert(symbol);
                        }
                        SymbolKind::Decl(target) => {
                            target.references.insert(symbol);
                        }
                        _ => unreachable!(),
                    }

                    ref_kind.target = Some(ReferenceTarget::Symbol(vis_symbol));
                    break;
                }
            }
        }
    }
}
