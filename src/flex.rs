use std::alloc::{Layout, alloc, dealloc};
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ptr::{NonNull, slice_from_raw_parts, slice_from_raw_parts_mut};

use memoffset::raw_field;

use crate::mem_intern::{leak_as_nonnull, reclaim_as_boxed};

pub struct FlexArray<NonFlex, T: Copy> {
    raw: NonNull<FLABuffer<NonFlex, T>>
}

#[derive(Clone, Eq, PartialEq)]
pub struct FLARef<'a, NF, T: Copy> {
    pub non_flex: &'a NF,
    pub slice: &'a [T]
}

#[derive(Eq, PartialEq)]
pub struct FLARefMut<'a, NF, T: Copy> {
    pub non_flex: &'a mut NF,
    pub slice: &'a mut [T]
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct FLAPtr<NF, T: Copy> {
    pub ptr_non_flex: NonNull<NF>,
    pub ptr_slice: NonNull<[T]>
}

#[repr(C)]
struct FLABuffer<NF, T: Copy> {
    non_flex: NF,
    len: usize,
    _phantom: PhantomData<T>,
}

#[repr(C)]
struct FLABufferHelper<NF, T: Copy> {
    non_flex: NF,
    len: usize,
    placeholder: T
}

impl<NF, T: Copy> FlexArray<NF, T> {
    fn compute_layout(len: usize) -> (Layout, usize) {
        let (layout, array_offset): (Layout, usize) = Layout::new::<FLABuffer<NF, T>>()
            .extend(Layout::array::<T>(len).unwrap())
            .unwrap();
        let layout: Layout = layout.pad_to_align();
        (layout, array_offset)
    }
}

impl<NF, T: Copy> FlexArray<NF, T> {
    pub fn new_empty(non_flex: NF) -> Self {
        Self {
            raw: leak_as_nonnull(Box::new(FLABuffer {
                non_flex,
                len: 0,
                _phantom: PhantomData
            }))
        }
    }

    pub fn new(non_flex: NF, slice: &[T]) -> Self {
        let len: usize = slice.len();
        if len == 0 {
            Self::new_empty(non_flex)
        } else {
            let (layout, array_offset): (Layout, usize) = Self::compute_layout(len);
            unsafe {
                let raw: *mut FLABuffer<NF, T> = alloc(layout) as *mut _;
                std::ptr::write(raw as *mut NF, non_flex);
                (*raw).len = len;
                std::ptr::copy_nonoverlapping(
                    slice.as_ptr(),
                    (raw as *mut u8).add(array_offset) as *mut T,
                    len
                );
                Self { raw: NonNull::new_unchecked(raw) }
            }
        }
    }

    pub fn new_with_iter<I: ExactSizeIterator<Item=T>>(non_flex: NF, iter: I) -> Self {
        let len: usize = iter.len();
        if len == 0 {
            Self::new_empty(non_flex)
        } else {
            let (layout, array_offset): (Layout, usize) = Self::compute_layout(len);
            unsafe {
                let raw: *mut FLABuffer<NF, T> = alloc(layout) as *mut _;
                let arr: *mut T = (raw as *mut u8).add(array_offset) as *mut T;

                std::ptr::write(raw as *mut NF, non_flex);
                (*raw).len = len;
                for (idx, item) /*: (usize, T)*/ in iter.enumerate() {
                    std::ptr::write(arr.add(idx), item);
                }
                Self { raw: NonNull::new_unchecked(raw) }
            }
        }
    }

    pub fn as_ref(&self) -> FLARef<NF, T> {
        let raw: *mut FLABuffer<NF, T> = self.raw.as_ptr();
        let len: usize = unsafe { (*raw).len };
        if len == 0 {
            FLARef {
                non_flex: unsafe { &(*raw).non_flex },
                slice: &[]
            }
        } else {
            unsafe {
                FLARef {
                    non_flex: &(*raw).non_flex,
                    slice: &*slice_from_raw_parts(
                        raw_field!(raw, FLABufferHelper<NF, T>, placeholder),
                        len
                    )
                }
            }
        }
    }

    pub fn as_mut(&mut self) -> FLARef<NF, T> {
        let raw: *mut FLABuffer<NF, T> = self.raw.as_ptr();
        let len: usize = unsafe { (*raw).len };
        if len == 0 {
            FLARef {
                non_flex: unsafe { &(*raw).non_flex },
                slice: &[]
            }
        } else {
            unsafe {
                FLARef {
                    non_flex: &mut (*raw).non_flex,
                    slice: &mut *slice_from_raw_parts_mut(
                        raw_field!(raw as *const _, FLABufferHelper<NF, T>, placeholder) as *mut _,
                        len
                    )
                }
            }
        }
    }
}

impl<NF, T: Copy> Drop for FlexArray<NF, T> {
    fn drop(&mut self) {
        unsafe {
            let raw: *mut FLABuffer<NF, T> = self.raw.as_ptr();
            if (*raw).len == 0 {
                drop(reclaim_as_boxed(self.raw));
            } else {
                let non_flex_ptr: *mut ManuallyDrop<MaybeUninit<NF>> = self.raw.as_ptr() as _;
                let non_flex: NF = ManuallyDrop::take(&mut *non_flex_ptr).assume_init();
                drop(non_flex);

                let (layout, _): (Layout, _) = Self::compute_layout((*raw).len);
                dealloc(self.raw.as_ptr() as *mut u8, layout);
            }
        }
    }
}

unsafe impl<NF, T> Send for FlexArray<NF, T> where NF: Send, T: Copy + Send {}
unsafe impl<NF, T> Sync for FlexArray<NF, T> where NF: Sync, T: Copy + Sync {}
