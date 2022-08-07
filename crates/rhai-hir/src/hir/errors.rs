use crate::{
    error::{Error, ErrorKind},
    source::Source,
    symbol::SymbolKind,
    HashMap, Hir, Symbol,
};

impl Hir {
    #[must_use]
    pub fn errors(&self) -> Vec<Error> {
        let mut errors = Vec::new();

        for (symbol, _) in self.symbols() {
            self.collect_errors_from_symbol(symbol, &mut errors);
        }

        errors
    }

    #[must_use]
    pub fn errors_for_source(&self, source: Source) -> Vec<Error> {
        let mut errors = Vec::new();

        for (symbol, _) in self
            .symbols()
            .filter(|(_, symbol_data)| symbol_data.source.source == Some(source))
        {
            self.collect_errors_from_symbol(symbol, &mut errors);
        }

        errors
    }

    fn collect_errors_from_symbol(&self, symbol: Symbol, errors: &mut Vec<Error>) {
        if let Some(symbol_data) = self.symbol(symbol) {
            match &symbol_data.kind {
                SymbolKind::Reference(r) => {
                    if !r.field_access && r.target.is_none() && r.name != "this" {
                        errors.push(Error {
                            kind: ErrorKind::UnresolvedReference {
                                reference_symbol: symbol,
                                similar_name: self.find_similar_name(symbol, &r.name),
                            },
                        });
                    }
                }
                SymbolKind::Fn(f) => {
                    if f.is_def {
                        return;
                    }

                    let top_level = self
                        .modules
                        .iter()
                        .any(|(_, m)| m.scope == symbol_data.parent_scope);

                    if !top_level {
                        errors.push(Error {
                            kind: ErrorKind::NestedFunction { function: symbol },
                        });
                    }

                    let mut param_names: HashMap<&str, Symbol> = HashMap::new();

                    for &param in &self[f.scope].symbols {
                        let param_data = &self[param];

                        if !param_data.is_param() {
                            break;
                        }

                        if let Some(name) = param_data.name(self) {
                            if name != "_" {
                                if let Some(existing_param) = param_names.insert(name, param) {
                                    errors.push(Error {
                                        kind: ErrorKind::DuplicateFnParameter {
                                            duplicate_symbol: param,
                                            existing_symbol: existing_param,
                                        },
                                    });
                                }
                            }
                        }
                    }
                }
                SymbolKind::Import(import) => {
                    if import.target.is_none() {
                        errors.push(Error {
                            kind: ErrorKind::UnresolvedImport { import: symbol },
                        });
                    }
                }
                _ => {}
            }
        }
    }
}
