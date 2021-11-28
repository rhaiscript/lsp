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
            .filter(|(_, d)| d.source.is_part_of(source))
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
            .filter(|(_, d)| d.source.is_part_of(source))
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
            .filter(|(_, d)| d.source.is_part_of(source))
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
            last_symbol: Some(symbol),
            scope_iter: Box::new(self.visible_scope_symbols_from(symbol)),
        }
    }

    /// Finds symbols by name, useful for debugging and tests.
    pub fn symbols_by_name<'m>(&'m self, name: &'m str) -> impl Iterator<Item = Symbol> + 'm {
        self.symbols()
            .filter_map(move |(s, data)| match data.name() {
                Some(n) if n == name => Some(s),
                _ => None,
            })
    }

    #[must_use]
    pub fn visible_symbols_from_offset(
        &self,
        source: Source,
        offset: TextSize,
    ) -> VisibleSymbols<'_> {
        let scope = match self.scope_at(source, offset, false) {
            Some(s) => s,
            None => self[self[source].module].scope,
        };

        let scope_data = &self[scope];

        if let Some((index, symbol)) =
            scope_data.symbols.iter().enumerate().rev().find(|(_, &s)| {
                self[s]
                    .text_range()
                    .map_or(false, |range| range.end() <= offset)
            })
        {
            return VisibleSymbols {
                hir: self,
                last_symbol: Some(*symbol),
                scope_iter: Box::new(self.visible_scope_symbols_from_index(scope, index)),
            };
        }

        if let Some(parent) = scope_data.parent {
            match parent {
                ScopeParent::Scope(parent_scope) => {
                    return VisibleSymbols {
                        hir: self,
                        last_symbol: None,
                        scope_iter: Box::new(self.scope_symbols(parent_scope)),
                    }
                }
                ScopeParent::Symbol(parent_symbol) => {
                    return VisibleSymbols {
                        hir: self,
                        last_symbol: Some(parent_symbol),
                        scope_iter: Box::new(self.visible_scope_symbols_from(parent_symbol)),
                    };
                }
            }
        }

        VisibleSymbols {
            hir: self,
            last_symbol: None,
            scope_iter: Box::new(scope_data.hoisted_symbols.iter().copied()),
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
            .filter(move |&&sym| matches!(&self[sym].kind, SymbolKind::Decl(_) | SymbolKind::Fn(_)))
            .copied()
            .chain(
                scope_data
                    .hoisted_symbols
                    .iter()
                    .filter(move |s| **s != symbol)
                    .copied(),
            )
    }

    fn visible_scope_symbols_from_index(
        &self,
        scope: Scope,
        index: usize,
    ) -> impl Iterator<Item = Symbol> + '_ {
        let scope_data = &self[scope];

        scope_data
            .symbols
            .iter()
            .enumerate()
            .rev()
            .skip_while(move |(i, _)| *i > index)
            .filter_map(move |(_, &sym)| {
                if matches!(&self[sym].kind, SymbolKind::Decl(_) | SymbolKind::Fn(_)) {
                    Some(sym)
                } else {
                    None
                }
            })
            .chain(scope_data.hoisted_symbols.iter().copied())
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

pub struct VisibleSymbols<'m> {
    hir: &'m Hir,
    last_symbol: Option<Symbol>,
    scope_iter: Box<dyn Iterator<Item = Symbol> + 'm>,
}

impl<'m> Iterator for VisibleSymbols<'m> {
    type Item = Symbol;

    fn next(&mut self) -> Option<Self::Item> {
        match self.scope_iter.next() {
            s @ Some(last_symbol) => {
                self.last_symbol = Some(last_symbol);
                s
            }
            None => match self
                .last_symbol
                .take()
                .and_then(|symbol| self.hir[self.hir[symbol].parent_scope].parent)
            {
                Some(parent) => match parent {
                    ScopeParent::Scope(parent_scope) => {
                        self.scope_iter = Box::new(self.hir.scope_symbols(parent_scope));
                        self.next()
                    }
                    ScopeParent::Symbol(parent_symbol) => {
                        self.scope_iter =
                            Box::new(self.hir.visible_scope_symbols_from(parent_symbol));
                        self.next()
                    }
                },
                None => None,
            },
        }
    }
}
