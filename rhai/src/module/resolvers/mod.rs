use crate::fn_native::SendSync;
use crate::{Engine, EvalAltResult, Module, Position, Shared, AST};
#[cfg(feature = "no_std")]
use std::prelude::v1::*;

mod dummy;
pub use dummy::DummyModuleResolver;

mod collection;
pub use collection::ModuleResolversCollection;

#[cfg(not(feature = "no_std"))]
#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
mod file;

#[cfg(not(feature = "no_std"))]
#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
pub use file::FileModuleResolver;

mod stat;
pub use stat::StaticModuleResolver;

/// Trait that encapsulates a module resolution service.
pub trait ModuleResolver: SendSync {
    /// Resolve a module based on a path string.
    fn resolve(
        &self,
        engine: &Engine,
        source_path: Option<&str>,
        path: &str,
        pos: Position,
    ) -> Result<Shared<Module>, Box<EvalAltResult>>;

    /// Resolve an `AST` based on a path string.
    ///
    /// Returns [`None`] (default) if such resolution is not supported
    /// (e.g. if the module is Rust-based).
    ///
    /// ## Low-Level API
    ///
    /// Override the default implementation of this method if the module resolver
    /// serves modules based on compiled Rhai scripts.
    #[allow(unused_variables)]
    #[must_use]
    fn resolve_ast(
        &self,
        engine: &Engine,
        source_path: Option<&str>,
        path: &str,
        pos: Position,
    ) -> Option<Result<AST, Box<EvalAltResult>>> {
        None
    }
}
