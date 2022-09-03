use rhai_rowan::{parser, util::is_valid_ident, TextRange, TextSize};
use std::cmp::Ordering;

use super::*;

pub mod modules;
pub mod scope_iter;
pub mod types;

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
            .or_else(|| self.module_by_source(source).map(|m| self[m].scope))
    }

    #[must_use]
    pub fn source_by_url(&self, url: &Url) -> Option<Source> {
        for (src, data) in self.sources.iter() {
            if data.url == *url {
                return Some(src);
            }
        }

        None
    }

    pub fn operators(&self) -> impl Iterator<Item = &OpSymbol> + '_ {
        self.symbols.values().filter_map(|v| v.kind.as_op())
    }

    /// Return the custom operators for parsing.
    pub fn parser_operators(&self) -> impl Iterator<Item = (String, parser::Operator)> + '_ {
        self.operators().filter_map(|op| {
            if is_valid_ident(&op.name) {
                Some((
                    op.name.clone(),
                    parser::Operator {
                        binding_power: op.binding_powers,
                    },
                ))
            } else {
                None
            }
        })
    }

    #[must_use]
    pub fn operator_by_name(&self, name: &str) -> Option<&OpSymbol> {
        self.operators().find(|&op| op.name == name)
    }
}

/// Used for filtering shadowed symbols.
///
/// This way symbols with the same name are filtered,
/// all other symbols remain unique.
#[doc(hidden)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum NameOrSymbol<'s> {
    Symbol(Symbol),
    Name(&'s str),
}
