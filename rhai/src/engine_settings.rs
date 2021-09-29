//! Configuration settings for [`Engine`].

use crate::token::Token;
use crate::Engine;
use crate::{engine::Precedence, Identifier};
#[cfg(feature = "no_std")]
use std::prelude::v1::*;

#[cfg(not(feature = "unchecked"))]
use std::num::{NonZeroU64, NonZeroUsize};

impl Engine {
    /// Control whether and how the [`Engine`] will optimize an [`AST`][crate::AST] after compilation.
    ///
    /// Not available under `no_optimize`.
    #[cfg(not(feature = "no_optimize"))]
    #[inline(always)]
    pub fn set_optimization_level(
        &mut self,
        optimization_level: crate::OptimizationLevel,
    ) -> &mut Self {
        self.optimization_level = optimization_level;
        self
    }
    /// The current optimization level.
    /// It controls whether and how the [`Engine`] will optimize an [`AST`][crate::AST] after compilation.
    ///
    /// Not available under `no_optimize`.
    #[cfg(not(feature = "no_optimize"))]
    #[inline(always)]
    #[must_use]
    pub const fn optimization_level(&self) -> crate::OptimizationLevel {
        self.optimization_level
    }
    /// Set the maximum levels of function calls allowed for a script in order to avoid
    /// infinite recursion and stack overflows.
    ///
    /// Not available under `unchecked` or `no_function`.
    #[cfg(not(feature = "unchecked"))]
    #[cfg(not(feature = "no_function"))]
    #[inline(always)]
    pub fn set_max_call_levels(&mut self, levels: usize) -> &mut Self {
        self.limits.max_call_stack_depth = levels;
        self
    }
    /// The maximum levels of function calls allowed for a script.
    ///
    /// Not available under `unchecked` or `no_function`.
    #[cfg(not(feature = "unchecked"))]
    #[cfg(not(feature = "no_function"))]
    #[inline(always)]
    #[must_use]
    pub const fn max_call_levels(&self) -> usize {
        self.limits.max_call_stack_depth
    }
    /// Set the maximum number of operations allowed for a script to run to avoid
    /// consuming too much resources (0 for unlimited).
    ///
    /// Not available under `unchecked`.
    #[cfg(not(feature = "unchecked"))]
    #[inline(always)]
    pub fn set_max_operations(&mut self, operations: u64) -> &mut Self {
        self.limits.max_operations = NonZeroU64::new(operations);
        self
    }
    /// The maximum number of operations allowed for a script to run (0 for unlimited).
    ///
    /// Not available under `unchecked`.
    #[cfg(not(feature = "unchecked"))]
    #[inline(always)]
    #[must_use]
    pub const fn max_operations(&self) -> u64 {
        if let Some(n) = self.limits.max_operations {
            n.get()
        } else {
            0
        }
    }
    /// Set the maximum number of imported [modules][crate::Module] allowed for a script.
    ///
    /// Not available under `unchecked` or `no_module`.
    #[cfg(not(feature = "unchecked"))]
    #[cfg(not(feature = "no_module"))]
    #[inline(always)]
    pub fn set_max_modules(&mut self, modules: usize) -> &mut Self {
        self.limits.max_modules = modules;
        self
    }
    /// The maximum number of imported [modules][crate::Module] allowed for a script.
    ///
    /// Not available under `unchecked` or `no_module`.
    #[cfg(not(feature = "unchecked"))]
    #[cfg(not(feature = "no_module"))]
    #[inline(always)]
    #[must_use]
    pub const fn max_modules(&self) -> usize {
        self.limits.max_modules
    }
    /// Set the depth limits for expressions (0 for unlimited).
    ///
    /// Not available under `unchecked`.
    #[cfg(not(feature = "unchecked"))]
    #[inline(always)]
    pub fn set_max_expr_depths(
        &mut self,
        max_expr_depth: usize,
        #[cfg(not(feature = "no_function"))] max_function_expr_depth: usize,
    ) -> &mut Self {
        self.limits.max_expr_depth = NonZeroUsize::new(max_expr_depth);
        #[cfg(not(feature = "no_function"))]
        {
            self.limits.max_function_expr_depth = NonZeroUsize::new(max_function_expr_depth);
        }
        self
    }
    /// The depth limit for expressions (0 for unlimited).
    ///
    /// Not available under `unchecked`.
    #[cfg(not(feature = "unchecked"))]
    #[inline(always)]
    #[must_use]
    pub const fn max_expr_depth(&self) -> usize {
        if let Some(n) = self.limits.max_expr_depth {
            n.get()
        } else {
            0
        }
    }
    /// The depth limit for expressions in functions (0 for unlimited).
    ///
    /// Not available under `unchecked` or `no_function`.
    #[cfg(not(feature = "unchecked"))]
    #[cfg(not(feature = "no_function"))]
    #[inline(always)]
    #[must_use]
    pub const fn max_function_expr_depth(&self) -> usize {
        if let Some(n) = self.limits.max_function_expr_depth {
            n.get()
        } else {
            0
        }
    }
    /// Set the maximum length of [strings][crate::ImmutableString] (0 for unlimited).
    ///
    /// Not available under `unchecked`.
    #[cfg(not(feature = "unchecked"))]
    #[inline(always)]
    pub fn set_max_string_size(&mut self, max_size: usize) -> &mut Self {
        self.limits.max_string_size = NonZeroUsize::new(max_size);
        self
    }
    /// The maximum length of [strings][crate::ImmutableString] (0 for unlimited).
    ///
    /// Not available under `unchecked`.
    #[cfg(not(feature = "unchecked"))]
    #[inline(always)]
    #[must_use]
    pub const fn max_string_size(&self) -> usize {
        if let Some(n) = self.limits.max_string_size {
            n.get()
        } else {
            0
        }
    }
    /// Set the maximum length of [arrays][crate::Array] (0 for unlimited).
    ///
    /// Not available under `unchecked` or `no_index`.
    #[cfg(not(feature = "unchecked"))]
    #[cfg(not(feature = "no_index"))]
    #[inline(always)]
    pub fn set_max_array_size(&mut self, max_size: usize) -> &mut Self {
        self.limits.max_array_size = NonZeroUsize::new(max_size);
        self
    }
    /// The maximum length of [arrays][crate::Array] (0 for unlimited).
    ///
    /// Not available under `unchecked` or `no_index`.
    #[cfg(not(feature = "unchecked"))]
    #[cfg(not(feature = "no_index"))]
    #[inline(always)]
    #[must_use]
    pub const fn max_array_size(&self) -> usize {
        if let Some(n) = self.limits.max_array_size {
            n.get()
        } else {
            0
        }
    }
    /// Set the maximum size of [object maps][crate::Map] (0 for unlimited).
    ///
    /// Not available under `unchecked` or `no_object`.
    #[cfg(not(feature = "unchecked"))]
    #[cfg(not(feature = "no_object"))]
    #[inline(always)]
    pub fn set_max_map_size(&mut self, max_size: usize) -> &mut Self {
        self.limits.max_map_size = NonZeroUsize::new(max_size);
        self
    }
    /// The maximum size of [object maps][crate::Map] (0 for unlimited).
    ///
    /// Not available under `unchecked` or `no_object`.
    #[cfg(not(feature = "unchecked"))]
    #[cfg(not(feature = "no_object"))]
    #[inline(always)]
    #[must_use]
    pub const fn max_map_size(&self) -> usize {
        if let Some(n) = self.limits.max_map_size {
            n.get()
        } else {
            0
        }
    }
    /// Set the module resolution service used by the [`Engine`].
    ///
    /// Not available under `no_module`.
    #[cfg(not(feature = "no_module"))]
    #[inline(always)]
    pub fn set_module_resolver(
        &mut self,
        resolver: impl crate::ModuleResolver + 'static,
    ) -> &mut Self {
        self.module_resolver = Some(Box::new(resolver));
        self
    }
    /// Disable a particular keyword or operator in the language.
    ///
    /// # Examples
    ///
    /// The following will raise an error during parsing because the `if` keyword is disabled
    /// and is recognized as a reserved symbol!
    ///
    /// ```rust,should_panic
    /// # fn main() -> Result<(), rhai::ParseError> {
    /// use rhai::Engine;
    ///
    /// let mut engine = Engine::new();
    ///
    /// engine.disable_symbol("if");    // disable the 'if' keyword
    ///
    /// engine.compile("let x = if true { 42 } else { 0 };")?;
    /// //                      ^ 'if' is rejected as a reserved symbol
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// The following will raise an error during parsing because the `+=` operator is disabled.
    ///
    /// ```rust,should_panic
    /// # fn main() -> Result<(), rhai::ParseError> {
    /// use rhai::Engine;
    ///
    /// let mut engine = Engine::new();
    ///
    /// engine.disable_symbol("+=");    // disable the '+=' operator
    ///
    /// engine.compile("let x = 42; x += 1;")?;
    /// //                            ^ unknown operator
    /// # Ok(())
    /// # }
    /// ```
    #[inline(always)]
    pub fn disable_symbol(&mut self, symbol: impl Into<Identifier>) -> &mut Self {
        self.disabled_symbols.insert(symbol.into());
        self
    }
    /// Register a custom operator with a precedence into the language.
    ///
    /// The operator must be a valid identifier (i.e. it cannot be a symbol).
    ///
    /// The precedence cannot be zero.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Box<rhai::EvalAltResult>> {
    /// use rhai::Engine;
    ///
    /// let mut engine = Engine::new();
    ///
    /// // Register a custom operator called 'foo' and give it
    /// // a precedence of 160 (i.e. between +|- and *|/).
    /// engine.register_custom_operator("foo", 160).expect("should succeed");
    ///
    /// // Register a binary function named 'foo'
    /// engine.register_fn("foo", |x: i64, y: i64| (x * y) - (x + y));
    ///
    /// assert_eq!(
    ///     engine.eval_expression::<i64>("1 + 2 * 3 foo 4 - 5 / 6")?,
    ///     15
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn register_custom_operator(
        &mut self,
        keyword: impl AsRef<str> + Into<Identifier>,
        precedence: u8,
    ) -> Result<&mut Self, String> {
        let precedence = Precedence::new(precedence);

        if precedence.is_none() {
            return Err("precedence cannot be zero".into());
        }

        match Token::lookup_from_syntax(keyword.as_ref()) {
            // Standard identifiers, reserved keywords and custom keywords are OK
            None | Some(Token::Reserved(_)) | Some(Token::Custom(_)) => (),
            // Active standard keywords cannot be made custom
            // Disabled keywords are OK
            Some(token) if token.is_standard_keyword() => {
                if !self.disabled_symbols.contains(token.syntax().as_ref()) {
                    return Err(format!("'{}' is a reserved keyword", keyword.as_ref()));
                }
            }
            // Active standard symbols cannot be made custom
            Some(token) if token.is_standard_symbol() => {
                if !self.disabled_symbols.contains(token.syntax().as_ref()) {
                    return Err(format!("'{}' is a reserved operator", keyword.as_ref()));
                }
            }
            // Active standard symbols cannot be made custom
            Some(token) if !self.disabled_symbols.contains(token.syntax().as_ref()) => {
                return Err(format!("'{}' is a reserved symbol", keyword.as_ref()))
            }
            // Disabled symbols are OK
            Some(_) => (),
        }

        // Add to custom keywords
        self.custom_keywords.insert(keyword.into(), precedence);

        Ok(self)
    }
}
