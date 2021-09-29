//! Module containing utilities to hash functions and function calls.

#[cfg(feature = "no_std")]
use std::prelude::v1::*;
use std::{
    any::TypeId,
    hash::{BuildHasher, Hash, Hasher},
    iter::empty,
};

/// A hasher that only takes one single [`u64`] and returns it as a hash key.
///
/// # Panics
///
/// Panics when hashing any data type other than a [`u64`].
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct StraightHasher(u64);

impl Hasher for StraightHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
        self.0
    }
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        assert_eq!(bytes.len(), 8, "StraightHasher can only hash u64 values");

        let mut key = [0_u8; 8];
        key.copy_from_slice(bytes);

        self.0 = u64::from_ne_bytes(key);
    }
}

/// A hash builder for `StraightHasher`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct StraightHasherBuilder;

impl BuildHasher for StraightHasherBuilder {
    type Hasher = StraightHasher;

    #[inline(always)]
    fn build_hasher(&self) -> Self::Hasher {
        StraightHasher(42)
    }
}

/// Create an instance of the default hasher.
#[inline(always)]
#[must_use]
pub fn get_hasher() -> ahash::AHasher {
    Default::default()
}

/// Calculate a [`u64`] hash key from a namespace-qualified variable name.
///
/// Module names are passed in via `&str` references from an iterator.
/// Parameter types are passed in via [`TypeId`] values from an iterator.
///
/// # Note
///
/// The first module name is skipped.  Hashing starts from the _second_ module in the chain.
#[inline]
#[must_use]
pub fn calc_qualified_var_hash<'a>(modules: impl Iterator<Item = &'a str>, var_name: &str) -> u64 {
    let s = &mut get_hasher();

    // We always skip the first module
    let mut len = 0;
    modules
        .inspect(|_| len += 1)
        .skip(1)
        .for_each(|m| m.hash(s));
    len.hash(s);
    var_name.hash(s);
    s.finish()
}

/// Calculate a [`u64`] hash key from a namespace-qualified function name
/// and the number of parameters, but no parameter types.
///
/// Module names are passed in via `&str` references from an iterator.
/// Parameter types are passed in via [`TypeId`] values from an iterator.
///
/// # Note
///
/// The first module name is skipped.  Hashing starts from the _second_ module in the chain.
#[inline]
#[must_use]
pub fn calc_qualified_fn_hash<'a>(
    modules: impl Iterator<Item = &'a str>,
    fn_name: &str,
    num: usize,
) -> u64 {
    let s = &mut get_hasher();

    // We always skip the first module
    let mut len = 0;
    modules
        .inspect(|_| len += 1)
        .skip(1)
        .for_each(|m| m.hash(s));
    len.hash(s);
    fn_name.hash(s);
    num.hash(s);
    s.finish()
}

/// Calculate a [`u64`] hash key from a non-namespace-qualified function name
/// and the number of parameters, but no parameter types.
///
/// Parameter types are passed in via [`TypeId`] values from an iterator.
#[inline(always)]
#[must_use]
pub fn calc_fn_hash(fn_name: &str, num: usize) -> u64 {
    calc_qualified_fn_hash(empty(), fn_name, num)
}

/// Calculate a [`u64`] hash key from a list of parameter types.
///
/// Parameter types are passed in via [`TypeId`] values from an iterator.
#[inline]
#[must_use]
pub fn calc_fn_params_hash(params: impl Iterator<Item = TypeId>) -> u64 {
    let s = &mut get_hasher();
    let mut len = 0;
    params.for_each(|t| {
        len += 1;
        t.hash(s);
    });
    len.hash(s);
    s.finish()
}

/// Combine two [`u64`] hashes by taking the XOR of them.
#[inline(always)]
#[must_use]
pub(crate) const fn combine_hashes(a: u64, b: u64) -> u64 {
    a ^ b
}
