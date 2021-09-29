use crate::{Engine, EvalAltResult, Identifier, Module, ModuleResolver, Position, Shared};
#[cfg(feature = "no_std")]
use std::prelude::v1::*;
use std::{
    collections::BTreeMap,
    io::Error as IoError,
    path::{Path, PathBuf},
};

pub const RHAI_SCRIPT_EXTENSION: &str = "rhai";

/// A [module][Module] resolution service that loads [module][Module] script files from the file system.
///
/// ## Caching
///
/// Resolved [Modules][Module] are cached internally so script files are not reloaded and recompiled
/// for subsequent requests.
///
/// Use [`clear_cache`][FileModuleResolver::clear_cache] or
/// [`clear_cache_for_path`][FileModuleResolver::clear_cache_for_path] to clear the internal cache.
///
/// ## Namespace
///
/// When a function within a script file module is called, all functions defined within the same
/// script are available, evan `private` ones.  In other words, functions defined in a module script
/// can always cross-call each other.
///
/// # Example
///
/// ```
/// use rhai::Engine;
/// use rhai::module_resolvers::FileModuleResolver;
///
/// // Create a new 'FileModuleResolver' loading scripts from the 'scripts' subdirectory
/// // with file extension '.x'.
/// let resolver = FileModuleResolver::new_with_path_and_extension("./scripts", "x");
///
/// let mut engine = Engine::new();
///
/// engine.set_module_resolver(resolver);
/// ```
#[derive(Debug)]
pub struct FileModuleResolver {
    base_path: Option<PathBuf>,
    extension: Identifier,
    cache_enabled: bool,

    #[cfg(not(feature = "sync"))]
    cache: std::cell::RefCell<BTreeMap<PathBuf, Shared<Module>>>,
    #[cfg(feature = "sync")]
    cache: std::sync::RwLock<BTreeMap<PathBuf, Shared<Module>>>,
}

impl Default for FileModuleResolver {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl FileModuleResolver {
    /// Create a new [`FileModuleResolver`] with the current directory as base path.
    ///
    /// The default extension is `.rhai`.
    ///
    /// # Example
    ///
    /// ```
    /// use rhai::Engine;
    /// use rhai::module_resolvers::FileModuleResolver;
    ///
    /// // Create a new 'FileModuleResolver' loading scripts from the current directory
    /// // with file extension '.rhai' (the default).
    /// let resolver = FileModuleResolver::new();
    ///
    /// let mut engine = Engine::new();
    /// engine.set_module_resolver(resolver);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self::new_with_extension(RHAI_SCRIPT_EXTENSION)
    }

    /// Create a new [`FileModuleResolver`] with a specific base path.
    ///
    /// The default extension is `.rhai`.
    ///
    /// # Example
    ///
    /// ```
    /// use rhai::Engine;
    /// use rhai::module_resolvers::FileModuleResolver;
    ///
    /// // Create a new 'FileModuleResolver' loading scripts from the 'scripts' subdirectory
    /// // with file extension '.rhai' (the default).
    /// let resolver = FileModuleResolver::new_with_path("./scripts");
    ///
    /// let mut engine = Engine::new();
    /// engine.set_module_resolver(resolver);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new_with_path(path: impl Into<PathBuf>) -> Self {
        Self::new_with_path_and_extension(path, RHAI_SCRIPT_EXTENSION)
    }

    /// Create a new [`FileModuleResolver`] with a file extension.
    ///
    /// # Example
    ///
    /// ```
    /// use rhai::Engine;
    /// use rhai::module_resolvers::FileModuleResolver;
    ///
    /// // Create a new 'FileModuleResolver' loading scripts with file extension '.rhai' (the default).
    /// let resolver = FileModuleResolver::new_with_extension("rhai");
    ///
    /// let mut engine = Engine::new();
    /// engine.set_module_resolver(resolver);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new_with_extension(extension: impl Into<Identifier>) -> Self {
        Self {
            base_path: None,
            extension: extension.into(),
            cache_enabled: true,
            cache: Default::default(),
        }
    }

    /// Create a new [`FileModuleResolver`] with a specific base path and file extension.
    ///
    /// # Example
    ///
    /// ```
    /// use rhai::Engine;
    /// use rhai::module_resolvers::FileModuleResolver;
    ///
    /// // Create a new 'FileModuleResolver' loading scripts from the 'scripts' subdirectory
    /// // with file extension '.x'.
    /// let resolver = FileModuleResolver::new_with_path_and_extension("./scripts", "x");
    ///
    /// let mut engine = Engine::new();
    /// engine.set_module_resolver(resolver);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new_with_path_and_extension(
        path: impl Into<PathBuf>,
        extension: impl Into<Identifier>,
    ) -> Self {
        Self {
            base_path: Some(path.into()),
            extension: extension.into(),
            cache_enabled: true,
            cache: Default::default(),
        }
    }

    /// Get the base path for script files.
    #[inline(always)]
    #[must_use]
    pub fn base_path(&self) -> Option<&Path> {
        self.base_path.as_ref().map(PathBuf::as_ref)
    }
    /// Set the base path for script files.
    #[inline(always)]
    pub fn set_base_path(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.base_path = Some(path.into());
        self
    }

    /// Get the script file extension.
    #[inline(always)]
    #[must_use]
    pub fn extension(&self) -> &str {
        &self.extension
    }

    /// Set the script file extension.
    #[inline(always)]
    pub fn set_extension(&mut self, extension: impl Into<Identifier>) -> &mut Self {
        self.extension = extension.into();
        self
    }

    /// Enable/disable the cache.
    #[inline(always)]
    pub fn enable_cache(&mut self, enable: bool) -> &mut Self {
        self.cache_enabled = enable;
        self
    }
    /// Is the cache enabled?
    #[inline(always)]
    #[must_use]
    pub fn is_cache_enabled(&self) -> bool {
        self.cache_enabled
    }

    /// Is a particular path cached?
    #[inline]
    #[must_use]
    pub fn is_cached(&self, path: &str, source_path: Option<&str>) -> bool {
        if !self.cache_enabled {
            return false;
        }

        let file_path = self.get_file_path(path, source_path);

        #[cfg(not(feature = "sync"))]
        return self.cache.borrow_mut().contains_key(&file_path);
        #[cfg(feature = "sync")]
        return self.cache.write().unwrap().contains_key(&file_path);
    }
    /// Empty the internal cache.
    #[inline(always)]
    pub fn clear_cache(&mut self) -> &mut Self {
        #[cfg(not(feature = "sync"))]
        self.cache.borrow_mut().clear();
        #[cfg(feature = "sync")]
        self.cache.write().unwrap().clear();

        self
    }
    /// Remove the specified path from internal cache.
    ///
    /// The next time this path is resolved, the script file will be loaded once again.
    #[inline]
    #[must_use]
    pub fn clear_cache_for_path(
        &mut self,
        path: &str,
        source_path: Option<&str>,
    ) -> Option<Shared<Module>> {
        let file_path = self.get_file_path(path, source_path);

        #[cfg(not(feature = "sync"))]
        return self
            .cache
            .borrow_mut()
            .remove_entry(&file_path)
            .map(|(_, v)| v);
        #[cfg(feature = "sync")]
        return self
            .cache
            .write()
            .unwrap()
            .remove_entry(&file_path)
            .map(|(_, v)| v);
    }
    /// Construct a full file path.
    #[must_use]
    fn get_file_path(&self, path: &str, source_path: Option<&str>) -> PathBuf {
        let path = Path::new(path);

        let mut file_path;

        if path.is_relative() {
            file_path = self
                .base_path
                .clone()
                .or_else(|| source_path.map(|p| p.into()))
                .unwrap_or_default();
            file_path.push(path);
        } else {
            file_path = path.into();
        }

        file_path.set_extension(self.extension.as_str()); // Force extension
        file_path
    }
}

impl ModuleResolver for FileModuleResolver {
    fn resolve(
        &self,
        engine: &Engine,
        source_path: Option<&str>,
        path: &str,
        pos: Position,
    ) -> Result<Shared<Module>, Box<EvalAltResult>> {
        // Load relative paths from source if there is no base path specified
        let source_path =
            source_path.and_then(|p| Path::new(p).parent().map(|p| p.to_string_lossy()));

        // Construct the script file path
        let file_path = self.get_file_path(path, source_path.as_ref().map(|p| p.as_ref()));

        // See if it is cached
        if self.is_cache_enabled() {
            #[cfg(not(feature = "sync"))]
            let c = self.cache.borrow();
            #[cfg(feature = "sync")]
            let c = self.cache.read().unwrap();

            if let Some(module) = c.get(&file_path) {
                return Ok(module.clone());
            }
        }

        // Load the script file and compile it
        let scope = Default::default();

        let mut ast = engine
            .compile_file(file_path.clone())
            .map_err(|err| match *err {
                EvalAltResult::ErrorSystem(_, err) if err.is::<IoError>() => {
                    Box::new(EvalAltResult::ErrorModuleNotFound(path.to_string(), pos))
                }
                _ => Box::new(EvalAltResult::ErrorInModule(path.to_string(), err, pos)),
            })?;

        ast.set_source(path);

        // Make a module from the AST
        let m: Shared<Module> = Module::eval_ast_as_new(scope, &ast, engine)
            .map_err(|err| Box::new(EvalAltResult::ErrorInModule(path.to_string(), err, pos)))?
            .into();

        // Put it into the cache
        if self.is_cache_enabled() {
            #[cfg(not(feature = "sync"))]
            self.cache.borrow_mut().insert(file_path, m.clone());
            #[cfg(feature = "sync")]
            self.cache.write().unwrap().insert(file_path, m.clone());
        }

        Ok(m)
    }

    /// Resolve an `AST` based on a path string.
    ///
    /// The file system is accessed during each call; the internal cache is by-passed.
    fn resolve_ast(
        &self,
        engine: &Engine,
        source_path: Option<&str>,
        path: &str,
        pos: Position,
    ) -> Option<Result<crate::AST, Box<EvalAltResult>>> {
        // Construct the script file path
        let file_path = self.get_file_path(path, source_path);

        // Load the script file and compile it
        Some(
            engine
                .compile_file(file_path)
                .map(|mut ast| {
                    ast.set_source(path);
                    ast
                })
                .map_err(|err| match *err {
                    EvalAltResult::ErrorSystem(_, err) if err.is::<IoError>() => {
                        EvalAltResult::ErrorModuleNotFound(path.to_string(), pos).into()
                    }
                    _ => EvalAltResult::ErrorInModule(path.to_string(), err, pos).into(),
                }),
        )
    }
}
