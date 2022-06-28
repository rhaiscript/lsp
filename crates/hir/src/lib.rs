#![warn(clippy::pedantic)]
#![allow(
    clippy::unused_async,
    clippy::single_match,
    clippy::wildcard_imports,
    clippy::too_many_lines,
    clippy::enum_glob_use,
    clippy::module_name_repetitions,
    clippy::single_match_else,
    clippy::default_trait_access,
    clippy::too_many_arguments
)]

pub mod error;
pub mod eval;
pub mod hir;
pub mod module;
pub mod scope;
pub mod source;
pub mod symbol;
pub mod ty;
pub(crate) mod util;

pub(crate) type IndexMap<K, V> = indexmap::IndexMap<K, V, ahash::RandomState>;
pub(crate) type IndexSet<V> = indexmap::IndexSet<V, ahash::RandomState>;
pub(crate) type HashSet<V> = ahash::AHashSet<V>;

pub use hir::Hir;
pub use module::Module;
pub use scope::Scope;
pub use symbol::Symbol;
pub use ty::Type;
