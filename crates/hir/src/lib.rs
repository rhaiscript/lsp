pub mod error;
pub mod eval;
pub mod hir;
pub mod module;
pub mod scope;
pub mod symbol;
pub mod syntax;
pub mod ty;

pub type IndexMap<K, V> = indexmap::IndexMap<K, V, ahash::RandomState>;
pub type IndexSet<V> = indexmap::IndexSet<V, ahash::RandomState>;
pub type HashMap<K, V> = ahash::AHashMap<K, V>;
pub type HashSet<V> = ahash::AHashSet<V>;

pub use hir::Hir;
pub use module::Module;
pub use scope::Scope;
pub use symbol::Symbol;
pub use ty::Type;
