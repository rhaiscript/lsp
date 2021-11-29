use std::cmp::Ordering;

use rhai_rowan::{TextRange, TextSize};

use crate::{
    error::{Error, ErrorKind},
    scope::ScopeParent,
};

use super::*;

// Nested ranges only.
fn range_scope(r1: TextRange, r2: TextRange) -> Ordering {
    if r1.start() < r2.start() || r1.end() > r2.end() {
        Ordering::Greater
    } else if r1.start() > r2.start() || r2.end() < r2.end() {
        Ordering::Less
    } else {
        Ordering::Equal
    }
}

impl Hir {
    #[must_use]
    pub fn symbol_selection_at(
        &self,
        source: Source,
        offset: TextSize,
        inclusive: bool,
    ) -> Option<Symbol> {
        self.symbols()
            .filter(|(_, d)| d.source.is(source))
            .filter_map(|(sym, d)| {
                d.source.selection_text_range.and_then(|range| {
                    if (inclusive && range.contains_inclusive(offset)) || range.contains(offset) {
                        Some((sym, range))
                    } else {
                        None
                    }
                })
            })
            .min_by(|(_, r1), (_, r2)| range_scope(*r1, *r2))
            .map(|(s, _)| s)
    }

    #[must_use]
    pub fn symbol_at(&self, source: Source, offset: TextSize, inclusive: bool) -> Option<Symbol> {
        self.symbols()
            .filter(|(_, d)| d.source.is(source))
            .filter_map(|(sym, d)| {
                d.source.text_range.and_then(|range| {
                    if (inclusive && range.contains_inclusive(offset)) || range.contains(offset) {
                        Some((sym, range))
                    } else {
                        None
                    }
                })
            })
            .min_by(|(_, r1), (_, r2)| range_scope(*r1, *r2))
            .map(|(s, _)| s)
    }

    #[must_use]
    pub fn scope_at(&self, source: Source, offset: TextSize, inclusive: bool) -> Option<Scope> {
        self.scopes()
            .filter(|(_, d)| d.source.is(source))
            .filter_map(|(sym, d)| {
                d.source.text_range.and_then(|range| {
                    if (inclusive && range.contains_inclusive(offset)) || range.contains(offset) {
                        Some((sym, range))
                    } else {
                        None
                    }
                })
            })
            .min_by(|(_, r1), (_, r2)| range_scope(*r1, *r2))
            .map(|(s, _)| s)
    }
}

impl Hir {
    #[must_use]
    pub fn visible_symbols_from_symbol(&self, symbol: Symbol) -> VisibleSymbols<'_> {
        VisibleSymbols {
            hir: self,
            scope: self[symbol].parent_scope,
            iter: Box::new(self.visible_scope_symbols_from(symbol)),
        }
    }

    #[must_use]
    pub fn visible_symbols_from_offset(
        &self,
        source: Source,
        offset: TextSize,
        inclusive: bool,
    ) -> VisibleSymbols<'_> {
        let scope = match self.scope_at(source, offset, inclusive) {
            Some(s) => s,
            None => self[self[source].module].scope,
        };

        VisibleSymbols {
            hir: self,
            scope,
            iter: Box::new(self.scope_symbols_from_offset(scope, offset)),
        }
    }

    fn scope_symbols(&self, scope: Scope) -> impl Iterator<Item = Symbol> + '_ {
        let scope_data = &self[scope];

        scope_data
            .symbols
            .iter()
            .rev()
            .copied()
            .chain(scope_data.hoisted_symbols.iter().copied())
    }

    fn scope_symbols_from_offset(
        &self,
        scope: Scope,
        offset: TextSize,
    ) -> impl Iterator<Item = Symbol> + '_ {
        let scope_data = &self[scope];

        scope_data
            .symbols
            .iter()
            .rev()
            .copied()
            .skip_while(move |&s| {
                self[s]
                    .source
                    .text_range
                    .map_or(false, |r| r.end() >= offset)
            })
            .chain(scope_data.hoisted_symbols.iter().copied())
    }

    fn visible_scope_symbols_from(&self, symbol: Symbol) -> impl Iterator<Item = Symbol> + '_ {
        let scope = self[symbol].parent_scope;
        let scope_data = &self[scope];

        let mut after_symbol = false;

        scope_data
            .symbols
            .iter()
            .rev()
            .skip_while(move |&&sym| {
                if sym == symbol {
                    after_symbol = true;
                    return true;
                }

                !after_symbol
            })
            .copied()
            .chain(
                scope_data
                    .hoisted_symbols
                    .iter()
                    .filter(move |s| **s != symbol)
                    .copied(),
            )
    }
}

impl Hir {
    #[must_use]
    pub fn collect_errors(&self) -> Vec<Error> {
        let mut errors = Vec::new();

        for (symbol, symbol_data) in self.symbols() {
            if let SymbolKind::Reference(r) = &symbol_data.kind {
                if r.target.is_none() {
                    errors.push(Error {
                        text_range: symbol_data.selection_or_text_range(),
                        kind: ErrorKind::UnresolvedReference {
                            reference_name: r.name.clone(),
                            reference_range: symbol_data.selection_or_text_range(),
                            reference_symbol: symbol,
                            similar_name: self.find_similar_name(symbol, &r.name),
                        },
                    });
                }
            }
        }

        errors
    }

    fn find_similar_name(&self, symbol: Symbol, name: &str) -> Option<String> {
        const MIN_DISTANCE: f64 = 0.5;

        self.visible_symbols_from_symbol(symbol)
            .filter_map(|symbol| self[symbol].name())
            .map(|visible_name| {
                (
                    strsim::normalized_damerau_levenshtein(name, visible_name),
                    visible_name,
                )
            })
            .max_by(|(distance_a, _), (distance_b, _)| {
                distance_a
                    .partial_cmp(distance_b)
                    .unwrap_or(Ordering::Equal)
            })
            .and_then(|(distance, name)| {
                if distance >= MIN_DISTANCE {
                    Some(name.to_string())
                } else {
                    None
                }
            })
    }
}

pub struct VisibleSymbols<'h> {
    hir: &'h Hir,
    scope: Scope,
    iter: Box<dyn Iterator<Item = Symbol> + 'h>,
}

impl<'h> Iterator for VisibleSymbols<'h> {
    type Item = Symbol;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .or_else(|| match self.hir[self.scope].parent {
                Some(parent) => {
                    match parent {
                        ScopeParent::Scope(parent_scope) => {
                            self.scope = parent_scope;
                            self.iter = Box::new(self.hir.scope_symbols(parent_scope));
                        }
                        ScopeParent::Symbol(parent_symbol) => {
                            self.scope = self.hir[parent_symbol].parent_scope;
                            self.iter =
                                Box::new(self.hir.visible_scope_symbols_from(parent_symbol));
                        }
                    };
                    self.next()
                }
                _ => None,
            })
    }
}
