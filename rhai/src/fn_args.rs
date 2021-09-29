//! Helper module which defines [`FuncArgs`] to make function calling easier.

#![cfg(not(feature = "no_function"))]
#![allow(non_snake_case)]

use crate::dynamic::Variant;
use crate::Dynamic;
#[cfg(feature = "no_std")]
use std::prelude::v1::*;

/// Trait that parses arguments to a function call.
///
/// Any data type can implement this trait in order to pass arguments to a function call.
pub trait FuncArgs {
    /// Parse function call arguments into a container.
    ///
    /// # Example
    ///
    /// ```
    /// use rhai::{Engine, Dynamic, FuncArgs, Scope};
    ///
    /// // A struct containing function arguments
    /// struct Options {
    ///     pub foo: bool,
    ///     pub bar: String,
    ///     pub baz: i64,
    /// }
    ///
    /// impl FuncArgs for Options {
    ///     fn parse<CONTAINER: Extend<Dynamic>>(self, container: &mut CONTAINER) {
    ///         container.extend(std::iter::once(self.foo.into()));
    ///         container.extend(std::iter::once(self.bar.into()));
    ///         container.extend(std::iter::once(self.baz.into()));
    ///     }
    /// }
    ///
    /// # fn main() -> Result<(), Box<rhai::EvalAltResult>> {
    /// let options = Options { foo: false, bar: "world".to_string(), baz: 42 };
    ///
    /// let engine = Engine::new();
    /// let mut scope = Scope::new();
    ///
    /// let ast = engine.compile(
    /// "
    ///     fn hello(x, y, z) {
    ///         if x { `hello ${y}` } else { y + z }
    ///     }
    /// ")?;
    ///
    /// let result: String = engine.call_fn(&mut scope, &ast, "hello", options)?;
    ///
    /// assert_eq!(result, "world42");
    /// # Ok(())
    /// # }
    /// ```
    fn parse<CONTAINER: Extend<Dynamic>>(self, container: &mut CONTAINER);
}

impl<T: Variant + Clone> FuncArgs for Vec<T> {
    #[inline(always)]
    fn parse<CONTAINER: Extend<Dynamic>>(self, container: &mut CONTAINER) {
        container.extend(self.into_iter().map(Variant::into_dynamic));
    }
}

/// Macro to implement [`FuncArgs`] for tuples of standard types (each can be
/// converted into a [`Dynamic`]).
macro_rules! impl_args {
    ($($p:ident),*) => {
        impl<$($p: Variant + Clone),*> FuncArgs for ($($p,)*)
        {
            #[inline(always)]
            #[allow(unused_variables)]
            fn parse<CONTAINER: Extend<Dynamic>>(self, container: &mut CONTAINER) {
                let ($($p,)*) = self;
                $(container.extend(Some($p.into_dynamic()));)*
            }
        }

        impl_args!(@pop $($p),*);
    };
    (@pop) => {
    };
    (@pop $head:ident) => {
        impl_args!();
    };
    (@pop $head:ident $(, $tail:ident)+) => {
        impl_args!($($tail),*);
    };
}

impl_args!(A, B, C, D, E, F, G, H, J, K, L, M, N, P, Q, R, S, T, U, V);
