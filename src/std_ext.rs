//! Extensions to standard libraries

use std::ptr::NonNull;

use crate::mem_intern::{leak_as_nonnull, reclaim_as_boxed};

/// Extensions on `Box`-like structure
pub trait BoxedExt<T: ?Sized> {
    /// "Leak" the content in the `Box` but returns `NonNull` instead.
    ///
    /// This function is equivalent to the following code:
    /// ```rust,ignore
    /// # use std::ptr::NonNull;
    /// let ptr: *mut T = Box::into_raw(boxed);
    /// let ptr: NonNull<T> = unsafe { NonNull::new_unchecked(ptr) };
    /// ```
    fn leak_as_nonnull(self) -> NonNull<T>;

    /// Assuming the pointer was previously "leaked" out from a `Box`, reclaim it and put it back
    /// into a `Box`.
    ///
    /// This function is equivalent to the following code:
    /// ```rust,ignore
    /// let boxed: Box<T> = unsafe { Box::from_raw(raw_ptr.as_ptr()) };
    /// ```
    ///
    /// # Safety
    /// The reclaimed pointer must be previously leaked from the same `Box`-like structure.
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

/// Extensions on `std::vec::Vec`
pub trait VecExt<T> {
    /// "Leak" the content in the `Vec`, returning a `NonNull` pointer to slice.
    ///
    /// This function is equivalent to the following code:
    /// ```rust,ignore
    /// # use std::ptr::NonNull;
    /// let boxed_slice: Box<[T]> = vec.into_boxed_slice();
    /// let ptr: *mut [T] = Box::into_raw(boxed_slice);
    /// let ptr: NonNull<[T]> = unsafe { NonNull::new_unchecked(ptr) };
    /// ```
    fn into_slice_ptr(self) -> NonNull<[T]>;
}

impl<T> VecExt<T> for Vec<T> {
    #[inline] fn into_slice_ptr(self) -> NonNull<[T]> {
        self.into_boxed_slice().leak_as_nonnull()
    }
}

#[cfg(test)]
mod test {
    use std::ptr::NonNull;

    use crate::std_ext::{BoxedExt, VecExt};

    #[test]
    fn test() {
        let v: Vec<i32> = vec![1, 2, 3, 4];
        let slice_ptr: NonNull<[i32]> = v.into_slice_ptr();

        let slice_ref: &[i32] = unsafe { slice_ptr.as_ref() };
        assert_eq!(slice_ref.len(), 4);
        assert_eq!(slice_ref[0], 1);
        assert_eq!(slice_ref[3], 4);

        let boxed: Box<[i32]> = unsafe { Box::reclaim(slice_ptr) };
        drop(boxed);
    }
}
