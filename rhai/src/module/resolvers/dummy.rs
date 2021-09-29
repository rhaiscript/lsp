use crate::{Engine, EvalAltResult, Module, ModuleResolver, Position, Shared};
#[cfg(feature = "no_std")]
use std::prelude::v1::*;

/// Empty/disabled [module][Module] resolution service that acts as a dummy.
///
/// # Example
///
/// ```
/// use rhai::{Engine, Module};
/// use rhai::module_resolvers::DummyModuleResolver;
///
/// let resolver = DummyModuleResolver::new();
/// let mut engine = Engine::new();
/// engine.set_module_resolver(resolver);
/// ```
#[derive(Debug, Copy, Eq, PartialEq, Clone, Default, Hash)]
pub struct DummyModuleResolver;

impl DummyModuleResolver {
    /// Create a new [`DummyModuleResolver`].
    ///
    /// # Example
    ///
    /// ```
    /// use rhai::{Engine, Module};
    /// use rhai::module_resolvers::DummyModuleResolver;
    ///
    /// let resolver = DummyModuleResolver::new();
    /// let mut engine = Engine::new();
    /// engine.set_module_resolver(resolver);
    /// ```
    #[inline(always)]
    pub const fn new() -> Self {
        Self
    }
}

impl ModuleResolver for DummyModuleResolver {
    #[inline(always)]
    fn resolve(
        &self,
        _: &Engine,
        _: Option<&str>,
        path: &str,
        pos: Position,
    ) -> Result<Shared<Module>, Box<EvalAltResult>> {
        EvalAltResult::ErrorModuleNotFound(path.into(), pos).into()
    }
}
