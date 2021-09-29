//! The `FnPtr` type.

use crate::token::is_valid_identifier;
use crate::{
    Dynamic, EvalAltResult, Identifier, NativeCallContext, Position, RhaiResult, StaticVec,
};
#[cfg(feature = "no_std")]
use std::prelude::v1::*;
use std::{
    convert::{TryFrom, TryInto},
    fmt, mem,
};

/// A general function pointer, which may carry additional (i.e. curried) argument values
/// to be passed onto a function during a call.
#[derive(Clone, Hash)]
pub struct FnPtr(Identifier, StaticVec<Dynamic>);

impl fmt::Debug for FnPtr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.is_curried() {
            write!(f, "Fn({})", self.fn_name())
        } else {
            f.debug_tuple("Fn").field(&self.0).field(&self.1).finish()
        }
    }
}

impl FnPtr {
    /// Create a new function pointer.
    #[inline(always)]
    pub fn new(name: impl Into<Identifier>) -> Result<Self, Box<EvalAltResult>> {
        name.into().try_into()
    }
    /// Create a new function pointer without checking its parameters.
    #[inline(always)]
    #[must_use]
    pub(crate) fn new_unchecked(name: Identifier, curry: StaticVec<Dynamic>) -> Self {
        Self(name, curry)
    }
    /// Get the name of the function.
    #[inline(always)]
    #[must_use]
    pub fn fn_name(&self) -> &str {
        self.fn_name_raw().as_ref()
    }
    /// Get the name of the function.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn fn_name_raw(&self) -> &Identifier {
        &self.0
    }
    /// Get the underlying data of the function pointer.
    #[inline(always)]
    #[must_use]
    pub(crate) fn take_data(self) -> (Identifier, StaticVec<Dynamic>) {
        (self.0, self.1)
    }
    /// Get the curried arguments.
    #[inline(always)]
    #[must_use]
    pub fn curry(&self) -> &[Dynamic] {
        self.1.as_ref()
    }
    /// Add a new curried argument.
    #[inline(always)]
    pub fn add_curry(&mut self, value: Dynamic) -> &mut Self {
        self.1.push(value);
        self
    }
    /// Set curried arguments to the function pointer.
    #[inline(always)]
    pub fn set_curry(&mut self, values: impl IntoIterator<Item = Dynamic>) -> &mut Self {
        self.1 = values.into_iter().collect();
        self
    }
    /// Is the function pointer curried?
    #[inline(always)]
    #[must_use]
    pub fn is_curried(&self) -> bool {
        !self.1.is_empty()
    }
    /// Get the number of curried arguments.
    #[inline(always)]
    #[must_use]
    pub fn num_curried(&self) -> usize {
        self.1.len()
    }
    /// Does the function pointer refer to an anonymous function?
    ///
    /// Not available under `no_function`.
    #[cfg(not(feature = "no_function"))]
    #[inline(always)]
    #[must_use]
    pub fn is_anonymous(&self) -> bool {
        self.0.starts_with(crate::engine::FN_ANONYMOUS)
    }
    /// Call the function pointer with curried arguments (if any).
    ///
    /// If this function is a script-defined function, it must not be marked private.
    ///
    /// # WARNING
    ///
    /// All the arguments are _consumed_, meaning that they're replaced by `()`.
    /// This is to avoid unnecessarily cloning the arguments.
    /// Do not use the arguments after this call. If they are needed afterwards,
    /// clone them _before_ calling this function.
    #[inline]
    pub fn call_dynamic(
        &self,
        ctx: &NativeCallContext,
        this_ptr: Option<&mut Dynamic>,
        mut arg_values: impl AsMut<[Dynamic]>,
    ) -> RhaiResult {
        let mut arg_values = arg_values.as_mut();
        let mut args_data;

        if self.num_curried() > 0 {
            args_data = StaticVec::with_capacity(self.num_curried() + arg_values.len());
            args_data.extend(self.curry().iter().cloned());
            args_data.extend(arg_values.iter_mut().map(mem::take));
            arg_values = args_data.as_mut();
        };

        let is_method = this_ptr.is_some();

        let mut args = StaticVec::with_capacity(arg_values.len() + 1);
        if let Some(obj) = this_ptr {
            args.push(obj);
        }
        args.extend(arg_values.iter_mut());

        ctx.call_fn_dynamic_raw(self.fn_name(), is_method, &mut args)
    }
}

impl fmt::Display for FnPtr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fn({})", self.0)
    }
}

impl TryFrom<Identifier> for FnPtr {
    type Error = Box<EvalAltResult>;

    #[inline]
    fn try_from(value: Identifier) -> Result<Self, Self::Error> {
        if is_valid_identifier(value.chars()) {
            Ok(Self(value, Default::default()))
        } else {
            EvalAltResult::ErrorFunctionNotFound(value.to_string(), Position::NONE).into()
        }
    }
}

#[cfg(not(feature = "no_smartstring"))]
impl TryFrom<crate::ImmutableString> for FnPtr {
    type Error = Box<EvalAltResult>;

    #[inline(always)]
    fn try_from(value: crate::ImmutableString) -> Result<Self, Self::Error> {
        let s: Identifier = value.into();
        Self::try_from(s)
    }
}

impl TryFrom<String> for FnPtr {
    type Error = Box<EvalAltResult>;

    #[inline(always)]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let s: Identifier = value.into();
        Self::try_from(s)
    }
}

impl TryFrom<&str> for FnPtr {
    type Error = Box<EvalAltResult>;

    #[inline(always)]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let s: Identifier = value.into();
        Self::try_from(s)
    }
}
