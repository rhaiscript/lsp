//! Module which defines the function registration mechanism.

#![allow(non_snake_case)]

use crate::dynamic::{DynamicWriteLock, Variant};
use crate::fn_call::FnCallArgs;
use crate::fn_native::{CallableFunction, FnAny, SendSync};
use crate::r#unsafe::unsafe_try_cast;
use crate::token::Position;
use crate::{Dynamic, EvalAltResult, NativeCallContext};
#[cfg(feature = "no_std")]
use std::prelude::v1::*;
use std::{any::TypeId, mem};

// These types are used to build a unique _marker_ tuple type for each combination
// of function parameter types in order to make each trait implementation unique.
// That is because stable Rust currently does not allow distinguishing implementations
// based purely on parameter types of traits (`Fn`, `FnOnce` and `FnMut`).
//
// For example:
//
// `NativeFunction<(Mut<A>, B, Ref<C>), R>`
//
// will have the function prototype constraint to:
//
// `FN: (&mut A, B, &C) -> R`
//
// These types are not actually used anywhere.
pub struct Mut<T>(T);
//pub struct Ref<T>(T);

/// Dereference into DynamicWriteLock
#[inline(always)]
#[must_use]
pub fn by_ref<T: Variant + Clone>(data: &mut Dynamic) -> DynamicWriteLock<T> {
    // Directly cast the &mut Dynamic into DynamicWriteLock to access the underlying data.
    data.write_lock::<T>().expect("data type was checked")
}

/// Dereference into value.
#[inline]
#[must_use]
pub fn by_value<T: Variant + Clone>(data: &mut Dynamic) -> T {
    if TypeId::of::<T>() == TypeId::of::<&str>() {
        // If T is `&str`, data must be `ImmutableString`, so map directly to it
        data.flatten_in_place();
        let ref_str = data
            .as_str_ref()
            .expect("argument passed by value is not shared");
        let ref_t = unsafe { mem::transmute::<_, &T>(&ref_str) };
        ref_t.clone()
    } else if TypeId::of::<T>() == TypeId::of::<String>() {
        // If T is `String`, data must be `ImmutableString`, so map directly to it
        let value = mem::take(data)
            .into_string()
            .expect("data type was checked");
        unsafe_try_cast(value).expect("data type was checked")
    } else {
        // We consume the argument and then replace it with () - the argument is not supposed to be used again.
        // This way, we avoid having to clone the argument again, because it is already a clone when passed here.
        mem::take(data).cast::<T>()
    }
}

/// Trait to register custom Rust functions.
pub trait RegisterNativeFunction<Args, Result> {
    /// Convert this function into a [`CallableFunction`].
    #[must_use]
    fn into_callable_function(self) -> CallableFunction;
    /// Get the type ID's of this function's parameters.
    #[must_use]
    fn param_types() -> Box<[TypeId]>;
    /// _(metadata)_ Get the type names of this function's parameters.
    /// Exported under the `metadata` feature only.
    #[cfg(feature = "metadata")]
    #[must_use]
    fn param_names() -> Box<[&'static str]>;
    /// _(metadata)_ Get the type ID of this function's return value.
    /// Exported under the `metadata` feature only.
    #[cfg(feature = "metadata")]
    #[must_use]
    fn return_type() -> TypeId;
    /// _(metadata)_ Get the type name of this function's return value.
    /// Exported under the `metadata` feature only.
    #[cfg(feature = "metadata")]
    #[must_use]
    fn return_type_name() -> &'static str;
}

#[inline]
#[must_use]
fn is_setter(_fn_name: &str) -> bool {
    #[cfg(not(feature = "no_object"))]
    if _fn_name.starts_with(crate::engine::FN_SET) {
        return true;
    }
    #[cfg(not(feature = "no_index"))]
    if _fn_name.starts_with(crate::engine::FN_IDX_SET) {
        return true;
    }
    false
}

macro_rules! def_register {
    () => {
        def_register!(imp from_pure :);
    };
    (imp $abi:ident : $($par:ident => $arg:expr => $mark:ty => $param:ty => $let:stmt => $clone:expr),*) => {
    //   ^ function ABI type
    //                  ^ function parameter generic type name (A, B, C etc.)
    //                                ^ call argument(like A, *B, &mut C etc)
    //                                             ^ function parameter marker type (T, Ref<T> or Mut<T>)
    //                                                         ^ function parameter actual type (T, &T or &mut T)
    //                                                                      ^ argument let statement

        impl<
            FN: Fn($($param),*) -> RET + SendSync + 'static,
            $($par: Variant + Clone,)*
            RET: Variant + Clone
        > RegisterNativeFunction<($($mark,)*), ()> for FN {
            #[inline(always)] fn param_types() -> Box<[TypeId]> { vec![$(TypeId::of::<$par>()),*].into_boxed_slice() }
            #[cfg(feature = "metadata")] #[inline(always)] fn param_names() -> Box<[&'static str]> { vec![$(std::any::type_name::<$par>()),*].into_boxed_slice() }
            #[cfg(feature = "metadata")] #[inline(always)] fn return_type() -> TypeId { TypeId::of::<RET>() }
            #[cfg(feature = "metadata")] #[inline(always)] fn return_type_name() -> &'static str { std::any::type_name::<RET>() }
            #[inline(always)] fn into_callable_function(self) -> CallableFunction {
                CallableFunction::$abi(Box::new(move |ctx: NativeCallContext, args: &mut FnCallArgs| {
                    if args.len() == 2 && args[0].is_read_only() && is_setter(ctx.fn_name()) {
                        return EvalAltResult::ErrorAssignmentToConstant(Default::default(), Position::NONE).into();
                    }

                    // The arguments are assumed to be of the correct number and types!
                    let mut _drain = args.iter_mut();
                    $($let $par = ($clone)(_drain.next().expect("arguments list is fixed")); )*

                    // Call the function with each argument value
                    let r = self($($arg),*);

                    // Map the result
                    Ok(r.into_dynamic())
                }) as Box<FnAny>)
            }
        }

        impl<
            FN: for<'a> Fn(NativeCallContext<'a>, $($param),*) -> RET + SendSync + 'static,
            $($par: Variant + Clone,)*
            RET: Variant + Clone
        > RegisterNativeFunction<(NativeCallContext<'static>, $($mark,)*), ()> for FN {
            #[inline(always)] fn param_types() -> Box<[TypeId]> { vec![$(TypeId::of::<$par>()),*].into_boxed_slice() }
            #[cfg(feature = "metadata")] #[inline(always)] fn param_names() -> Box<[&'static str]> { vec![$(std::any::type_name::<$par>()),*].into_boxed_slice() }
            #[cfg(feature = "metadata")] #[inline(always)] fn return_type() -> TypeId { TypeId::of::<RET>() }
            #[cfg(feature = "metadata")] #[inline(always)] fn return_type_name() -> &'static str { std::any::type_name::<RET>() }
            #[inline(always)] fn into_callable_function(self) -> CallableFunction {
                CallableFunction::$abi(Box::new(move |ctx: NativeCallContext, args: &mut FnCallArgs| {
                    if args.len() == 2 && args[0].is_read_only() && is_setter(ctx.fn_name()) {
                        return EvalAltResult::ErrorAssignmentToConstant(Default::default(), Position::NONE).into();
                    }

                    // The arguments are assumed to be of the correct number and types!
                    let mut _drain = args.iter_mut();
                    $($let $par = ($clone)(_drain.next().expect("arguments list is fixed")); )*

                    // Call the function with each argument value
                    let r = self(ctx, $($arg),*);

                    // Map the result
                    Ok(r.into_dynamic())
                }) as Box<FnAny>)
            }
        }

        impl<
            FN: Fn($($param),*) -> Result<RET, Box<EvalAltResult>> + SendSync + 'static,
            $($par: Variant + Clone,)*
            RET: Variant + Clone
        > RegisterNativeFunction<($($mark,)*), Result<RET, Box<EvalAltResult>>> for FN {
            #[inline(always)] fn param_types() -> Box<[TypeId]> { vec![$(TypeId::of::<$par>()),*].into_boxed_slice() }
            #[cfg(feature = "metadata")] #[inline(always)] fn param_names() -> Box<[&'static str]> { vec![$(std::any::type_name::<$par>()),*].into_boxed_slice() }
            #[cfg(feature = "metadata")] #[inline(always)] fn return_type() -> TypeId { TypeId::of::<Result<RET, Box<EvalAltResult>>>() }
            #[cfg(feature = "metadata")] #[inline(always)] fn return_type_name() -> &'static str { std::any::type_name::<Result<RET, Box<EvalAltResult>>>() }
            #[inline(always)] fn into_callable_function(self) -> CallableFunction {
                CallableFunction::$abi(Box::new(move |ctx: NativeCallContext, args: &mut FnCallArgs| {
                    if args.len() == 2 && args[0].is_read_only() && is_setter(ctx.fn_name()) {
                        return EvalAltResult::ErrorAssignmentToConstant(Default::default(), Position::NONE).into();
                    }

                    // The arguments are assumed to be of the correct number and types!
                    let mut _drain = args.iter_mut();
                    $($let $par = ($clone)(_drain.next().expect("arguments list is fixed")); )*

                    // Call the function with each argument value
                    self($($arg),*).map(Dynamic::from)
                }) as Box<FnAny>)
            }
        }

        impl<
            FN: for<'a> Fn(NativeCallContext<'a>, $($param),*) -> Result<RET, Box<EvalAltResult>> + SendSync + 'static,
            $($par: Variant + Clone,)*
            RET: Variant + Clone
        > RegisterNativeFunction<(NativeCallContext<'static>, $($mark,)*), Result<RET, Box<EvalAltResult>>> for FN {
            #[inline(always)] fn param_types() -> Box<[TypeId]> { vec![$(TypeId::of::<$par>()),*].into_boxed_slice() }
            #[cfg(feature = "metadata")] #[inline(always)] fn param_names() -> Box<[&'static str]> { vec![$(std::any::type_name::<$par>()),*].into_boxed_slice() }
            #[cfg(feature = "metadata")] #[inline(always)] fn return_type() -> TypeId { TypeId::of::<Result<RET, Box<EvalAltResult>>>() }
            #[cfg(feature = "metadata")] #[inline(always)] fn return_type_name() -> &'static str { std::any::type_name::<Result<RET, Box<EvalAltResult>>>() }
            #[inline(always)] fn into_callable_function(self) -> CallableFunction {
                CallableFunction::$abi(Box::new(move |ctx: NativeCallContext, args: &mut FnCallArgs| {
                    if args.len() == 2 && args[0].is_read_only() && is_setter(ctx.fn_name()) {
                        return EvalAltResult::ErrorAssignmentToConstant(Default::default(), Position::NONE).into();
                    }

                    // The arguments are assumed to be of the correct number and types!
                    let mut _drain = args.iter_mut();
                    $($let $par = ($clone)(_drain.next().expect("arguments list is fixed")); )*

                    // Call the function with each argument value
                    self(ctx, $($arg),*).map(Dynamic::from)
                }) as Box<FnAny>)
            }
        }

        //def_register!(imp_pop $($par => $mark => $param),*);
    };
    ($p0:ident $(, $p:ident)*) => {
        def_register!(imp from_pure   : $p0 => $p0      => $p0      => $p0      => let $p0     => by_value $(, $p => $p => $p => $p => let $p => by_value)*);
        def_register!(imp from_method : $p0 => &mut $p0 => Mut<$p0> => &mut $p0 => let mut $p0 => by_ref   $(, $p => $p => $p => $p => let $p => by_value)*);
        //                ^ CallableFunction constructor
        //                                                             ^ first parameter passed through
        //                                                                                                     ^ others passed by value (by_value)

        // Currently does not support first argument which is a reference, as there will be
        // conflicting implementations since &T: Any and T: Any cannot be distinguished
        //def_register!(imp $p0 => Ref<$p0> => &$p0     => by_ref   $(, $p => $p => $p => by_value)*);

        def_register!($($p),*);
    };
}

def_register!(A, B, C, D, E, F, G, H, J, K, L, M, N, P, Q, R, S, T, U, V);
