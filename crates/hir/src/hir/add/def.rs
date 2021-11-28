use super::*;
use crate::{module::ModuleKind, source::SourceInfo, Type};
use rhai_rowan::{
    ast::{AstNode, Def, DefStmt, RhaiDef},
    syntax::SyntaxElement,
    T,
};

impl Hir {
    pub(super) fn add_def(&mut self, source: Source, def: &RhaiDef) {
        let def_mod = match def.def_module_decl() {
            Some(d) => d,
            None => return,
        };

        // TODO: module docs
        // let docs = def_mod.docs_content();

        let def_mod = match def_mod.def_module() {
            Some(d) => d,
            None => return,
        };

        let module_kind = if def_mod.kw_static_token().is_some() {
            if def_mod.lit_str_token().is_some() {
                // TODO: error
            }

            if let Some(name) = def_mod.ident_token() {
                ModuleKind::StaticNamespaced(name.text().into())
            } else {
                ModuleKind::Static
            }
        } else if let Some(_name) = def_mod.lit_str_token() {
            // TODO: parse string literals properly
            // ModuleKind::Dynamic(name.text().trim_matches('"').trim_matches('\'').into())
            // TODO: turn module name into url
            todo!()
        } else {
            // TODO: error
            return;
        };

        let module = self.ensure_module(module_kind);
        self.source_mut(source).module = module;

        for stmt in def.statements() {
            self.add_def_statement(source, self[module].scope, &stmt);
        }
    }

    fn add_def_statement(&mut self, source: Source, scope: Scope, stmt: &DefStmt) {
        let def = match stmt.item().and_then(|it| it.def()) {
            Some(d) => d,
            None => return,
        };

        let docs = stmt.item().map(|it| it.docs_content()).unwrap_or_default();

        match def {
            Def::Import(import_def) => {
                if let Some(alias) = import_def.alias() {
                    let symbol = self.symbols.insert(SymbolData {
                        source: SourceInfo {
                            source: Some(source),
                            text_range: Some(import_def.syntax().text_range()),
                            ..SourceInfo::default()
                        },
                        parent_scope: Scope::default(),
                        kind: SymbolKind::Decl(Box::new(DeclSymbol {
                            name: alias.text().into(),
                            is_import: true,
                            ty: Type::Module,
                            docs,
                            ..DeclSymbol::default()
                        })),
                    });

                    scope.add_symbol(self, symbol, true);
                }
            }
            Def::Const(const_def) => {
                let ident_token = match const_def.ident_token() {
                    Some(s) => s,
                    None => return,
                };

                let symbol = self.symbols.insert(SymbolData {
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
            Def::Fn(f) => {
                let ident = match f.ident_token() {
                    Some(i) => i,
                    None => return,
                };

                let symbol = self.symbols.insert(SymbolData {
                    source: SourceInfo {
                        source: Some(source),
                        text_range: Some(f.syntax().text_range()),
                        selection_text_range: Some(ident.text_range()),
                    },
                    parent_scope: Scope::default(),
                    kind: SymbolKind::Fn(FnSymbol {
                        name: ident.text().into(),
                        scope: Scope::default(),
                        docs,
                        ..FnSymbol::default()
                    }),
                });

                scope.add_symbol(self, symbol, true);
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
                    source: SourceInfo {
                        source: Some(source),
                        text_range: Some(f.syntax().text_range()),
                        selection_text_range: Some(ident.text_range()),
                    },
                    parent_scope: Scope::default(),
                    kind: SymbolKind::Op(OpSymbol {
                        name: ident.text().into(),
                        docs,
                        ..OpSymbol::default()
                    }),
                });

                scope.add_symbol(self, symbol, true);
            }
            Def::Type(_) => {
                // TODO
            }
        }
    }
}
