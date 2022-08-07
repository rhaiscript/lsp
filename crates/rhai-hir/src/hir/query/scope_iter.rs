use crate::scope::ScopeParent;
use core::iter;
use itertools::Either;
use rhai_rowan::TextSize;
use std::cmp::Ordering;

use super::*;

impl Hir {
    pub fn visible_symbols_from_symbol(&self, symbol: Symbol) -> impl Iterator<Item = Symbol> + '_ {
        VisibleSymbols {
            hir: self,
            scope: self[symbol].parent_scope,
            iter: Box::new(self.visible_scope_symbols_from(symbol)),
        }
    }

    pub fn visible_symbols_from_offset(
        &self,
        source: Source,
        offset: TextSize,
        inclusive: bool,
    ) -> impl Iterator<Item = Symbol> + '_ {
        match self.scope_at(source, offset, inclusive) {
            Some(scope) => Either::Left(VisibleSymbols {
                hir: self,
                scope,
                iter: Box::new(self.scope_symbols_from_offset(scope, offset)),
            }),
            None => Either::Right(iter::empty()),
        }
    }

    /// Same as [`Self::scope_symbols`], but includes all symbols in the tree.
    pub fn descendant_symbols(&self, scope: Scope) -> impl Iterator<Item = Symbol> + '_ {
        DescendantSymbols {
            hir: self,
            iter_stack: vec![Box::new(self.scope_symbols(scope))],
            iter: None,
        }
    }

    /// Iterate over all symbols in the scope.
    pub fn scope_symbols(&self, scope: Scope) -> impl Iterator<Item = Symbol> + '_ {
        let scope_data = &self[scope];

        scope_data
            .symbols
            .iter()
            .copied()
            .chain(scope_data.hoisted_symbols.iter().copied())
    }

    /// Iterate over all symbols in the scope in reverse order.
    pub fn scope_symbols_rev(&self, scope: Scope) -> impl Iterator<Item = Symbol> + '_ {
        let scope_data = &self[scope];

        scope_data
            .symbols
            .iter()
            .rev()
            .copied()
            .chain(scope_data.hoisted_symbols.iter().copied())
    }

    /// Filter symbols with unique name, to be used with [`unique_by`](itertools::Itertools::unique_by).
    #[must_use]
    pub fn unique_symbol_name(&self, symbol: &Symbol) -> NameOrSymbol {
        match self[*symbol].name(self) {
            Some(s) => NameOrSymbol::Name(s),
            None => NameOrSymbol::Symbol(*symbol),
        }
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
            .chain(scope_data.hoisted_symbols.iter().copied())
    }

    pub(crate) fn find_similar_name(&self, symbol: Symbol, name: &str) -> Option<String> {
        const MIN_DISTANCE: f64 = 0.5;

        self.visible_symbols_from_symbol(symbol)
            .filter_map(|symbol| self[symbol].name(self))
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
                            self.iter = Box::new(self.hir.scope_symbols_rev(parent_scope));
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

pub struct DescendantSymbols<'h> {
    hir: &'h Hir,
    iter_stack: Vec<Box<dyn Iterator<Item = Symbol> + 'h>>,
    iter: Option<Box<dyn Iterator<Item = Symbol> + 'h>>,
}

impl<'h> Iterator for DescendantSymbols<'h> {
    type Item = Symbol;

    fn next(&mut self) -> Option<Self::Item> {
        if self.iter.is_none() {
            self.iter = self.iter_stack.pop();
        }

        match &mut self.iter {
            Some(iter) => match iter.next() {
                None => {
                    if let Some(iter) = self.iter_stack.pop() {
                        self.iter = Some(iter);
                        self.next()
                    } else {
                        None
                    }
                }
                Some(sym) => {
                    collect_symbol_scope_iters(self.hir, &mut self.iter_stack, sym);
                    Some(sym)
                }
            },
            None => None,
        }
    }
}

fn collect_symbol_scope_iters<'h>(
    hir: &'h Hir,
    iters: &mut Vec<Box<dyn Iterator<Item = Symbol> + 'h>>,
    symbol: Symbol,
) {
    match &hir[symbol].kind {
        SymbolKind::Block(sym) => iters.push(Box::new(hir.scope_symbols(sym.scope))),
        SymbolKind::Fn(sym) => iters.push(Box::new(hir.scope_symbols(sym.scope))),
        SymbolKind::Decl(sym) => {
            if let Some(scope) = sym.value_scope {
                iters.push(Box::new(hir.scope_symbols(scope)));
            }
        }
        SymbolKind::Path(sym) => iters.push(Box::new(hir.scope_symbols(sym.scope))),
        SymbolKind::Unary(sym) => {
            if let Some(sym) = sym.rhs {
                collect_symbol_scope_iters(hir, iters, sym);
            }
        }
        SymbolKind::Binary(sym) => {
            if let Some(sym) = sym.lhs {
                collect_symbol_scope_iters(hir, iters, sym);
            }

            if let Some(sym) = sym.rhs {
                collect_symbol_scope_iters(hir, iters, sym);
            }
        }
        SymbolKind::Array(sym) => {
            for &value in &sym.values {
                collect_symbol_scope_iters(hir, iters, value);
            }
        }
        SymbolKind::Index(sym) => {
            if let Some(sym) = sym.base {
                collect_symbol_scope_iters(hir, iters, sym);
            }

            if let Some(sym) = sym.index {
                collect_symbol_scope_iters(hir, iters, sym);
            }
        }
        SymbolKind::Object(sym) => {
            for (_, field) in &sym.fields {
                if let Some(sym) = field.value {
                    collect_symbol_scope_iters(hir, iters, sym);
                }
            }
        }
        SymbolKind::Call(sym) => {
            if let Some(sym) = sym.lhs {
                collect_symbol_scope_iters(hir, iters, sym);
            }

            for &arg in &sym.arguments {
                collect_symbol_scope_iters(hir, iters, arg);
            }
        }
        SymbolKind::Closure(sym) => {
            iters.push(Box::new(hir.scope_symbols(sym.scope)));
        }
        SymbolKind::If(sym) => {
            for (condition, branch) in &sym.branches {
                if let Some(sym) = *condition {
                    collect_symbol_scope_iters(hir, iters, sym);
                }

                iters.push(Box::new(hir.scope_symbols(*branch)));
            }
        }
        SymbolKind::Loop(sym) => iters.push(Box::new(hir.scope_symbols(sym.scope))),
        SymbolKind::For(sym) => {
            if let Some(sym) = sym.iterable {
                collect_symbol_scope_iters(hir, iters, sym);
            }

            iters.push(Box::new(hir.scope_symbols(sym.scope)));
        }
        SymbolKind::While(sym) => {
            if let Some(sym) = sym.condition {
                collect_symbol_scope_iters(hir, iters, sym);
            }

            iters.push(Box::new(hir.scope_symbols(sym.scope)));
        }
        SymbolKind::Break(sym) => {
            if let Some(sym) = sym.expr {
                collect_symbol_scope_iters(hir, iters, sym);
            }
        }

        SymbolKind::Return(sym) => {
            if let Some(sym) = sym.expr {
                collect_symbol_scope_iters(hir, iters, sym);
            }
        }
        SymbolKind::Switch(sym) => {
            if let Some(sym) = sym.target {
                collect_symbol_scope_iters(hir, iters, sym);
            }

            for SwitchArm {
                pat_expr,
                condition_expr,
                value_expr,
            } in &sym.arms
            {
                if let Some(sym) = *pat_expr {
                    collect_symbol_scope_iters(hir, iters, sym);
                }

                if let Some(sym) = *condition_expr {
                    collect_symbol_scope_iters(hir, iters, sym);
                }

                if let Some(sym) = *value_expr {
                    collect_symbol_scope_iters(hir, iters, sym);
                }
            }
        }
        SymbolKind::Export(sym) => {
            if let Some(sym) = sym.target {
                collect_symbol_scope_iters(hir, iters, sym);
            }
        }
        SymbolKind::Try(sym) => {
            iters.push(Box::new(hir.scope_symbols(sym.try_scope)));
            iters.push(Box::new(hir.scope_symbols(sym.catch_scope)));
        }
        SymbolKind::Import(sym) => iters.push(Box::new(hir.scope_symbols(sym.scope))),
        SymbolKind::Throw(sym) => {
            if let Some(sym) = sym.expr {
                collect_symbol_scope_iters(hir, iters, sym);
            }
        }
        SymbolKind::Virtual(VirtualSymbol::Module(m)) => {
            iters.push(Box::new(hir.scope_symbols(hir[m.module].scope)));
        }
        SymbolKind::Lit(lit) => {
            for scope in lit.interpolated_scopes.iter().copied() {
                iters.push(Box::new(hir.scope_symbols(scope)));
            }
        }
        SymbolKind::Op(_)
        | SymbolKind::Reference(_)
        | SymbolKind::Continue(_)
        | SymbolKind::Discard(_)
        | SymbolKind::Virtual(VirtualSymbol::Proxy(..))
        | SymbolKind::TypeDecl(_) => {}
    }
}
