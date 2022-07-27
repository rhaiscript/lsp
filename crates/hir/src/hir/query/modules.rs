use crate::scope::ScopeParent;

use super::*;

impl Hir {
    #[must_use]
    pub fn module_by_url(&self, url: &Url) -> Option<Module> {
        self.modules.iter().find_map(|(m, data)| {
            data.url()
                .and_then(|module_url| if module_url == url { Some(m) } else { None })
        })
    }

    #[must_use]
    pub fn module_by_symbol(&self, symbol: Symbol) -> Option<Module> {
        self.module_of_scope(self[symbol].parent_scope)
    }

    #[must_use]
    pub fn module_by_source(&self, source: Source) -> Option<Module> {
        let m = self[source].module;

        if m.is_null() {
            None
        } else {
            Some(m)
        }
    }

    #[must_use]
    pub fn module_of_scope(&self, mut scope: Scope) -> Option<Module> {
        loop {
            if let Some(m) =
                self.modules.iter().find_map(
                    |(m, m_data)| {
                        if m_data.scope == scope {
                            Some(m)
                        } else {
                            None
                        }
                    },
                )
            {
                return Some(m);
            }

            match self[scope].parent {
                Some(parent) => match parent {
                    ScopeParent::Scope(s) => {
                        scope = s;
                        continue;
                    }
                    ScopeParent::Symbol(sym) => {
                        scope = self[sym].parent_scope;
                    }
                },
                None => return None,
            }
        }
    }

    /// All the missing modules that appear in imports.
    #[must_use]
    pub fn missing_modules(&self) -> impl ExactSizeIterator<Item = Url> {
        let mut missing = Vec::new();

        for (_, data) in &self.symbols {
            if let SymbolKind::Import(import) = &data.kind {
                if let Some(import_path) = import.import_path(self) {
                    if let Some(module_url) = self
                        .resolve_import_url(data.source.source.map(|s| &self[s].url), import_path)
                    {
                        if !self
                            .modules
                            .iter()
                            .any(|(_, m)| m.url().map_or(false, |url| *url == module_url))
                        {
                            missing.push(module_url);
                        }
                    }
                }
            }
        }

        missing.into_iter()
    }

    /// Resolve a symbol in a module.
    #[must_use]
    pub fn find_in_module(&self, module: Module, name: &str) -> Option<Symbol> {
        self.scope_symbols(self[module].scope)
            .filter(|s| self[*s].export)
            .find(|s| self[*s].name(self) == Some(name))
    }

    /// Recursively resolve a module from a reference.
    #[must_use]
    pub fn target_module(&self, reference_symbol: Symbol) -> Option<Module> {
        let mut reference_symbol = Some(reference_symbol);

        while let Some(ref_symbol) = reference_symbol.take() {
            match self[ref_symbol].target() {
                Some(target) => match target {
                    ReferenceTarget::Symbol(sym) => {
                        reference_symbol = Some(sym);
                    }
                    ReferenceTarget::Module(m) => return Some(m),
                },
                None => {
                    return None;
                }
            }
        }

        None
    }
}
