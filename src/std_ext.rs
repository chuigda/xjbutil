use std::cell::UnsafeCell;
use std::ptr::NonNull;

use crate::mem_intern::{leak_as_nonnull, reclaim_as_boxed};

pub trait BoxedExt<T: ?Sized> {
    fn leak_as_nonnull(self) -> NonNull<T>;
    unsafe fn reclaim(raw_ptr: NonNull<T>) -> Self;
}

impl<T: ?Sized> BoxedExt<T> for Box<T> {
    #[inline] fn leak_as_nonnull(self) -> NonNull<T> {
        leak_as_nonnull(self)
    }

    #[inline] unsafe fn reclaim(raw_ptr: NonNull<T>) -> Self {
        reclaim_as_boxed(raw_ptr)
    }
}

pub trait VecExt<T> {
    fn into_slice_ptr(self) -> NonNull<[T]>;
}

impl<T> VecExt<T> for Vec<T> {
    #[inline] fn into_slice_ptr(self) -> NonNull<[T]> {
        self.into_boxed_slice().leak_as_nonnull()
    }
}

/// Unchecked operations added to `UnsafeCell`
pub trait UncheckedCellOps {
    type Target;

    /// Assume the Rust aliasing model invariants are hold, gets an immutable reference from given
    /// `UnsafeCell` without checking.
    ///
    /// # Safety
    ///
    /// If another mutable reference already exists, calling this function would immediately trigger
    /// undefined behavior.
    unsafe fn get_ref_unchecked(&self) -> &Self::Target;

    /// Assume the Rust aliasing model invariants are hold, gets a mutable reference from given
    /// `UnsafeCell` without checking.
    ///
    /// # Safety
    ///
    /// If another mutable reference or immutable reference already exists, calling this function
    /// would immediately trigger undefined behavior.
    unsafe fn get_mut_ref_unchecked(&self) -> &mut Self::Target;
}

impl<T> UncheckedCellOps for UnsafeCell<T> {
    type Target = T;

    unsafe fn get_ref_unchecked(&self) -> &Self::Target {
        &*(self.get() as *const T)
    }

    unsafe fn get_mut_ref_unchecked(&self) -> &mut Self::Target {
        &mut *self.get()
    }
}

