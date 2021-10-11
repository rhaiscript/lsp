use crate::{scope::ScopeData, symbol::*, syntax::SyntaxInfo, Scope};
use core::ops;
use rhai_rowan::{
    ast::{AstNode, Expr, Rhai, Stmt},
    syntax::SyntaxNode,
};
use serde::{Deserialize, Serialize};
use slotmap::SlotMap;

mod edit;
mod remove;
mod query;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub root_scope: Scope,
    pub syntax: Option<SyntaxInfo>,
    pub scopes: SlotMap<Scope, ScopeData>,
    pub symbols: SlotMap<Symbol, SymbolData>,
}

impl ops::Index<Scope> for Module {
    type Output = ScopeData;

    fn index(&self, index: Scope) -> &Self::Output {
        self.scopes.get(index).unwrap()
    }
}

impl ops::Index<Symbol> for Module {
    type Output = SymbolData;

    fn index(&self, index: Symbol) -> &Self::Output {
        self.symbols.get(index).unwrap()
    }
}

impl Module {
    pub fn scopes(&self) -> impl Iterator<Item = (Scope, &ScopeData)> {
        self.scopes.iter()
    }

    pub fn contains_scope(&self, scope: Scope) -> bool {
        self.scopes.contains_key(scope)
    }

    pub fn scope_count(&self) -> usize {
        self.scopes.len()
    }

    pub fn symbols(&self) -> impl Iterator<Item = (Symbol, &SymbolData)> {
        self.symbols.iter()
    }

    pub fn contains_symbol(&self, symbol: Symbol) -> bool {
        self.symbols.contains_key(symbol)
    }

    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }
}
