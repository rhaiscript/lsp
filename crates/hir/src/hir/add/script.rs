use rhai_rowan::{
    ast::{ExportTarget, Expr, Rhai, Stmt},
    syntax::{SyntaxKind, SyntaxToken},
};
use crate::{eval::Value, source::SourceInfo};

use super::*;

impl Hir {
    pub(crate) fn add_script(&mut self, source: Source, scope: Scope, rhai: &Rhai) {
        self.add_statements(source, scope, rhai.statements());
    }

    fn add_statements(
        &mut self,
        source: Source,
        scope: Scope,
        statements: impl Iterator<Item = Stmt>,
    ) {
        for statement in statements {
            self.add_statement(source, scope, statement);
        }
    }

    #[tracing::instrument(skip(self), level = "trace")]
    fn add_statement(&mut self, source: Source, scope: Scope, stmt: Stmt) -> Option<Symbol> {
        stmt.item().and_then(|item| {
            item.expr()
                .and_then(|expr| match self.add_expression(source, scope, expr) {
                    Some(symbol) => {
                        match &mut self.symbol_mut(symbol).kind {
                            SymbolKind::Fn(f) => f.docs = item.docs_content(),
                            SymbolKind::Decl(decl) => decl.docs = item.docs_content(),
                            _ => {}
                        };
                        Some(symbol)
                    }
                    None => None,
                })
        })
    }

    #[tracing::instrument(skip(self), level = "trace")]
    fn add_expression(&mut self, source: Source, scope: Scope, expr: Expr) -> Option<Symbol> {
        /// `let` or `const`
        fn add_decl(
            source: Source,
            hir: &mut Hir,
            scope: Scope,
            value: Option<Expr>,
            ident_syntax: Option<SyntaxToken>,
            syntax: &SyntaxNode,
            is_const: bool,
        ) -> Symbol {
            let (value, value_scope) = value
                .map(|expr| {
                    let scope = hir.scopes.insert(ScopeData {
                        source: SourceInfo {
                            source: Some(source),
                            text_range: expr.syntax().text_range().into(),
                            selection_text_range: None,
                        },
                        ..ScopeData::default()
                    });
                    (hir.add_expression(source, scope, expr), Some(scope))
                })
                .unwrap_or_default();

            let symbol = hir.symbols.insert(SymbolData {
                parent_scope: Scope::default(),
                source: SourceInfo {
                    source: Some(source),
                    text_range: syntax.text_range().into(),
                    selection_text_range: ident_syntax.as_ref().map(SyntaxToken::text_range),
                },
                kind: SymbolKind::Decl(Box::new(DeclSymbol {
                    name: ident_syntax
                        .map(|s| s.text().to_string())
                        .unwrap_or_default(),
                    is_const,
                    value,
                    value_scope,
                    ..DeclSymbol::default()
                })),
            });

            if let Some(value_scope) = value_scope {
                value_scope.set_parent(hir, symbol);
            }

            scope.add_symbol(hir, symbol, false);
            symbol
        }

        match expr {
            Expr::Ident(expr) => {
                let symbol = self.symbols.insert(SymbolData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: expr.ident_token().map(|t| t.text_range()),
                    },
                    kind: SymbolKind::Reference(ReferenceSymbol {
                        name: expr
                            .ident_token()
                            .map(|s| s.text().to_string())
                            .unwrap_or_default(),
                        ..ReferenceSymbol::default()
                    }),
                    parent_scope: Scope::default(),
                });

                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Path(expr_path) => {
                let segments = match expr_path.path() {
                    Some(p) => p.segments(),
                    None => return None,
                };
                let symbol = SymbolData {
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr_path.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Path(PathSymbol {
                        segments: segments
                            .map(|s| {
                                let symbol = self.symbols.insert(SymbolData {
                                    source: SourceInfo {
                                        source: Some(source),
                                        text_range: s.text_range().into(),
                                        selection_text_range: None,
                                    },
                                    parent_scope: Scope::default(),
                                    kind: SymbolKind::Reference(ReferenceSymbol {
                                        name: s.text().to_string(),
                                        part_of_path: true,
                                        ..ReferenceSymbol::default()
                                    }),
                                });
                                scope.add_symbol(self, symbol, false);
                                symbol
                            })
                            .collect(),
                    }),
                };
                let sym = self.symbols.insert(symbol);

                scope.add_symbol(self, sym, false);
                Some(sym)
            }
            Expr::Lit(expr) => {
                let symbol = self.symbols.insert(SymbolData {
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Lit(LitSymbol {
                        ty: expr
                            .lit()
                            .and_then(|l| l.token())
                            .map(|lit| match lit.kind() {
                                SyntaxKind::LIT_INT => Type::Int,
                                SyntaxKind::LIT_FLOAT => Type::Float,
                                SyntaxKind::LIT_BOOL => Type::Bool,
                                SyntaxKind::LIT_STR => Type::String,
                                SyntaxKind::LIT_CHAR => Type::Char,
                                _ => Type::Unknown,
                            })
                            .unwrap_or(Type::Unknown),
                        value: expr
                            .lit()
                            .and_then(|l| l.token())
                            .map(|lit| match lit.kind() {
                                SyntaxKind::LIT_INT => lit
                                    .text()
                                    .parse::<i64>()
                                    .map(Value::Int)
                                    .unwrap_or(Value::Unknown),
                                SyntaxKind::LIT_FLOAT => lit
                                    .text()
                                    .parse::<f64>()
                                    .map(Value::Float)
                                    .unwrap_or(Value::Unknown),
                                SyntaxKind::LIT_BOOL => lit
                                    .text()
                                    .parse::<bool>()
                                    .map(Value::Bool)
                                    .unwrap_or(Value::Unknown),
                                // TODO: parse string and char literals
                                // SyntaxKind::LIT_STR => Value::Unknown,
                                // SyntaxKind::LIT_CHAR => Value::Unknown,
                                _ => Value::Unknown,
                            })
                            .unwrap_or(Value::Unknown),
                    }),
                });

                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            // `let` and `const` values have a separate scope created for them
            Expr::Let(expr) => add_decl(
                source,
                self,
                scope,
                expr.expr(),
                expr.ident_token(),
                &expr.syntax(),
                false,
            )
            .into(),
            Expr::Const(expr) => add_decl(
                source,
                self,
                scope,
                expr.expr(),
                expr.ident_token(),
                &expr.syntax(),
                true,
            )
            .into(),
            Expr::Block(expr) => {
                let block_scope = self.scopes.insert(ScopeData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    ..ScopeData::default()
                });

                let symbol = self.symbols.insert(SymbolData {
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Block(BlockSymbol { scope: block_scope }),
                });

                block_scope.set_parent(self, symbol);
                self.add_statements(source, block_scope, expr.statements());

                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Unary(expr) => {
                let rhs = expr
                    .expr()
                    .and_then(|rhs| self.add_expression(source, scope, rhs));

                let symbol = self.symbols.insert(SymbolData {
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Unary(UnarySymbol {
                        op: expr.op_token().map(|t| t.kind()),
                        rhs,
                    }),
                });

                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Binary(expr) => {
                let lhs = expr
                    .lhs()
                    .and_then(|lhs| self.add_expression(source, scope, lhs));

                let rhs = expr
                    .rhs()
                    .and_then(|rhs| self.add_expression(source, scope, rhs));

                let symbol = self.symbols.insert(SymbolData {
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Binary(BinarySymbol {
                        rhs,
                        op: expr.op_token().map(|t| t.kind()),
                        lhs,
                    }),
                });

                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Paren(expr) => expr
                .expr()
                .and_then(|expr| self.add_expression(source, scope, expr)),
            Expr::Array(expr) => {
                let symbol_data = SymbolData {
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Array(ArraySymbol {
                        values: expr
                            .values()
                            .filter_map(|expr| self.add_expression(source, scope, expr))
                            .collect(),
                    }),
                };

                let symbol = self.symbols.insert(symbol_data);

                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Index(expr) => {
                let base = expr
                    .base()
                    .and_then(|base| self.add_expression(source, scope, base));

                let index = expr
                    .index()
                    .and_then(|index| self.add_expression(source, scope, index));

                let symbol = self.symbols.insert(SymbolData {
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Index(IndexSymbol { base, index }),
                });

                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Object(expr) => {
                let symbol_data = SymbolData {
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Object(ObjectSymbol {
                        fields: expr
                            .fields()
                            .filter_map(|field| match (field.property(), field.expr()) {
                                (Some(name), Some(expr)) => Some((
                                    name.text().to_string(),
                                    ObjectField {
                                        property_name: name.text().to_string(),
                                        property_syntax: SourceInfo {
                                            source: Some(source),
                                            text_range: name.text_range().into(),
                                            selection_text_range: None,
                                        },
                                        field_syntax: SourceInfo {
                                            source: Some(source),
                                            text_range: field.syntax().text_range().into(),
                                            selection_text_range: None,
                                        },
                                        value: self.add_expression(source, scope, expr),
                                    },
                                )),
                                _ => None,
                            })
                            .collect(),
                    }),
                };

                let symbol = self.symbols.insert(symbol_data);
                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Call(expr) => {
                let lhs = expr
                    .expr()
                    .and_then(|expr| self.add_expression(source, scope, expr));

                let symbol_data = SymbolData {
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Call(CallSymbol {
                        lhs,
                        arguments: match expr.arg_list() {
                            Some(arg_list) => arg_list
                                .arguments()
                                .filter_map(|expr| self.add_expression(source, scope, expr))
                                .collect(),
                            None => Vec::default(),
                        },
                    }),
                };

                let symbol = self.symbols.insert(symbol_data);
                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Closure(expr) => {
                let closure_scope = self.scopes.insert(ScopeData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    ..ScopeData::default()
                });

                if let Some(param_list) = expr.param_list() {
                    for param in param_list.params() {
                        let symbol = self.symbols.insert(SymbolData {
                            parent_scope: Scope::default(),
                            source: SourceInfo {
                                source: Some(source),
                                text_range: param.syntax().text_range().into(),
                                selection_text_range: None,
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

                        scope.add_symbol(self, symbol, false);
                    }
                }

                let closure_expr_symbol = expr
                    .body()
                    .and_then(|body| self.add_expression(source, closure_scope, body));

                let symbol = self.symbols.insert(SymbolData {
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Closure(ClosureSymbol {
                        scope,
                        expr: closure_expr_symbol,
                    }),
                });

                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::If(expr) => {
                let symbol = self.symbols.insert(SymbolData {
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::If(IfSymbol::default()),
                });

                // Here we flatten the branches of the `if` expression
                // from the recursive syntax tree.
                let mut next_branch = Some(expr);

                while let Some(branch) = next_branch.take() {
                    let branch_condition = branch
                        .expr()
                        .and_then(|expr| self.add_expression(source, scope, expr));

                    let then_scope = self.scopes.insert(ScopeData {
                        source: SourceInfo {
                            source: Some(source),
                            text_range: branch.then_branch().map(|body| body.syntax().text_range()),
                            selection_text_range: None,
                        },
                        ..ScopeData::default()
                    });

                    then_scope.set_parent(self, symbol);

                    if let Some(body) = branch.then_branch() {
                        self.add_statements(source, then_scope, body.statements());
                    }

                    self.symbol_mut(symbol)
                        .kind
                        .as_if_mut()
                        .unwrap()
                        .branches
                        .push((branch_condition, then_scope));

                    // trailing `else` branch
                    if let Some(else_body) = branch.else_branch() {
                        let then_scope = self.scopes.insert(ScopeData {
                            source: SourceInfo {
                                source: Some(source),
                                text_range: else_body.syntax().text_range().into(),
                                selection_text_range: None,
                            },
                            ..ScopeData::default()
                        });

                        then_scope.set_parent(self, symbol);
                        self.add_statements(source, then_scope, else_body.statements());
                        self.symbol_mut(symbol)
                            .kind
                            .as_if_mut()
                            .unwrap()
                            .branches
                            .push((None, then_scope));
                        break;
                    }

                    next_branch = branch.else_if_branch();
                }

                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Loop(expr) => {
                let loop_scope = self.scopes.insert(ScopeData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.loop_body().map(|body| body.syntax().text_range()),
                        selection_text_range: None,
                    },
                    ..ScopeData::default()
                });

                if let Some(body) = expr.loop_body() {
                    self.add_statements(source, loop_scope, body.statements());
                }

                let symbol = self.symbols.insert(SymbolData {
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Loop(LoopSymbol { scope: loop_scope }),
                });

                loop_scope.set_parent(self, symbol);

                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::For(expr) => {
                let for_scope = self.scopes.insert(ScopeData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.loop_body().map(|body| body.syntax().text_range()),
                        selection_text_range: None,
                    },
                    ..ScopeData::default()
                });

                if let Some(pat) = expr.pat() {
                    for ident in pat.idents() {
                        let ident_symbol = self.symbols.insert(SymbolData {
                            source: SourceInfo {
                                source: Some(source),
                                text_range: ident.text_range().into(),
                                selection_text_range: None,
                            },
                            parent_scope: Scope::default(),
                            kind: SymbolKind::Decl(Box::new(DeclSymbol {
                                name: ident.text().into(),
                                docs: String::new(),
                                is_pat: true,
                                ..DeclSymbol::default()
                            })),
                        });
                        scope.add_symbol(self, ident_symbol, false);
                    }
                }

                if let Some(body) = expr.loop_body() {
                    self.add_statements(source, for_scope, body.statements());
                }

                let sym = SymbolData {
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::For(ForSymbol {
                        iterable: expr
                            .iterable()
                            .and_then(|expr| self.add_expression(source, scope, expr)),
                        scope,
                    }),
                };

                let symbol = self.symbols.insert(sym);
                for_scope.set_parent(self, symbol);
                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::While(expr) => {
                let while_scope = self.scopes.insert(ScopeData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.loop_body().map(|body| body.syntax().text_range()),
                        selection_text_range: None,
                    },
                    ..ScopeData::default()
                });

                if let Some(body) = expr.loop_body() {
                    self.add_statements(source, while_scope, body.statements());
                }

                let symbol_data = SymbolData {
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::While(WhileSymbol {
                        scope: while_scope,
                        condition: expr
                            .expr()
                            .and_then(|expr| self.add_expression(source, scope, expr)),
                    }),
                };

                let symbol = self.symbols.insert(symbol_data);
                while_scope.set_parent(self, symbol);

                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Break(expr) => {
                let symbol_data = SymbolData {
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Break(BreakSymbol {
                        expr: expr
                            .expr()
                            .and_then(|expr| self.add_expression(source, scope, expr)),
                    }),
                };

                let symbol = self.symbols.insert(symbol_data);
                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Continue(expr) => {
                let symbol_data = SymbolData {
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Continue(ContinueSymbol {}),
                };

                let symbol = self.symbols.insert(symbol_data);
                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Switch(expr) => {
                let target = expr
                    .expr()
                    .and_then(|expr| self.add_expression(source, scope, expr));

                let arms = expr
                    .switch_arm_list()
                    .map(|arm_list| {
                        arm_list
                            .arms()
                            .map(|arm| {
                                let mut left = None;
                                let mut right = None;

                                if let Some(discard) = arm.discard_token() {
                                    left = Some(self.symbols.insert(SymbolData {
                                        source: SourceInfo {
                                            source: Some(source),
                                            text_range: discard.text_range().into(),
                                            selection_text_range: None,
                                        },
                                        parent_scope: Scope::default(),
                                        kind: SymbolKind::Discard(DiscardSymbol {}),
                                    }));
                                }

                                if let Some(expr) = arm.pattern_expr() {
                                    left = self.add_expression(source, scope, expr);
                                }

                                if let Some(expr) = arm.value_expr() {
                                    right = self.add_expression(source, scope, expr);
                                }

                                (left, right)
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                let symbol = self.symbols.insert(SymbolData {
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Switch(SwitchSymbol { target, arms }),
                });

                scope.add_symbol(self, symbol, true);
                Some(symbol)
            }
            Expr::Return(expr) => {
                let symbol_data = SymbolData {
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Return(ReturnSymbol {
                        expr: expr
                            .expr()
                            .and_then(|expr| self.add_expression(source, scope, expr)),
                    }),
                };

                let symbol = self.symbols.insert(symbol_data);
                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Fn(expr) => {
                let fn_scope = self.scopes.insert(ScopeData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    ..ScopeData::default()
                });

                if let Some(param_list) = expr.param_list() {
                    for param in param_list.params() {
                        let symbol = self.symbols.insert(SymbolData {
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

                        scope.add_symbol(self, symbol, false);
                    }
                }

                if let Some(body) = expr.body() {
                    self.add_statements(source, scope, body.statements());
                }
                let symbol = self.symbols.insert(SymbolData {
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
                        scope: fn_scope,
                        ..FnSymbol::default()
                    }),
                });

                scope.add_symbol(self, symbol, true);
                fn_scope.set_parent(self, symbol);
                Some(symbol)
            }
            Expr::Import(expr) => {
                let symbol_data = SymbolData {
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Import(ImportSymbol {
                        alias: expr.alias().map(|alias| {
                            self.symbols.insert(SymbolData {
                                source: SourceInfo {
                                    source: Some(source),
                                    text_range: alias.text_range().into(),
                                    selection_text_range: None,
                                },
                                kind: SymbolKind::Decl(Box::new(DeclSymbol {
                                    name: alias.text().into(),
                                    ..DeclSymbol::default()
                                })),
                                parent_scope: Scope::default(),
                            })
                        }),
                        expr: expr
                            .expr()
                            .and_then(|expr| self.add_expression(source, scope, expr)),
                    }),
                };

                let symbol = self.symbols.insert(symbol_data);

                scope.add_symbol(self, symbol, true);
                Some(symbol)
            }
            Expr::Export(expr) => {
                let target = expr.export_target().and_then(|target| match target {
                    ExportTarget::ExprLet(expr) => {
                        self.add_expression(source, scope, Expr::Let(expr))
                    }
                    ExportTarget::ExprConst(expr) => {
                        self.add_expression(source, scope, Expr::Const(expr))
                    }
                    ExportTarget::Ident(expr) => {
                        let symbol = self.symbols.insert(SymbolData {
                            source: SourceInfo {
                                source: Some(source),
                                text_range: expr.syntax().text_range().into(),
                                selection_text_range: expr.ident_token().map(|t| t.text_range()),
                            },
                            kind: SymbolKind::Reference(ReferenceSymbol {
                                name: expr
                                    .ident_token()
                                    .map(|s| s.text().to_string())
                                    .unwrap_or_default(),
                                ..ReferenceSymbol::default()
                            }),
                            parent_scope: Scope::default(),
                        });

                        scope.add_symbol(self, symbol, false);
                        Some(symbol)
                    }
                });

                let symbol = self.symbols.insert(SymbolData {
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Export(ExportSymbol { target }),
                });

                scope.add_symbol(self, symbol, false);

                Some(symbol)
            }
            Expr::Try(expr) => {
                let try_scope = self.scopes.insert(ScopeData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.try_block().map(|body| body.syntax().text_range()),
                        selection_text_range: None,
                    },
                    ..ScopeData::default()
                });

                if let Some(body) = expr.try_block() {
                    self.add_statements(source, try_scope, body.statements());
                }

                let catch_scope = self.scopes.insert(ScopeData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.catch_block().map(|body| body.syntax().text_range()),
                        selection_text_range: None,
                    },
                    ..ScopeData::default()
                });

                if let Some(catch_params) = expr.catch_params() {
                    for param in catch_params.params() {
                        let symbol = self.symbols.insert(SymbolData {
                            source: SourceInfo {
                                source: Some(source),
                                text_range: param.syntax().text_range().into(),
                                selection_text_range: None,
                            },
                            parent_scope: Scope::default(),
                            kind: SymbolKind::Decl(Box::new(DeclSymbol {
                                name: param
                                    .ident_token()
                                    .map(|s| s.text().to_string())
                                    .unwrap_or_default(),
                                is_param: true,
                                ..DeclSymbol::default()
                            })),
                        });

                        scope.add_symbol(self, symbol, false);
                    }
                }

                if let Some(body) = expr.catch_block() {
                    self.add_statements(source, catch_scope, body.statements());
                }

                let sym = SymbolData {
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Try(TrySymbol {
                        try_scope,
                        catch_scope,
                    }),
                };

                let symbol = self.symbols.insert(sym);
                try_scope.set_parent(self, symbol);
                catch_scope.set_parent(self, symbol);
                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
        }
    }
}
