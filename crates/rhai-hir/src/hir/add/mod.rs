use super::*;
use crate::{
    eval::Value,
    module::{ModuleKind, STATIC_URL_SCHEME},
    scope::ScopeParent,
    source::SourceKind,
    TypeKind,
};
use rhai_rowan::{
    ast::{AstNode, Lit, Rhai, RhaiDef},
    syntax::SyntaxKind,
    util::unescape,
    TextRange, TextSize,
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
                protected: true,
                sources: Default::default(),
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

    pub(crate) fn ensure_builtin_types(&mut self) {
        // If any of them is not null, it has been
        // initialized.
        if !self.builtin_types.is_uninit() {
            return;
        }

        self.builtin_types = BuiltinTypes {
            module: self.types.insert(TypeData {
                kind: TypeKind::Module,
                ..TypeData::default()
            }),
            int: self.types.insert(TypeData {
                kind: TypeKind::Int,
                ..TypeData::default()
            }),
            float: self.types.insert(TypeData {
                kind: TypeKind::Float,
                ..TypeData::default()
            }),
            bool: self.types.insert(TypeData {
                kind: TypeKind::Bool,
                ..TypeData::default()
            }),
            char: self.types.insert(TypeData {
                kind: TypeKind::Char,
                ..TypeData::default()
            }),
            string: self.types.insert(TypeData {
                kind: TypeKind::String,
                ..TypeData::default()
            }),
            timestamp: self.types.insert(TypeData {
                kind: TypeKind::Timestamp,
                ..TypeData::default()
            }),
            void: self.types.insert(TypeData {
                kind: TypeKind::Void,
                ..TypeData::default()
            }),
            unknown: self.types.insert(TypeData {
                kind: TypeKind::Unknown,
                ..TypeData::default()
            }),
            never: self.types.insert(TypeData {
                kind: TypeKind::Never,
                ..TypeData::default()
            }),
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
                        protected: false,
                        sources: Default::default(),
                        docs: String::new(),
                    })
                }),
            ModuleKind::Inline => unreachable!(),
        }
    }

    pub(crate) fn add_module_to_static_scope(&mut self, module: Module) {
        match &self[module].kind {
            ModuleKind::Static | ModuleKind::Inline => {
                // Inserting the root static module makes no sense,
                // and while inline modules can in fact be part of the static scope,
                // they are never inserted via this function.
                unreachable!()
            }
            ModuleKind::Url(url) => {
                if url.scheme() != STATIC_URL_SCHEME {
                    return;
                }

                let name = match url.host_str() {
                    Some(name) => name,
                    _ => unreachable!(),
                };

                for static_symbol in self[self[self.static_module].scope].iter_symbols() {
                    if let SymbolKind::Virtual(VirtualSymbol::Module(m)) = &self[static_symbol].kind
                    {
                        if m.module == module {
                            return;
                        }
                    }
                }

                let name = name.to_string();

                let virt_module_symbol = self.add_symbol(SymbolData {
                    source: Default::default(),
                    parent_scope: Scope::default(),
                    kind: SymbolKind::Virtual(VirtualSymbol::Module(VirtualModuleSymbol {
                        name,
                        module,
                    })),
                    export: true,
                    ty: self.builtin_types.unknown,
                });

                self[self.static_module]
                    .scope
                    .add_symbol(self, virt_module_symbol, true);
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
        let parent = parent.into();

        if cfg!(debug_assertions) {
            match parent {
                ScopeParent::Scope(s) => {
                    assert_ne!(s, self, "scope cannot be the parent of itself");
                }
                ScopeParent::Symbol(_) => {}
            }
        }

        let s = hir.scope_mut(self);
        debug_assert!(s.parent.is_none());
        s.parent = Some(parent);
    }
}

/// Contextual information during insertion
/// into the hir.
#[derive(Debug, Default, Clone, Copy)]
struct AddContext {
    /// Override the root offset of syntax spans.
    ///
    /// Used for modifying the spans of definitions
    /// in comments.
    root_offset_override: Option<TextSize>,
}

impl AddContext {
    fn with_root_offset(mut self, offset: TextSize) -> Self {
        self.root_offset_override = self
            .root_offset_override
            .map_or(Some(offset), |o| o.checked_add(offset));
        self
    }

    fn text_range(self, range: impl Into<Option<TextRange>>) -> Option<TextRange> {
        range.into().map(|range| {
            if let Some(root) = self.root_offset_override {
                TextRange::new(
                    range
                        .start()
                        .checked_add(root)
                        .unwrap_or_else(|| range.start()),
                    range.end().checked_add(root).unwrap_or_else(|| range.end()),
                )
            } else {
                range
            }
        })
    }
}

#[allow(clippy::needless_pass_by_value)]
fn value_of_lit(lit: Lit) -> Value {
    if let Some(lit) = lit.lit_token() {
        match lit.kind() {
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
            SyntaxKind::LIT_STR => {
                let mut text = lit.text();

                if text.starts_with('"') {
                    text = text
                        .strip_prefix('"')
                        .unwrap_or(text)
                        .strip_suffix('"')
                        .unwrap_or(text);

                    Value::String(unescape(text, '"').0)
                } else {
                    text = text
                        .strip_prefix('`')
                        .unwrap_or(text)
                        .strip_suffix('`')
                        .unwrap_or(text);
                    Value::String(unescape(text, '`').0)
                }
            }
            SyntaxKind::LIT_CHAR => {
                let mut text = lit.text();
                text = text
                    .strip_prefix('\'')
                    .unwrap_or(text)
                    .strip_suffix('\'')
                    .unwrap_or(text);

                Value::Char(
                    // FIXME: this allocates a string.
                    unescape(text, '\'').0.chars().next().unwrap_or('💩'),
                )
            }
            _ => Value::Unknown,
        }
    } else {
        // It's a string template literal
        // FIXME: we know its content if it has
        // no code interpolations.
        Value::String(String::new())
    }
}
