use core::iter;
use std::cmp::Ordering;

use itertools::Either;
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
            .or_else(|| self.module_by_source(source).map(|m| self[m].scope))
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

    /// Iterate over all symbols in the scope and its sub-scopes.
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
        match self[*symbol].name() {
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
}

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

    #[must_use]
    pub fn source_by_url(&self, url: &Url) -> Option<Source> {
        for (src, data) in self.sources.iter() {
            if data.url == *url {
                return Some(src);
            }
        }

        None
    }

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
            .find(|s| self[*s].name() == Some(name))
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

    fn collect_errors_from_symbol(&self, symbol: Symbol, errors: &mut Vec<Error>) {
        if let Some(symbol_data) = self.symbol(symbol) {
            if let SymbolKind::Reference(r) = &symbol_data.kind {
                if !r.field_access && r.target.is_none() && r.name != "this" {
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

            for (pat, arm) in &sym.arms {
                if let Some(sym) = *pat {
                    collect_symbol_scope_iters(hir, iters, sym);
                }

                if let Some(sym) = *arm {
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
        SymbolKind::Op(_)
        | SymbolKind::Lit(_)
        | SymbolKind::Reference(_)
        | SymbolKind::Continue(_)
        | SymbolKind::Discard(_) => {}
    }
}
