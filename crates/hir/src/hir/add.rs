use super::*;
use crate::{
    module::{ModuleKind, STATIC_URL_SCHEME},
    scope::ScopeParent,
    source::SourceKind,
    Type,
};
use rhai_rowan::{
    ast::{AstNode, DefStmt, Rhai, RhaiDef},
    parser::{parsers::def::parse_def_stmt, Parser},
};

mod def;
mod script;

impl Hir {
    pub fn add_source(&mut self, url: &Url, syntax: &SyntaxNode) {
        if let Some(s) = self.source_of(url) {
            self.remove_source(s);
        }

        if let Some(rhai) = Rhai::cast(syntax.clone()) {
            let source = self.sources.insert(SourceData {
                kind: SourceKind::Script,
                url: url.clone(),
                module: Module::null(),
            });

            self.add_script(source, &rhai);
        }

        if let Some(def) = RhaiDef::cast(syntax.clone()) {
            let source = self.sources.insert(SourceData {
                kind: SourceKind::Def,
                url: url.clone(),
                module: Module::null(),
            });

            self.add_def(source, &def);
        }
    }
}

impl Hir {
    pub(crate) fn ensure_static_module(&mut self) {
        if self.static_module.is_null() {
            let scope = self.scopes.insert(ScopeData::default());
            self.static_module = self.modules.insert(ModuleData {
                scope,
                kind: ModuleKind::Static,
                docs: String::new(),
            });
        }
    }

    pub(crate) fn ensure_virtual_source(&mut self) {
        if self.virtual_source.is_null() {
            let source = self.sources.insert(SourceData {
                url: "rhai-virtual:///".parse().unwrap(),
                kind: SourceKind::Def,
                module: self.static_module,
            });
            self.virtual_source = source;
        }
    }

    fn ensure_module(&mut self, kind: ModuleKind) -> Module {
        match &kind {
            ModuleKind::Static => self.static_module,
            ModuleKind::Url(_) => self
                .modules
                .iter()
                .find_map(|(m, data)| if data.kind == kind { Some(m) } else { None })
                .unwrap_or_else(|| {
                    let scope = self.scopes.insert(ScopeData {
                        parent: Some(ScopeParent::Scope(self[self.static_module].scope)),
                        ..ScopeData::default()
                    });
                    self.modules.insert(ModuleData {
                        scope,
                        kind,
                        docs: String::new(),
                    })
                }),
        }
    }

    pub(crate) fn add_module_to_static_scope(&mut self, module: Module) {
        match &self[module].kind {
            ModuleKind::Static => {
                tracing::debug!("cannot insert static module");
            }
            ModuleKind::Url(url) => {
                if url.scheme() != STATIC_URL_SCHEME {
                    return;
                }

                let name = match url.host_str() {
                    Some(name) => name,
                    _ => return,
                };

                let import_src = format!(r#"import "{url}" as {name}"#);

                let mut parser = Parser::new(&import_src);
                parser.execute(parse_def_stmt);

                self.add_def_statement(
                    self.virtual_source,
                    self[self.static_module].scope,
                    &DefStmt::cast(parser.finish().into_syntax()).unwrap(),
                );
            }
        }
    }
}

impl Scope {
    pub(crate) fn add_symbol(self, hir: &mut Hir, symbol: Symbol, hoist: bool) {
        assert!(!self.is_null(), "the scope cannot be null");
        assert!(!symbol.is_null(), "the provided symbol cannot be null");
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

        tracing::trace!(
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
