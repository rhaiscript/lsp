use crate::{eval::Value, source::SourceInfo};
use rhai_rowan::{
    ast::{ExportTarget, Expr, Item, Rhai, Stmt},
    parser::Parser,
    syntax::{SyntaxKind, SyntaxToken},
    TextSize,
};

use super::*;
use pulldown_cmark::{CodeBlockKind, Tag};
use std::mem;

impl Hir {
    pub(crate) fn add_script(&mut self, source: Source, rhai: &Rhai) {
        let url = self[source].url.clone();

        let module = self.ensure_module(ModuleKind::Url(url));
        self.module_mut(module).sources.insert(source);

        let script_docs = rhai.script_docs();

        if !script_docs.is_empty() {
            self.module_mut(module).docs = script_docs;
        }

        self.source_mut(source).module = module;

        self.add_statements(source, self[module].scope, true, rhai.statements());
    }

    fn add_statements(
        &mut self,
        source: Source,
        scope: Scope,
        can_export: bool,
        statements: impl Iterator<Item = Stmt>,
    ) {
        for statement in statements {
            self.add_statement(source, scope, can_export, statement);
        }
    }

    #[tracing::instrument(skip(self))]
    fn add_statement(
        &mut self,
        source: Source,
        scope: Scope,
        can_export: bool,
        stmt: Stmt,
    ) -> Option<Symbol> {
        stmt.item().and_then(|item| {
            item.expr()
                .and_then(|expr| self.add_expression(source, scope, can_export, expr))
        })
    }

    #[tracing::instrument(skip(self))]
    pub(super) fn add_expression(
        &mut self,
        source: Source,
        scope: Scope,
        can_export: bool,
        expr: Expr,
    ) -> Option<Symbol> {
        /// `let` or `const`
        fn add_decl(
            source: Source,
            hir: &mut Hir,
            scope: Scope,
            value: Option<Expr>,
            ident_syntax: Option<SyntaxToken>,
            syntax: &SyntaxNode,
            is_const: bool,
            export: bool,
            unknown_type: Type,
        ) -> Symbol {
            let (value, value_scope) = value
                .map(|expr| {
                    let scope = hir.add_scope(ScopeData {
                        source: SourceInfo {
                            source: Some(source),
                            text_range: expr.syntax().text_range().into(),
                            selection_text_range: None,
                        },
                        ..ScopeData::default()
                    });
                    (hir.add_expression(source, scope, false, expr), Some(scope))
                })
                .unwrap_or_default();

            let mut docs = String::new();
            if let Some(item) = syntax.ancestors().nth(2).and_then(Item::cast) {
                docs = item.docs_content();
            }

            let symbol = hir.add_symbol(SymbolData {
                export,
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
                    docs,
                    is_const,
                    value,
                    value_scope,
                    ..DeclSymbol::default()
                })),
                ty: unknown_type,
            });

            if let Some(value_scope) = value_scope {
                value_scope.set_parent(hir, symbol);
            }

            scope.add_symbol(hir, symbol, false);
            symbol
        }

        match expr {
            Expr::Ident(expr) => {
                let symbol = self.add_symbol(SymbolData {
                    export: can_export,
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: expr.ident_token().map(|t| t.text_range()),
                    },
                    kind: SymbolKind::Ref(ReferenceSymbol {
                        name: expr
                            .ident_token()
                            .map(|s| s.text().to_string())
                            .unwrap_or_default(),
                        ..ReferenceSymbol::default()
                    }),
                    parent_scope: Scope::default(),
                    ty: self.builtin_types.unknown,
                });

                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Path(expr_path) => {
                let segments = match expr_path.path() {
                    Some(p) => p.segments(),
                    None => return None,
                };

                let path_scope = self.add_scope(ScopeData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr_path.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    ..ScopeData::default()
                });

                let symbol = SymbolData {
                    export: false,
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr_path.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Path(PathSymbol {
                        scope: path_scope,
                        segments: segments
                            .map(|s| {
                                let symbol = self.add_symbol(SymbolData {
                                    export: can_export,
                                    source: SourceInfo {
                                        source: Some(source),
                                        text_range: s.text_range().into(),
                                        selection_text_range: s.text_range().into(),
                                    },
                                    parent_scope: Scope::default(),
                                    kind: SymbolKind::Ref(ReferenceSymbol {
                                        name: s.text().to_string(),
                                        part_of_path: true,
                                        ..ReferenceSymbol::default()
                                    }),
                                    ty: self.builtin_types.unknown,
                                });
                                path_scope.add_symbol(self, symbol, false);
                                symbol
                            })
                            .collect(),
                    }),
                    ty: self.builtin_types.unknown,
                };
                let sym = self.add_symbol(symbol);

                scope.add_symbol(self, sym, false);
                path_scope.set_parent(self, sym);
                Some(sym)
            }
            Expr::Lit(expr) => {
                let symbol = self.add_symbol(SymbolData {
                    export: false,
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Lit(LitSymbol {
                        value: expr.lit().map_or(Value::Unknown, value_of_lit),
                        interpolated_scopes: Vec::default(),
                    }),
                    ty: self.builtin_types.unknown,
                });

                if let Some(lit) = expr.lit().and_then(|l| l.lit_str_template()) {
                    let mut interpolated_scopes = Vec::new();
                    for interpolation in lit.interpolations() {
                        let interpolation_scope = self.add_scope(ScopeData {
                            source: SourceInfo {
                                source: Some(source),
                                text_range: interpolation.syntax().text_range().into(),
                                selection_text_range: None,
                            },
                            ..ScopeData::default()
                        });

                        interpolation_scope.set_parent(self, symbol);
                        self.add_statements(
                            source,
                            interpolation_scope,
                            false,
                            interpolation.statements(),
                        );
                        interpolated_scopes.push(interpolation_scope);
                    }

                    self.symbol_mut(symbol)
                        .kind
                        .as_lit_mut()
                        .unwrap()
                        .interpolated_scopes = interpolated_scopes;
                }

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
                can_export,
                self.builtin_types.unknown,
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
                can_export,
                self.builtin_types.unknown,
            )
            .into(),
            Expr::Block(expr) => {
                let block_scope = self.add_scope(ScopeData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    ..ScopeData::default()
                });

                let symbol = self.add_symbol(SymbolData {
                    export: false,
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Block(BlockSymbol { scope: block_scope }),
                    ty: self.builtin_types.unknown,
                });

                block_scope.set_parent(self, symbol);
                self.add_statements(source, block_scope, false, expr.statements());

                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Unary(expr) => {
                let rhs = expr
                    .expr()
                    .and_then(|rhs| self.add_expression(source, scope, false, rhs));

                let symbol = self.add_symbol(SymbolData {
                    export: false,
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Unary(UnarySymbol {
                        lookup_text: expr
                            .op_token()
                            .map(|t| t.text().trim().to_string())
                            .unwrap_or_default(),
                        op: expr.op_token().map(|t| t.kind()),
                        rhs,
                    }),
                    ty: self.builtin_types.unknown,
                });

                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Binary(expr) => {
                let binary_scope = self.add_scope(ScopeData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    ..Default::default()
                });

                let lhs = expr
                    .lhs()
                    .and_then(|lhs| self.add_expression(source, binary_scope, false, lhs));

                let rhs = expr
                    .rhs()
                    .and_then(|rhs| self.add_expression(source, binary_scope, false, rhs));

                let op = expr.op_token().map(|t| {
                    if t.kind() == SyntaxKind::IDENT {
                        BinaryOpKind::Custom(CustomBinaryOp {
                            name: t.text().to_string(),
                            range: t.text_range(),
                        })
                    } else {
                        BinaryOpKind::Regular(t.kind())
                    }
                });

                if let Some(BinaryOpKind::Regular(SyntaxKind::PUNCT_DOT)) = op {
                    if let Some(rhs) = rhs {
                        if let Some(ref_rhs) = self.symbol_mut(rhs).kind.as_reference_mut() {
                            ref_rhs.field_access = true;
                        }
                    }
                }

                let symbol = self.add_symbol(SymbolData {
                    export: false,
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Binary(BinarySymbol {
                        scope: binary_scope,
                        lookup_text: expr
                            .op_token()
                            .map(|t| t.text().trim().to_string())
                            .unwrap_or_default(),
                        lhs,
                        op,
                        rhs,
                    }),
                    ty: self.builtin_types.unknown,
                });
                binary_scope.set_parent(self, symbol);

                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Paren(expr) => expr
                .expr()
                .and_then(|expr| self.add_expression(source, scope, false, expr)),
            Expr::Array(expr) => {
                let symbol_data = SymbolData {
                    export: false,
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Array(ArraySymbol {
                        values: expr
                            .values()
                            .filter_map(|expr| self.add_expression(source, scope, false, expr))
                            .collect(),
                    }),
                    ty: self.builtin_types.unknown,
                };

                let symbol = self.add_symbol(symbol_data);

                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Index(expr) => {
                let base = expr
                    .base()
                    .and_then(|base| self.add_expression(source, scope, false, base));

                let index = expr
                    .index()
                    .and_then(|index| self.add_expression(source, scope, false, index));

                let symbol = self.add_symbol(SymbolData {
                    export: false,
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Index(IndexSymbol { base, index }),
                    ty: self.builtin_types.unknown,
                });

                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Object(expr) => {
                let symbol_data = SymbolData {
                    export: false,
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
                                        value: self.add_expression(source, scope, false, expr),
                                    },
                                )),
                                _ => None,
                            })
                            .collect(),
                    }),
                    ty: self.builtin_types.unknown,
                };

                let symbol = self.add_symbol(symbol_data);
                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Call(expr) => {
                let lhs = expr
                    .expr()
                    .and_then(|expr| self.add_expression(source, scope, false, expr));

                let symbol_data = SymbolData {
                    export: false,
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
                                .filter_map(|expr| self.add_expression(source, scope, false, expr))
                                .collect(),
                            None => Vec::default(),
                        },
                    }),
                    ty: self.builtin_types.unknown,
                };

                let symbol = self.add_symbol(symbol_data);
                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Closure(expr) => {
                let closure_scope = self.add_scope(ScopeData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    ..ScopeData::default()
                });

                if let Some(param_list) = expr.param_list() {
                    for param in param_list.params() {
                        let symbol = self.add_symbol(SymbolData {
                            export: false,
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
                            ty: self.builtin_types.unknown,
                        });

                        closure_scope.add_symbol(self, symbol, false);
                    }
                }

                let closure_expr_symbol = expr
                    .body()
                    .and_then(|body| self.add_expression(source, closure_scope, false, body));

                let symbol = self.add_symbol(SymbolData {
                    export: false,
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Closure(ClosureSymbol {
                        scope: closure_scope,
                        expr: closure_expr_symbol,
                    }),
                    ty: self.builtin_types.unknown,
                });

                scope.add_symbol(self, symbol, false);
                closure_scope.set_parent(self, symbol);

                Some(symbol)
            }
            Expr::If(expr) => {
                let symbol = self.add_symbol(SymbolData {
                    export: false,
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::If(IfSymbol::default()),
                    ty: self.builtin_types.unknown,
                });

                // Here we flatten the branches of the `if` expression
                // from the recursive syntax tree.
                let mut next_branch = Some(expr);

                while let Some(branch) = next_branch.take() {
                    let branch_condition = branch
                        .expr()
                        .and_then(|expr| self.add_expression(source, scope, false, expr));

                    let then_scope = self.add_scope(ScopeData {
                        source: SourceInfo {
                            source: Some(source),
                            text_range: branch.then_branch().map(|body| body.syntax().text_range()),
                            selection_text_range: None,
                        },
                        ..ScopeData::default()
                    });

                    then_scope.set_parent(self, symbol);

                    if let Some(body) = branch.then_branch() {
                        self.add_statements(source, then_scope, false, body.statements());
                    }

                    self.symbol_mut(symbol)
                        .kind
                        .as_if_mut()
                        .unwrap()
                        .branches
                        .push((branch_condition, then_scope));

                    // trailing `else` branch
                    if let Some(else_body) = branch.else_branch() {
                        let then_scope = self.add_scope(ScopeData {
                            source: SourceInfo {
                                source: Some(source),
                                text_range: else_body.syntax().text_range().into(),
                                selection_text_range: None,
                            },
                            ..ScopeData::default()
                        });

                        then_scope.set_parent(self, symbol);
                        self.add_statements(source, then_scope, false, else_body.statements());
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
                let loop_scope = self.add_scope(ScopeData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.loop_body().map(|body| body.syntax().text_range()),
                        selection_text_range: None,
                    },
                    ..ScopeData::default()
                });

                if let Some(body) = expr.loop_body() {
                    self.add_statements(source, loop_scope, false, body.statements());
                }

                let symbol = self.add_symbol(SymbolData {
                    export: false,
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Loop(LoopSymbol { scope: loop_scope }),
                    ty: self.builtin_types.unknown,
                });

                loop_scope.set_parent(self, symbol);

                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::For(expr) => {
                let for_scope = self.add_scope(ScopeData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.loop_body().map(|body| body.syntax().text_range()),
                        selection_text_range: None,
                    },
                    ..ScopeData::default()
                });

                if let Some(pat) = expr.pat() {
                    for ident in pat.idents() {
                        let ident_symbol = self.add_symbol(SymbolData {
                            export: false,
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
                            ty: self.builtin_types.unknown,
                        });
                        scope.add_symbol(self, ident_symbol, false);
                    }
                }

                if let Some(body) = expr.loop_body() {
                    self.add_statements(source, for_scope, false, body.statements());
                }

                let sym = SymbolData {
                    export: false,
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::For(ForSymbol {
                        cursor: expr
                            .iterable()
                            .and_then(|expr| self.add_expression(source, scope, false, expr)),
                        scope: for_scope,
                    }),
                    ty: self.builtin_types.unknown,
                };

                let symbol = self.add_symbol(sym);
                for_scope.set_parent(self, symbol);
                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::While(expr) => {
                let while_scope = self.add_scope(ScopeData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.loop_body().map(|body| body.syntax().text_range()),
                        selection_text_range: None,
                    },
                    ..ScopeData::default()
                });

                if let Some(body) = expr.loop_body() {
                    self.add_statements(source, while_scope, false, body.statements());
                }

                let symbol_data = SymbolData {
                    export: false,
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
                            .and_then(|expr| self.add_expression(source, scope, false, expr)),
                    }),
                    ty: self.builtin_types.unknown,
                };

                let symbol = self.add_symbol(symbol_data);
                while_scope.set_parent(self, symbol);

                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Break(expr) => {
                let symbol_data = SymbolData {
                    export: false,
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Break(BreakSymbol {
                        expr: expr
                            .expr()
                            .and_then(|expr| self.add_expression(source, scope, false, expr)),
                    }),
                    ty: self.builtin_types.unknown,
                };

                let symbol = self.add_symbol(symbol_data);
                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Continue(expr) => {
                let symbol_data = SymbolData {
                    export: false,
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Continue(ContinueSymbol {}),
                    ty: self.builtin_types.unknown,
                };

                let symbol = self.add_symbol(symbol_data);
                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Switch(expr) => {
                let target = expr
                    .expr()
                    .and_then(|expr| self.add_expression(source, scope, false, expr));

                let arms = expr
                    .switch_arm_list()
                    .map(|arm_list| {
                        arm_list
                            .arms()
                            .map(|arm| {
                                let condition = None;
                                let mut left = None;
                                let mut right = None;

                                if let Some(discard) = arm.discard_token() {
                                    let discard_symbol = self.add_symbol(SymbolData {
                                        export: false,
                                        source: SourceInfo {
                                            source: Some(source),
                                            text_range: discard.text_range().into(),
                                            selection_text_range: None,
                                        },
                                        parent_scope: Scope::default(),
                                        kind: SymbolKind::Discard(DiscardSymbol {}),
                                        ty: self.builtin_types.unknown,
                                    });

                                    scope.add_symbol(self, discard_symbol, false);

                                    left = Some(discard_symbol);
                                }

                                if let Some(expr) = arm.condition().and_then(|c| c.expr()) {
                                    left = self.add_expression(source, scope, false, expr);
                                }

                                if let Some(expr) = arm.pattern_expr() {
                                    left = self.add_expression(source, scope, false, expr);
                                }

                                if let Some(expr) = arm.value_expr() {
                                    right = self.add_expression(source, scope, false, expr);
                                }

                                SwitchArm {
                                    pat_expr: left,
                                    condition_expr: condition,
                                    value_expr: right,
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                let symbol = self.add_symbol(SymbolData {
                    export: false,
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Switch(SwitchSymbol { target, arms }),
                    ty: self.builtin_types.unknown,
                });

                scope.add_symbol(self, symbol, true);
                Some(symbol)
            }
            Expr::Return(expr) => {
                let symbol_data = SymbolData {
                    export: false,
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Return(ReturnSymbol {
                        expr: expr
                            .expr()
                            .and_then(|expr| self.add_expression(source, scope, false, expr)),
                    }),
                    ty: self.builtin_types.unknown,
                };

                let symbol = self.add_symbol(symbol_data);
                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Fn(expr) => {
                let fn_scope = self.add_scope(ScopeData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    ..ScopeData::default()
                });

                let mut docs = String::new();
                if let Some(fn_item) = expr.syntax().ancestors().nth(2).and_then(Item::cast) {
                    for (root, doc_def) in extract_doc_definitions(&fn_item) {
                        let def =
                            RhaiDef::cast(Parser::new(&doc_def).parse_def().into_syntax()).unwrap();

                        for stmt in def.statements() {
                            self.add_def_statement(
                                AddContext::default().with_root_offset(root),
                                source,
                                fn_scope,
                                &stmt,
                            );
                        }
                    }

                    // So that We have syntax highlight.
                    // FIXME: this replaces `rhai-scope` everywhere, not just code blocks.
                    docs = fn_item.docs_content().replace("rhai-scope", "rhai");
                }

                if let Some(param_list) = expr.param_list() {
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
                            ty: self.builtin_types.unknown,
                        });

                        fn_scope.add_symbol(self, symbol, false);
                    }
                }

                if let Some(body) = expr.body() {
                    self.add_statements(source, fn_scope, false, body.statements());
                }
                let symbol = self.add_symbol(SymbolData {
                    export: expr.kw_private_token().is_none() && can_export,
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
                        ..FnSymbol::default()
                    }),
                    ty: self.builtin_types.unknown,
                });

                scope.add_symbol(self, symbol, true);
                fn_scope.set_parent(self, symbol);
                Some(symbol)
            }
            Expr::Import(expr) => {
                let import_scope = self.add_scope(ScopeData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    ..ScopeData::default()
                });

                let symbol_data = SymbolData {
                    export: true,
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Import(ImportSymbol {
                        target: None,
                        scope: import_scope,
                        alias: expr.alias().map(|alias| {
                            let alias_symbol = self.add_symbol(SymbolData {
                                export: can_export,
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
                                ty: self.builtin_types.unknown,
                            });

                            import_scope.add_symbol(self, alias_symbol, false);

                            alias_symbol
                        }),
                        expr: expr.expr().and_then(|expr| {
                            self.add_expression(source, import_scope, false, expr)
                        }),
                    }),
                    ty: self.builtin_types.unknown,
                };

                let symbol = self.add_symbol(symbol_data);

                scope.add_symbol(self, symbol, false);
                import_scope.set_parent(self, symbol);

                Some(symbol)
            }
            Expr::Export(expr) => {
                let target = expr.export_target().and_then(|target| match target {
                    ExportTarget::ExprLet(expr) => {
                        self.add_expression(source, scope, can_export, Expr::Let(expr))
                    }
                    ExportTarget::ExprConst(expr) => {
                        self.add_expression(source, scope, can_export, Expr::Const(expr))
                    }
                    ExportTarget::Ident(expr) => {
                        let symbol = self.add_symbol(SymbolData {
                            export: can_export,
                            source: SourceInfo {
                                source: Some(source),
                                text_range: expr.syntax().text_range().into(),
                                selection_text_range: expr.ident_token().map(|t| t.text_range()),
                            },
                            kind: SymbolKind::Ref(ReferenceSymbol {
                                name: expr
                                    .ident_token()
                                    .map(|s| s.text().to_string())
                                    .unwrap_or_default(),
                                ..ReferenceSymbol::default()
                            }),
                            parent_scope: Scope::default(),
                            ty: self.builtin_types.unknown,
                        });

                        scope.add_symbol(self, symbol, false);
                        Some(symbol)
                    }
                });

                let symbol = self.add_symbol(SymbolData {
                    // The content is exported,
                    // but not the export symbol itself.
                    export: false,
                    parent_scope: Scope::default(),
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    kind: SymbolKind::Export(ExportSymbol { target }),
                    ty: self.builtin_types.unknown,
                });

                scope.add_symbol(self, symbol, false);

                Some(symbol)
            }
            Expr::Try(expr) => {
                let try_scope = self.add_scope(ScopeData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.try_block().map(|body| body.syntax().text_range()),
                        selection_text_range: None,
                    },
                    ..ScopeData::default()
                });

                if let Some(body) = expr.try_block() {
                    self.add_statements(source, try_scope, false, body.statements());
                }

                let catch_scope = self.add_scope(ScopeData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: expr.catch_block().map(|body| body.syntax().text_range()),
                        selection_text_range: None,
                    },
                    ..ScopeData::default()
                });

                if let Some(catch_params) = expr.catch_params() {
                    for param in catch_params.params() {
                        let symbol = self.add_symbol(SymbolData {
                            export: false,
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
                            ty: self.builtin_types.unknown,
                        });

                        scope.add_symbol(self, symbol, false);
                    }
                }

                if let Some(body) = expr.catch_block() {
                    self.add_statements(source, catch_scope, false, body.statements());
                }

                let sym = SymbolData {
                    export: false,
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
                    ty: self.builtin_types.unknown,
                };

                let symbol = self.add_symbol(sym);
                try_scope.set_parent(self, symbol);
                catch_scope.set_parent(self, symbol);
                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
            Expr::Throw(throw_expr) => {
                let expr = throw_expr
                    .expr()
                    .and_then(|e| self.add_expression(source, scope, false, e));

                let symbol = self.add_symbol(SymbolData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: throw_expr.syntax().text_range().into(),
                        selection_text_range: None,
                    },
                    parent_scope: Scope::default(),
                    export: false,
                    kind: SymbolKind::Throw(ThrowSymbol { expr }),
                    ty: self.builtin_types.unknown,
                });

                scope.add_symbol(self, symbol, false);
                Some(symbol)
            }
        }
    }
}

/// Definitions in doc comment blocks
#[allow(clippy::cast_possible_truncation)]
fn extract_doc_definitions(item: &Item) -> Vec<(TextSize, String)> {
    let mut definitions = Vec::new();
    for doc in item.docs() {
        let token = match doc.token() {
            Some(t) if t.kind() == SyntaxKind::COMMENT_BLOCK_DOC => t,
            _ => continue,
        };

        let mut def_code = String::from("module ;\n");
        let text = token.text();

        // strip /** */
        let text = &text[3..text.len() - 2];

        let doc_offset = token.text_range().start();
        let doc_offset = doc_offset.checked_add(3.into()).unwrap_or(doc_offset);
        let mut root_offset = doc_offset;

        let mut in_scope = false;
        for (event, range) in pulldown_cmark::Parser::new(text).into_offset_iter() {
            match event {
                pulldown_cmark::Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(c)))
                    if &*c == "rhai-scope" =>
                {
                    in_scope = true;
                }
                pulldown_cmark::Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(_))) => {
                    definitions.push((
                        root_offset
                            .checked_sub(("module ;\n".len() as u32).into())
                            .unwrap_or(root_offset),
                        mem::replace(&mut def_code, String::from("module ;\n")),
                    ));
                    root_offset = doc_offset;
                    in_scope = false;
                }
                pulldown_cmark::Event::Text(content) => {
                    root_offset = root_offset
                        .checked_add((range.start as u32).into())
                        .unwrap_or(root_offset);
                    if in_scope {
                        def_code += &*content;
                    }
                }
                _ => {}
            }
        }
    }

    definitions
}

impl Hir {
    pub(super) fn add_symbol(&mut self, data: SymbolData) -> Symbol {
        self.symbols.insert(data)
    }

    pub(super) fn add_scope(&mut self, data: ScopeData) -> Scope {
        self.scopes.insert(data)
    }
}
