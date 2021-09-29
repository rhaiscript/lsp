//! Module defining macros for developing _plugins_.

use crate::fn_call::FnCallArgs;
pub use crate::fn_native::CallableFunction;
pub use crate::{
    Dynamic, Engine, EvalAltResult, FnAccess, FnNamespace, ImmutableString, Module,
    NativeCallContext, Position,
};
#[cfg(feature = "no_std")]
use std::prelude::v1::*;
pub use std::{any::TypeId, mem};
pub type RhaiResult = Result<Dynamic, Box<EvalAltResult>>;

#[cfg(not(features = "no_module"))]
pub use rhai_codegen::*;
#[cfg(features = "no_module")]
pub use rhai_codegen::{export_fn, register_exported_fn};

/// Trait implemented by a _plugin function_.
///
/// This trait should not be used directly.
/// Use the `#[export_module]` and `#[export_fn]` procedural attributes instead.
pub trait PluginFunction {
    /// Call the plugin function with the arguments provided.
    fn call(&self, context: NativeCallContext, args: &mut FnCallArgs) -> RhaiResult;

    /// Is this plugin function a method?
    #[must_use]
    fn is_method_call(&self) -> bool;
}
