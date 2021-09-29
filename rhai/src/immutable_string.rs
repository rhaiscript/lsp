//! The `ImmutableString` type.

use crate::fn_native::{shared_make_mut, shared_take};
use crate::{Shared, SmartString};
#[cfg(feature = "no_std")]
use std::prelude::v1::*;
use std::{
    borrow::Borrow,
    cmp::Ordering,
    fmt,
    hash::Hash,
    iter::FromIterator,
    ops::{Add, AddAssign, Deref, Sub, SubAssign},
    str::FromStr,
};

/// The system immutable string type.
///
/// An [`ImmutableString`] wraps an [`Rc`][std::rc::Rc]`<`[`SmartString`][smartstring::SmartString]`>`
///  (or [`Arc`][std::sync::Arc]`<`[`SmartString`][smartstring::SmartString]`>` under the `sync` feature)
/// so that it can be simply shared and not cloned.
///
/// # Example
///
/// ```
/// use rhai::ImmutableString;
///
/// let s1: ImmutableString = "hello".into();
///
/// // No actual cloning of the string is involved below.
/// let s2 = s1.clone();
/// let s3 = s2.clone();
///
/// assert_eq!(s1, s2);
///
/// // Clones the underlying string (because it is already shared) and extracts it.
/// let mut s: String = s1.into_owned();
///
/// // Changing the clone has no impact on the previously shared version.
/// s.push_str(", world!");
///
/// // The old version still exists.
/// assert_eq!(s2, s3);
/// assert_eq!(s2.as_str(), "hello");
///
/// // Not equals!
/// assert_ne!(s2.as_str(), s.as_str());
/// assert_eq!(s, "hello, world!");
/// ```
#[derive(Clone, Eq, Ord, Hash, Default)]
pub struct ImmutableString(Shared<SmartString>);

impl Deref for ImmutableString {
    type Target = SmartString;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<SmartString> for ImmutableString {
    #[inline(always)]
    fn as_ref(&self) -> &SmartString {
        &self.0
    }
}

impl AsRef<str> for ImmutableString {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<SmartString> for ImmutableString {
    #[inline(always)]
    fn borrow(&self) -> &SmartString {
        &self.0
    }
}

impl Borrow<str> for ImmutableString {
    #[inline(always)]
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for ImmutableString {
    #[inline(always)]
    fn from(value: &str) -> Self {
        Self(Into::<SmartString>::into(value).into())
    }
}
impl From<&String> for ImmutableString {
    #[inline(always)]
    fn from(value: &String) -> Self {
        Self(Into::<SmartString>::into(value).into())
    }
}
impl From<String> for ImmutableString {
    #[inline(always)]
    fn from(value: String) -> Self {
        Self(Into::<SmartString>::into(value).into())
    }
}
#[cfg(not(feature = "no_smartstring"))]
impl From<&SmartString> for ImmutableString {
    #[inline(always)]
    fn from(value: &SmartString) -> Self {
        Self(Into::<SmartString>::into(value.as_str()).into())
    }
}
#[cfg(not(feature = "no_smartstring"))]
impl From<SmartString> for ImmutableString {
    #[inline(always)]
    fn from(value: SmartString) -> Self {
        Self(value.into())
    }
}
impl From<&ImmutableString> for SmartString {
    #[inline(always)]
    fn from(value: &ImmutableString) -> Self {
        value.as_str().into()
    }
}
impl From<ImmutableString> for SmartString {
    #[inline(always)]
    fn from(mut value: ImmutableString) -> Self {
        std::mem::take(shared_make_mut(&mut value.0))
    }
}

impl FromStr for ImmutableString {
    type Err = ();

    #[inline(always)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Into::<SmartString>::into(s).into()))
    }
}

impl FromIterator<char> for ImmutableString {
    #[inline(always)]
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        Self(iter.into_iter().collect::<SmartString>().into())
    }
}

impl<'a> FromIterator<&'a char> for ImmutableString {
    #[inline(always)]
    fn from_iter<T: IntoIterator<Item = &'a char>>(iter: T) -> Self {
        Self(iter.into_iter().cloned().collect::<SmartString>().into())
    }
}

impl<'a> FromIterator<&'a str> for ImmutableString {
    #[inline(always)]
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        Self(iter.into_iter().collect::<SmartString>().into())
    }
}

impl<'a> FromIterator<String> for ImmutableString {
    #[inline(always)]
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self(iter.into_iter().collect::<SmartString>().into())
    }
}

#[cfg(not(feature = "no_smartstring"))]
impl<'a> FromIterator<SmartString> for ImmutableString {
    #[inline(always)]
    fn from_iter<T: IntoIterator<Item = SmartString>>(iter: T) -> Self {
        Self(iter.into_iter().collect::<SmartString>().into())
    }
}

impl fmt::Display for ImmutableString {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.0.as_str(), f)
    }
}

impl fmt::Debug for ImmutableString {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.0.as_str(), f)
    }
}

impl Add for ImmutableString {
    type Output = Self;

    #[inline]
    fn add(mut self, rhs: Self) -> Self::Output {
        if rhs.is_empty() {
            self
        } else if self.is_empty() {
            rhs
        } else {
            self.make_mut().push_str(rhs.0.as_str());
            self
        }
    }
}

impl Add for &ImmutableString {
    type Output = ImmutableString;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        if rhs.is_empty() {
            self.clone()
        } else if self.is_empty() {
            rhs.clone()
        } else {
            let mut s = self.clone();
            s.make_mut().push_str(rhs.0.as_str());
            s
        }
    }
}

impl AddAssign<&ImmutableString> for ImmutableString {
    #[inline]
    fn add_assign(&mut self, rhs: &ImmutableString) {
        if !rhs.is_empty() {
            if self.is_empty() {
                self.0 = rhs.0.clone();
            } else {
                self.make_mut().push_str(rhs.0.as_str());
            }
        }
    }
}

impl AddAssign<ImmutableString> for ImmutableString {
    #[inline]
    fn add_assign(&mut self, rhs: ImmutableString) {
        if !rhs.is_empty() {
            if self.is_empty() {
                self.0 = rhs.0;
            } else {
                self.make_mut().push_str(rhs.0.as_str());
            }
        }
    }
}

impl Add<&str> for ImmutableString {
    type Output = Self;

    #[inline]
    fn add(mut self, rhs: &str) -> Self::Output {
        if !rhs.is_empty() {
            self.make_mut().push_str(rhs);
        }
        self
    }
}

impl Add<&str> for &ImmutableString {
    type Output = ImmutableString;

    #[inline]
    fn add(self, rhs: &str) -> Self::Output {
        if rhs.is_empty() {
            self.clone()
        } else {
            let mut s = self.clone();
            s.make_mut().push_str(rhs);
            s
        }
    }
}

impl AddAssign<&str> for ImmutableString {
    #[inline(always)]
    fn add_assign(&mut self, rhs: &str) {
        if !rhs.is_empty() {
            self.make_mut().push_str(rhs);
        }
    }
}

impl Add<String> for ImmutableString {
    type Output = Self;

    #[inline]
    fn add(mut self, rhs: String) -> Self::Output {
        if rhs.is_empty() {
            self
        } else if self.is_empty() {
            rhs.into()
        } else {
            self.make_mut().push_str(&rhs);
            self
        }
    }
}

impl Add<String> for &ImmutableString {
    type Output = ImmutableString;

    #[inline]
    fn add(self, rhs: String) -> Self::Output {
        if rhs.is_empty() {
            self.clone()
        } else if self.is_empty() {
            rhs.into()
        } else {
            let mut s = self.clone();
            s.make_mut().push_str(&rhs);
            s
        }
    }
}

impl AddAssign<String> for ImmutableString {
    #[inline]
    fn add_assign(&mut self, rhs: String) {
        if !rhs.is_empty() {
            if self.is_empty() {
                self.0 = Into::<SmartString>::into(rhs).into();
            } else {
                self.make_mut().push_str(&rhs);
            }
        }
    }
}

impl Add<char> for ImmutableString {
    type Output = Self;

    #[inline(always)]
    fn add(mut self, rhs: char) -> Self::Output {
        self.make_mut().push(rhs);
        self
    }
}

impl Add<char> for &ImmutableString {
    type Output = ImmutableString;

    #[inline(always)]
    fn add(self, rhs: char) -> Self::Output {
        let mut s = self.clone();
        s.make_mut().push(rhs);
        s
    }
}

impl AddAssign<char> for ImmutableString {
    #[inline(always)]
    fn add_assign(&mut self, rhs: char) {
        self.make_mut().push(rhs);
    }
}

impl Sub for ImmutableString {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        if rhs.is_empty() {
            self
        } else if self.is_empty() {
            rhs
        } else {
            self.replace(rhs.as_str(), "").into()
        }
    }
}

impl Sub for &ImmutableString {
    type Output = ImmutableString;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        if rhs.is_empty() {
            self.clone()
        } else if self.is_empty() {
            rhs.clone()
        } else {
            self.replace(rhs.as_str(), "").into()
        }
    }
}

impl SubAssign<&ImmutableString> for ImmutableString {
    #[inline]
    fn sub_assign(&mut self, rhs: &ImmutableString) {
        if !rhs.is_empty() {
            if self.is_empty() {
                self.0 = rhs.0.clone();
            } else {
                self.0 = Into::<SmartString>::into(self.replace(rhs.as_str(), "")).into();
            }
        }
    }
}

impl SubAssign<ImmutableString> for ImmutableString {
    #[inline]
    fn sub_assign(&mut self, rhs: ImmutableString) {
        if !rhs.is_empty() {
            if self.is_empty() {
                self.0 = rhs.0;
            } else {
                self.0 = Into::<SmartString>::into(self.replace(rhs.as_str(), "")).into();
            }
        }
    }
}

impl Sub<String> for ImmutableString {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: String) -> Self::Output {
        if rhs.is_empty() {
            self
        } else if self.is_empty() {
            rhs.into()
        } else {
            self.replace(&rhs, "").into()
        }
    }
}

impl Sub<String> for &ImmutableString {
    type Output = ImmutableString;

    #[inline]
    fn sub(self, rhs: String) -> Self::Output {
        if rhs.is_empty() {
            self.clone()
        } else if self.is_empty() {
            rhs.into()
        } else {
            self.replace(&rhs, "").into()
        }
    }
}

impl SubAssign<String> for ImmutableString {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: String) {
        self.0 = Into::<SmartString>::into(self.replace(&rhs, "")).into();
    }
}

impl Sub<char> for ImmutableString {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: char) -> Self::Output {
        self.replace(rhs, "").into()
    }
}

impl Sub<char> for &ImmutableString {
    type Output = ImmutableString;

    #[inline(always)]
    fn sub(self, rhs: char) -> Self::Output {
        self.replace(rhs, "").into()
    }
}

impl SubAssign<char> for ImmutableString {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: char) {
        self.0 = Into::<SmartString>::into(self.replace(rhs, "")).into();
    }
}

impl<S: AsRef<str>> PartialEq<S> for ImmutableString {
    #[inline(always)]
    fn eq(&self, other: &S) -> bool {
        self.as_str().eq(other.as_ref())
    }
}

impl PartialEq<ImmutableString> for str {
    #[inline(always)]
    fn eq(&self, other: &ImmutableString) -> bool {
        self.eq(other.as_str())
    }
}

impl PartialEq<ImmutableString> for String {
    #[inline(always)]
    fn eq(&self, other: &ImmutableString) -> bool {
        self.eq(other.as_str())
    }
}

impl<S: AsRef<str>> PartialOrd<S> for ImmutableString {
    fn partial_cmp(&self, other: &S) -> Option<Ordering> {
        self.as_str().partial_cmp(other.as_ref())
    }
}

impl PartialOrd<ImmutableString> for str {
    #[inline(always)]
    fn partial_cmp(&self, other: &ImmutableString) -> Option<Ordering> {
        self.partial_cmp(other.as_str())
    }
}

impl PartialOrd<ImmutableString> for String {
    #[inline(always)]
    fn partial_cmp(&self, other: &ImmutableString) -> Option<Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl ImmutableString {
    /// Create a new [`ImmutableString`].
    #[inline(always)]
    pub fn new() -> Self {
        Self(SmartString::new().into())
    }
    /// Consume the [`ImmutableString`] and convert it into a [`String`].
    /// If there are other references to the same string, a cloned copy is returned.
    #[inline(always)]
    pub fn into_owned(mut self) -> String {
        self.make_mut(); // Make sure it is unique reference
        shared_take(self.0).into() // Should succeed
    }
    /// Make sure that the [`ImmutableString`] is unique (i.e. no other outstanding references).
    /// Then return a mutable reference to the [`SmartString`].
    #[inline(always)]
    pub(crate) fn make_mut(&mut self) -> &mut SmartString {
        shared_make_mut(&mut self.0)
    }
}
