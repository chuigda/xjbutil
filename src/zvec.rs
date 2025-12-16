//! A special vector lets you soundly read garbage data.

use std::alloc::{Layout, alloc_zeroed, dealloc};
use std::intrinsics::copy_nonoverlapping;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::ptr::NonNull;
use std::slice;
use std::slice::SliceIndex;

use unchecked_unwrap::UncheckedUnwrap;

/// Types which can be simply initialized with zero, and might not care about garbage values.
pub unsafe trait TrivialInit: Copy {}

struct ZeroRawVec<T: TrivialInit> {
    ptr: NonNull<T>,
    cap: usize
}

impl<T: TrivialInit> ZeroRawVec<T> {
    pub unsafe fn new(cap: usize) -> Self {
        debug_assert_ne!(cap, 0);
        let layout: Layout = Layout::array::<T>(cap).unwrap();
        let ptr: *mut u8 = alloc_zeroed(layout);
        let ptr: NonNull<T> = NonNull::new(ptr as *mut T).unwrap();
        Self { ptr, cap }
    }

    pub fn extend(&mut self, cap: usize) {
        let double_cap: usize = self.cap * 2;
        let new_cap: usize = usize::max(double_cap, cap);

        let old_layout: Layout = unsafe { Layout::array::<T>(self.cap).unchecked_unwrap() };
        let new_layout: Layout = Layout::array::<T>(new_cap).unwrap();
        let old_ptr: *mut u8 = self.ptr.as_ptr() as _;
        let new_ptr: *mut u8 = unsafe { alloc_zeroed(new_layout) };
        self.ptr = NonNull::new(new_ptr as *mut T).unwrap();
        self.cap = new_cap;
        unsafe {
            copy_nonoverlapping(old_ptr, new_ptr, old_layout.size());
            dealloc(old_ptr, old_layout);
        }
    }
}

impl<T: TrivialInit> Drop for ZeroRawVec<T> {
    fn drop(&mut self) {
        let layout: Layout = unsafe { Layout::array::<T>(self.cap).unchecked_unwrap() };
        unsafe { dealloc(self.ptr.as_ptr() as _, layout); }
    }
}

/// A vector for soundly reading garbage data
///
/// Compared to [`std::vec::Vec`], this vector does not re-initialize spaces previously used when
/// preforming `resize` operations or so. This is useful when something is garbage insensitive.
pub struct ZeroVec<T: TrivialInit> {
    raw: ZeroRawVec<T>,
    len: usize
}

impl<T: TrivialInit> ZeroVec<T> {
    pub fn new() -> Self {
        Self {
            raw: unsafe { ZeroRawVec::new(16) },
            len: 0
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        assert_ne!(cap, 0);
        Self {
            raw: unsafe { ZeroRawVec::new(cap) },
            len: 0
        }
    }

    pub fn resize(&mut self, new_len: usize) {
        if new_len > self.len && new_len > self.raw.cap {
            self.raw.extend(new_len);
        }
        self.len = new_len
    }

    pub fn len(&self) -> usize {
        self.len
    }

    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is undefined behavior.
    pub unsafe fn get_unchecked(&self, idx: usize) -> &T {
        &*self.raw.ptr.as_ptr().add(idx)
    }

    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is undefined behavior.
    pub unsafe fn get_unchecked_mut(&mut self, idx: usize) -> &mut T {
        &mut *self.raw.ptr.as_ptr().add(idx)
    }
}

impl<T: TrivialInit> Deref for ZeroVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.raw.ptr.as_ptr() as _, self.len) }
    }
}

impl<T: TrivialInit> DerefMut for ZeroVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.raw.ptr.as_ptr(), self.len) }
    }
}

impl<T: TrivialInit, I: SliceIndex<[T]>> Index<I> for ZeroVec<T> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<T: TrivialInit, I: SliceIndex<[T]>> IndexMut<I> for ZeroVec<T> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}

unsafe impl TrivialInit for i8 {}
unsafe impl TrivialInit for i16 {}
unsafe impl TrivialInit for i32 {}
unsafe impl TrivialInit for i64 {}
unsafe impl TrivialInit for u8 {}
unsafe impl TrivialInit for u16 {}
unsafe impl TrivialInit for u32 {}
unsafe impl TrivialInit for u64 {}

unsafe impl<T1, T2> TrivialInit for (T1, T2) where T1: TrivialInit, T2: TrivialInit {}

unsafe impl<T1, T2, T3> TrivialInit for (T1, T2, T3)
    where T1: TrivialInit, T2: TrivialInit, T3: TrivialInit {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_safe_access() {
        let mut vec: ZeroVec<i32> = ZeroVec::new();
        vec.resize(10);
        
        // Safe access through Index trait
        vec[0] = 42;
        assert_eq!(vec[0], 42);
        
        // Safe access through Deref
        assert_eq!(vec.len(), 10);
        vec[5] = 100;
        assert_eq!(vec[5], 100);
    }
    
    #[test]
    fn test_unsafe_get_unchecked() {
        let mut vec: ZeroVec<i32> = ZeroVec::new();
        vec.resize(10);
        vec[3] = 99;
        
        // Unsafe access requires unsafe block
        unsafe {
            let val = vec.get_unchecked(3);
            assert_eq!(*val, 99);
            
            let val_mut = vec.get_unchecked_mut(3);
            *val_mut = 101;
        }
        
        assert_eq!(vec[3], 101);
    }
}
