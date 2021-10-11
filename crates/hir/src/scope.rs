use crate::{syntax::SyntaxInfo, HashSet, IndexSet, Symbol};
use serde::{Deserialize, Serialize};

slotmap::new_key_type! { pub struct Scope; }

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ScopeData {
    pub syntax: Option<SyntaxInfo>,
    pub parent_symbol: Option<Symbol>,
    pub symbols: IndexSet<Symbol>,
    pub hoisted_symbols: HashSet<Symbol>,
}
