//! This crate provides mainly the trait `Some` which can be used on *shared*
//! reference types. (`&T`, `Rc`, `Arc`). It enables the users to test identity
//! of objects. It's analogous to `PartialEq`, which tests *equality* instead.
//!
//! Additionally, this crate provides `RefHash` trait, which is used for hashing
//! references and `RefCmp` wrapper struct, which implements `Eq`, `PartialEq`
//! and `Hash` by delegating to `Same` and `RefHash` traits. This is mainly
//! useful if one wants to store objects in `HashSet` or similar data structure,
//! with.
//!
//! This crate is `no_std`-compatible.

#![no_std]

#[cfg(feature = "std")]
extern crate std;

use core::hash::{Hash, Hasher};

/// Allows to test identity of objects.
///
/// # Example:
///
/// ```
/// use same::Same;
///
/// let a = 42;
/// let b = 42;
/// let a_ref0 = &a;
/// let a_ref1 = &a;
/// let b_ref = &b;
///
/// // `a_ref0` and `a_ref1` point to the same object...
/// assert!(a_ref0.same(&a_ref1));
/// // ... but `a_ref0` and `b_ref` don't...
/// assert!(!a_ref0.same(&b_ref));
/// // ... neither do `a_ref1` and `b_ref`.
/// assert!(!a_ref1.same(&b_ref));
/// ```
///
/// This trait is currently implemented for *shared* references,
/// `Rc` and `Arc`.
///
/// Note that it doesn't make sense to implement this trait for mutable
/// references, nor boxes because there can never be two of them pointing
/// to the same address, so the implementation would always return `false`.
pub trait Same {
    /// Returns true if `self` is the same instance of object as `other`.
    fn same(&self, other: &Self) -> bool;
}

/// Hashes the pointer to the object instead of the object itself.
///
/// This trait works exatly like `Hash`, the only difference being
/// that it hashes the pointer.
pub trait RefHash {
    /// Feeds the value into the hasher.
    fn ref_hash<H: Hasher>(&self, hasher: &mut H);
}

impl<'a, T> Same for &'a T
where
    T: ?Sized,
{
    fn same(&self, other: &Self) -> bool {
        let a: *const T = *self;
        let b: *const T = *other;

        a == b
    }
}

#[cfg(feature = "std")]
impl<T> Same for std::boxed::Box<T>
where
    T: ?Sized,
{
    fn same(&self, other: &Self) -> bool {
        let a: *const T = &**self;
        let b: *const T = &**other;

        a == b
    }
}

#[cfg(feature = "std")]
impl<T> Same for std::rc::Rc<T>
where
    T: ?Sized,
{
    fn same(&self, other: &Self) -> bool {
        std::rc::Rc::ptr_eq(self, other)
    }
}

#[cfg(feature = "std")]
impl<T> Same for std::sync::Arc<T>
where
    T: ?Sized,
{
    fn same(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(self, other)
    }
}

impl<'a, T> RefHash for &'a T
where
    T: ?Sized,
{
    fn ref_hash<H: Hasher>(&self, hasher: &mut H) {
        let ptr: *const T = *self;

        ptr.hash(hasher);
    }
}

#[cfg(feature = "std")]
impl<T> RefHash for std::boxed::Box<T>
where
    T: ?Sized,
{
    fn ref_hash<H: Hasher>(&self, hasher: &mut H) {
        (&**self).ref_hash(hasher);
    }
}

#[cfg(feature = "std")]
impl<T> RefHash for std::rc::Rc<T>
where
    T: ?Sized,
{
    fn ref_hash<H: Hasher>(&self, hasher: &mut H) {
        (&**self).ref_hash(hasher);
    }
}

#[cfg(feature = "std")]
impl<T> RefHash for std::sync::Arc<T>
where
    T: ?Sized,
{
    fn ref_hash<H: Hasher>(&self, hasher: &mut H) {
        (&**self).ref_hash(hasher);
    }
}

/// Wrapper for types to make their equality operations compare pointers.
///
/// This wrapper turns `Same` into `PartialEq` and `RefHash` into `Hash`.
/// It is mainly useful for storing unique objects in hash sets and similar
/// data structures.
///
/// # Example
/// ```
/// use same::RefCmp;
///
/// use std::{collections::HashSet, rc::Rc};
///
/// let a = ::std::sync::Arc::new(42);
/// let a_cloned = a.clone();
/// let b = ::std::sync::Arc::new(42);
///
/// let mut hash_set = ::std::collections::HashSet::new();
/// assert!(hash_set.insert(RefCmp(a)));
/// // a_cloned points to the same object as a...
/// assert!(!hash_set.insert(RefCmp(a_cloned)));
/// // but `b` doesn't, even though it has same value.
/// assert!(hash_set.insert(RefCmp(b)));
/// ```
#[repr(transparent)]
pub struct RefCmp<T: Same + ?Sized>(pub T);

impl<T: Same + ?Sized> RefCmp<T> {
    /// Creates a reference to `RefCmp<T>` from a reference to `T` without
    /// copying.
    pub fn from_ref(inner: &T) -> &Self {
        // this is safe thanks to #[repr(transparent)]
        unsafe { &*(inner as *const _ as *const Self) }
    }
}

impl<T: Same + ?Sized> Eq for RefCmp<T> {}
impl<T: Same + ?Sized> PartialEq for RefCmp<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.same(&other.0)
    }
}

impl<T: Same + RefHash + ?Sized> Hash for RefCmp<T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.0.ref_hash(hasher);
    }
}

impl<T: Same + ?Sized> core::ops::Deref for RefCmp<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<U, T: Same + AsRef<U> + ?Sized> AsRef<U> for RefCmp<T> {
    fn as_ref(&self) -> &U {
        self.0.as_ref()
    }
}

impl<T: Same + ?Sized> core::borrow::Borrow<T> for RefCmp<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use RefCmp;
    use Same;

    #[test]
    fn refs() {
        let a = 42;
        let b = 42;

        let a_ref = &a;
        let a_ref_again = &a;
        let b_ref = &b;

        assert!(a_ref.same(&a_ref_again));
        assert!(!a_ref.same(&b_ref));

        let mut hash_set = ::std::collections::HashSet::new();
        assert!(hash_set.insert(RefCmp(a_ref)));
        assert!(!hash_set.insert(RefCmp(a_ref_again)));
        assert!(hash_set.insert(RefCmp(b_ref)));
    }

    #[test]
    fn boxes() {
        let a = ::std::boxed::Box::new(42);
        let b = ::std::boxed::Box::new(42);

        let a_box = &a;
        let a_box_again = &a;
        let b_box = &b;

        assert!(a_box.same(&a_box_again));
        assert!(!a_box.same(&b_box));

        let mut hash_set = ::std::collections::HashSet::new();
        assert!(hash_set.insert(RefCmp(a_box)));
        assert!(!hash_set.insert(RefCmp(a_box_again)));
        assert!(hash_set.insert(RefCmp(b_box)));
    }

    #[test]
    fn rcs() {
        let a = ::std::rc::Rc::new(42);
        let a_cloned = a.clone();
        let b = ::std::rc::Rc::new(42);

        assert!(a.same(&a_cloned));
        assert!(!a.same(&b));

        let mut hash_set = ::std::collections::HashSet::new();
        assert!(hash_set.insert(RefCmp(a)));
        assert!(!hash_set.insert(RefCmp(a_cloned)));
        assert!(hash_set.insert(RefCmp(b)));
    }

    #[test]
    fn arcs() {
        let a = ::std::sync::Arc::new(42);
        let a_cloned = a.clone();
        let b = ::std::sync::Arc::new(42);

        assert!(a.same(&a_cloned));
        assert!(!a.same(&b));

        let mut hash_set = ::std::collections::HashSet::new();
        assert!(hash_set.insert(RefCmp(a)));
        assert!(!hash_set.insert(RefCmp(a_cloned)));
        assert!(hash_set.insert(RefCmp(b)));
    }
}
