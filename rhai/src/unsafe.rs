//! A helper module containing unsafe utility functions.

#[cfg(feature = "no_std")]
use std::prelude::v1::*;
use std::{
    any::{Any, TypeId},
    mem, ptr,
};

/// Cast a type into another type.
#[inline(always)]
pub fn unsafe_try_cast<A: Any, B: Any>(a: A) -> Result<B, A> {
    if TypeId::of::<B>() == a.type_id() {
        // SAFETY: Just checked we have the right type. We explicitly forget the
        // value immediately after moving out, removing any chance of a destructor
        // running or value otherwise being used again.
        unsafe {
            let ret: B = ptr::read(&a as *const _ as *const B);
            mem::forget(a);
            Ok(ret)
        }
    } else {
        Err(a)
    }
}

/// Cast a Boxed type into another type.
#[inline(always)]
pub fn unsafe_cast_box<X: Any, T: Any>(item: Box<X>) -> Result<Box<T>, Box<X>> {
    // Only allow casting to the exact same type
    if TypeId::of::<X>() == TypeId::of::<T>() {
        // SAFETY: just checked whether we are pointing to the correct type
        unsafe {
            let raw: *mut dyn Any = Box::into_raw(item as Box<dyn Any>);
            Ok(Box::from_raw(raw as *mut T))
        }
    } else {
        // Return the consumed item for chaining.
        Err(item)
    }
}

/// # DANGEROUS!!!
///
/// A dangerous function that blindly casts a `&str` from one lifetime to a `&str` of
/// another lifetime.  This is mainly used to let us push a block-local variable into the
/// current [`Scope`][crate::Scope] without cloning the variable name.  Doing this is safe because all local
/// variables in the [`Scope`][crate::Scope] are cleared out before existing the block.
///
/// Force-casting a local variable's lifetime to the current [`Scope`][crate::Scope]'s larger lifetime saves
/// on allocations and string cloning, thus avoids us having to maintain a chain of [`Scope`][crate::Scope]'s.
#[inline(always)]
#[must_use]
pub fn unsafe_cast_var_name_to_lifetime<'s>(name: &str) -> &'s str {
    // WARNING - force-cast the variable name into the scope's lifetime to avoid cloning it
    //           this is safe because all local variables are cleared at the end of the block
    unsafe { mem::transmute(name) }
}
