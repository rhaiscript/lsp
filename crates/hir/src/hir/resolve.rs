use itertools::Itertools;
use url::Url;

use crate::{
    symbol::{ReferenceTarget, SymbolKind},
    Hir, Module, Symbol,
};

impl Hir {
    pub fn clear_references(&mut self) {
        let ref_symbols = self.symbols.iter_mut();

        for (_, sym_data) in ref_symbols {
            match &mut sym_data.kind {
                SymbolKind::Fn(f) => f.references.clear(),
                SymbolKind::Op(f) => f.references.clear(),
                SymbolKind::Decl(d) => {
                    d.target = None;
                    d.references.clear();
                }
                SymbolKind::Reference(r) => r.target = None,
                _ => {}
            }
        }
    }

    pub fn resolve_references(&mut self) {
        self.clear_references();

        // The ordering is important here,
        // e.g. paths already rely on submodules
        // to be resolved.
        self.resolve_imports();
        self.resolve_paths();
        self.resolve_same_file_references();
    }

    fn resolve_same_file_references(&mut self) {
        let ref_symbols_to_resolve: Vec<Symbol> = self
            .symbols
            .iter()
            .filter_map(|(s, data)| match &data.kind {
                SymbolKind::Reference(ref_data)
                    if !ref_data.part_of_path && !ref_data.field_access =>
                {
                    Some(s)
                }
                _ => None,
            })
            .collect();

        for ref_symbol in ref_symbols_to_resolve {
            let mut visible_symbols = self.visible_symbols_from_symbol(ref_symbol);

            while let Some(visible_symbol) = visible_symbols.next() {
                if self[visible_symbol].name() != self[ref_symbol].name() {
                    continue;
                }

                match &self[ref_symbol].kind {
                    SymbolKind::Reference(_) => {
                        if matches!(
                            &self[visible_symbol].kind,
                            SymbolKind::Fn(_) | SymbolKind::Decl(_)
                        ) {
                            drop(visible_symbols);
                            let vis_symbol_data = self.symbol_mut(visible_symbol);

                            match &mut vis_symbol_data.kind {
                                SymbolKind::Fn(target) => {
                                    target.references.insert(ref_symbol);
                                }
                                SymbolKind::Decl(target) => {
                                    target.references.insert(ref_symbol);
                                }
                                _ => {}
                            }

                            match &mut self.symbol_mut(ref_symbol).kind {
                                SymbolKind::Reference(r) => {
                                    r.target = Some(ReferenceTarget::Symbol(visible_symbol));
                                }
                                _ => {}
                            }

                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn resolve_imports(&mut self) {
        let import_symbols_to_resolve: Vec<Symbol> = self
            .symbols
            .iter()
            .filter_map(|(s, data)| match &data.kind {
                SymbolKind::Import(_) => Some(s),
                _ => None,
            })
            .collect();

        for import_symbol in import_symbols_to_resolve {
            let module = match self.module_by_symbol(import_symbol) {
                Some(m) => m,
                None => {
                    tracing::debug!("symbol has no module");
                    continue;
                }
            };

            if let Some(import_symbol) = self[import_symbol].kind.as_import() {
                if let Some(import_path) = import_symbol.import_path(self) {
                    let import_url = match self.resolve_import_url(self[module].url(), import_path)
                    {
                        Some(u) => u,
                        None => continue,
                    };

                    let target_module = match self.module_by_url(&import_url) {
                        Some(m) => m,
                        None => continue,
                    };

                    if let Some(alias) = import_symbol.alias {
                        if let Some(alias_decl) = self.symbol_mut(alias).kind.as_decl_mut() {
                            alias_decl.target = Some(ReferenceTarget::Module(target_module));
                        }
                    }
                }
            }
        }
    }

    fn resolve_paths(&mut self) {
        let path_symbols_to_resolve: Vec<Vec<Symbol>> = self
            .symbols
            .iter()
            .filter_map(|(_, data)| match &data.kind {
                SymbolKind::Path(p) => Some(p.segments.clone()),
                _ => None,
            })
            .collect();

        for path in path_symbols_to_resolve {
            let module_reference = match path.get(0) {
                Some(sym) => *sym,
                None => continue,
            };

            {
                let mut visible_symbols = self.visible_symbols_from_symbol(module_reference);

                while let Some(mut visible_symbol) = visible_symbols.next() {
                    match &self[module_reference].kind {
                        SymbolKind::Reference(_) => {
                            if matches!(&self[visible_symbol].kind, SymbolKind::Import(_)) {
                                let vis_symbol_data = &self[visible_symbol];

                                match &vis_symbol_data.kind {
                                    SymbolKind::Import(import) => {
                                        let import_alias = match import.alias {
                                            Some(sym) => sym,
                                            None => {
                                                continue;
                                            }
                                        };

                                        if self[import_alias].name()
                                            != self[module_reference].name()
                                        {
                                            continue;
                                        }

                                        visible_symbol = import_alias;
                                    }
                                    _ => {}
                                }

                                drop(visible_symbols);
                                match &mut self.symbol_mut(module_reference).kind {
                                    SymbolKind::Reference(r) => {
                                        r.target = Some(ReferenceTarget::Symbol(visible_symbol));
                                    }
                                    _ => {}
                                }

                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }

            for (m, segment) in path.into_iter().tuple_windows() {
                match self.target_module(m) {
                    Some(m) => {
                        self.resolve_in_module(m, segment);
                    }
                    None => break,
                }
            }
        }
    }

    fn resolve_in_module(&mut self, module: Module, ref_symbol: Symbol) {
        let target_symbol = {
            self.descendant_symbols(self[module].scope)
                .find(|&target_symbol| {
                    self[target_symbol].export
                        && self[target_symbol].name() == self[ref_symbol].name()
                })
        };

        if let Some(target_symbol) = target_symbol {
            let target_symbol_data = self.symbol_mut(target_symbol);

            match &mut target_symbol_data.kind {
                SymbolKind::Fn(target) => {
                    target.references.insert(ref_symbol);
                }
                SymbolKind::Decl(target) => {
                    target.references.insert(ref_symbol);
                }
                _ => {}
            }

            if let Some(r) = self.symbol_mut(ref_symbol).kind.as_reference_mut() {
                r.target = Some(ReferenceTarget::Symbol(target_symbol));
            }
        }
    }

    #[allow(clippy::unused_self)]
    pub(super) fn resolve_import_url(&self, from: Option<&Url>, import_path: &str) -> Option<Url> {
        if import_path.starts_with('.') {
            match from {
                Some(from) => match from.join(import_path) {
                    Ok(import) => Some(import),
                    Err(err) => {
                        tracing::debug!("{err}");
                        None
                    }
                },
                None => {
                    tracing::debug!("cannot resolve relative module with no base");
                    None
                }
            }
        } else {
            // TODO: delegate non-url string import paths.
            import_path.parse().ok()
        }
    }
}
