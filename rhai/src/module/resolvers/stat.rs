use crate::{
    Engine, EvalAltResult, Identifier, Module, ModuleResolver, Position, Shared, SmartString,
};
#[cfg(feature = "no_std")]
use std::prelude::v1::*;
use std::{collections::BTreeMap, ops::AddAssign};

/// A static [module][Module] resolution service that serves [modules][Module] added into it.
///
/// # Example
///
/// ```
/// use rhai::{Engine, Module};
/// use rhai::module_resolvers::StaticModuleResolver;
///
/// let mut resolver = StaticModuleResolver::new();
///
/// let module = Module::new();
/// resolver.insert("hello", module);
///
/// let mut engine = Engine::new();
///
/// engine.set_module_resolver(resolver);
/// ```
#[derive(Debug, Clone, Default)]
pub struct StaticModuleResolver(BTreeMap<Identifier, Shared<Module>>);

impl StaticModuleResolver {
    /// Create a new [`StaticModuleResolver`].
    ///
    /// # Example
    ///
    /// ```
    /// use rhai::{Engine, Module};
    /// use rhai::module_resolvers::StaticModuleResolver;
    ///
    /// let mut resolver = StaticModuleResolver::new();
    ///
    /// let module = Module::new();
    /// resolver.insert("hello", module);
    ///
    /// let mut engine = Engine::new();
    /// engine.set_module_resolver(resolver);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Default::default()
    }
    /// Add a [module][Module] keyed by its path.
    #[inline(always)]
    pub fn insert(&mut self, path: impl Into<Identifier>, mut module: Module) {
        module.build_index();
        self.0.insert(path.into(), module.into());
    }
    /// Remove a [module][Module] given its path.
    #[inline(always)]
    pub fn remove(&mut self, path: &str) -> Option<Shared<Module>> {
        self.0.remove(path)
    }
    /// Does the path exist?
    #[inline(always)]
    #[must_use]
    pub fn contains_path(&self, path: &str) -> bool {
        self.0.contains_key(path)
    }
    /// Get an iterator of all the [modules][Module].
    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Shared<Module>)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v))
    }
    /// Get a mutable iterator of all the [modules][Module].
    #[inline(always)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&str, &mut Shared<Module>)> {
        self.0.iter_mut().map(|(k, v)| (k.as_str(), v))
    }
    /// Get an iterator of all the [module][Module] paths.
    #[inline(always)]
    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(|s| s.as_str())
    }
    /// Get an iterator of all the [modules][Module].
    #[inline(always)]
    pub fn values(&self) -> impl Iterator<Item = &Shared<Module>> {
        self.0.values()
    }
    /// Remove all [modules][Module].
    #[inline(always)]
    pub fn clear(&mut self) -> &mut Self {
        self.0.clear();
        self
    }
    /// Is this [`StaticModuleResolver`] empty?
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Get the number of [modules][Module] in this [`StaticModuleResolver`].
    #[inline(always)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Merge another [`StaticModuleResolver`] into this.
    /// The other [`StaticModuleResolver`] is consumed.
    ///
    /// Existing modules of the same path name are overwritten.
    #[inline(always)]
    pub fn merge(&mut self, other: Self) -> &mut Self {
        if !other.is_empty() {
            self.0.extend(other.0.into_iter());
        }
        self
    }
}

impl IntoIterator for StaticModuleResolver {
    type Item = (Identifier, Shared<Module>);
    type IntoIter = std::collections::btree_map::IntoIter<SmartString, Shared<Module>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl ModuleResolver for StaticModuleResolver {
    #[inline(always)]
    fn resolve(
        &self,
        _: &Engine,
        _: Option<&str>,
        path: &str,
        pos: Position,
    ) -> Result<Shared<Module>, Box<EvalAltResult>> {
        self.0
            .get(path)
            .cloned()
            .ok_or_else(|| EvalAltResult::ErrorModuleNotFound(path.into(), pos).into())
    }
}

impl AddAssign<Self> for StaticModuleResolver {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.merge(rhs);
    }
}
