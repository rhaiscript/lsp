use std::cmp::Ordering;

use rhai_rowan::{TextRange, TextSize};

use super::*;

// Nested ranges only.
fn range_scope(r1: &TextRange, r2: &TextRange) -> Ordering {
    if r1.start() < r2.start() || r1.end() > r2.end() {
        Ordering::Greater
    } else if r1.start() > r2.start() || r2.end() < r2.end() {
        Ordering::Less
    } else {
        Ordering::Equal
    }
}

impl Module {
    pub fn symbol_selection_at(&self, offset: TextSize, inclusive: bool) -> Option<Symbol> {
        self.symbols()
            .filter_map(|(sym, d)| {
                d.selection_syntax.and_then(|s| {
                    s.text_range.and_then(|range| {
                        if inclusive && range.contains_inclusive(offset) {
                            Some((sym, range))
                        } else if range.contains(offset) {
                            Some((sym, range))
                        } else {
                            None
                        }
                    })
                })
            })
            .min_by(|(_, r1), (_, r2)| range_scope(r1, r2))
            .map(|(s, _)| s)
    }

    pub fn symbol_at(&self, offset: TextSize, inclusive: bool) -> Option<Symbol> {
        self.symbols()
            .filter_map(|(sym, d)| {
                d.syntax.and_then(|s| {
                    s.text_range.and_then(|range| {
                        if inclusive && range.contains_inclusive(offset) {
                            Some((sym, range))
                        } else if range.contains(offset) {
                            Some((sym, range))
                        } else {
                            None
                        }
                    })
                })
            })
            .min_by(|(_, r1), (_, r2)| range_scope(r1, r2))
            .map(|(s, _)| s)
    }

    pub fn scope_at(&self, offset: TextSize, inclusive: bool) -> Option<Scope> {
        self.scopes()
            .filter_map(|(sym, d)| {
                d.syntax.and_then(|s| {
                    s.text_range.and_then(|range| {
                        if inclusive && range.contains_inclusive(offset) {
                            Some((sym, range))
                        } else if range.contains(offset) {
                            Some((sym, range))
                        } else {
                            None
                        }
                    })
                })
            })
            .min_by(|(_, r1), (_, r2)| range_scope(r1, r2))
            .map(|(s, _)| s)
    }
}

impl Module {
    pub fn visible_symbols_from_symbol<'m>(&'m self, symbol: Symbol) -> VisibleSymbols<'m> {
        VisibleSymbols {
            module: self,
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

    pub fn visible_symbols_from_offset<'m>(&'m self, offset: TextSize) -> VisibleSymbols<'m> {
        let scope = match self.scope_at(offset, false) {
            Some(s) => s,
            None => self.root_scope,
        };

        let scope_data = &self[scope];

        if let Some((index, symbol)) =
            scope_data.symbols.iter().enumerate().rev().find(|(_, &s)| {
                self[s]
                    .text_range()
                    .map(|range| range.end() <= offset)
                    .unwrap_or(false)
            })
        {
            return VisibleSymbols {
                module: self,
                last_symbol: Some(*symbol),
                scope_iter: Box::new(self.visible_scope_symbols_from_index(scope, index)),
            };
        }

        if let Some(symbol) = scope_data.parent_symbol {
            return VisibleSymbols {
                module: self,
                last_symbol: Some(symbol),
                scope_iter: Box::new(self.visible_scope_symbols_from(symbol)),
            };
        }

        VisibleSymbols {
            module: self,
            last_symbol: None,
            scope_iter: Box::new(scope_data.hoisted_symbols.iter().copied()),
        }
    }

    fn visible_scope_symbols_from<'m>(
        &'m self,
        symbol: Symbol,
    ) -> impl Iterator<Item = Symbol> + 'm {
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

    fn visible_scope_symbols_from_index<'m>(
        &'m self,
        scope: Scope,
        index: usize,
    ) -> impl Iterator<Item = Symbol> + 'm {
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

pub struct VisibleSymbols<'m> {
    module: &'m Module,
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
                .and_then(|symbol| self.module[self.module[symbol].parent_scope].parent_symbol)
            {
                Some(s) => {
                    self.scope_iter = Box::new(self.module.visible_scope_symbols_from(s));
                    self.next()
                }
                None => None,
            },
        }
    }
}
