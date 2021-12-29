//! A simple [flexible array member](https://en.wikipedia.org/wiki/Flexible_array_member) for Rust
//!
//! The usage is quite simple:
//!
//! ```
//! # use xjbutil::flex::FlexArray;
//! # fn main() {
//! let mut flex: FlexArray<String, char> = FlexArray::new("Ultraman Ace".into(), &['A' ,'C', 'E']);
//! assert_eq!(flex.fixed(), "Ultraman Ace");
//! assert_eq!(flex.flex(), &['A', 'C', 'E']);
//!
//! *flex.fixed_mut() = "ウルトラマンエース".into();
//! let flex_mut = flex.flex_mut();
//! flex_mut[0] = 'エ';
//! flex_mut[1] = 'ー';
//! flex_mut[2] = 'ス';
//!
//! assert_eq!(flex.fixed(), "ウルトラマンエース");
//! assert_eq!(flex.flex(), &['エ', 'ー', 'ス']);
//! # }
//! ```

use std::alloc::{Layout, alloc, dealloc};
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ptr::{NonNull, addr_of, addr_of_mut, slice_from_raw_parts, slice_from_raw_parts_mut};

use crate::mem_intern::{leak_as_nonnull, reclaim_as_boxed};

pub struct FlexArray<NonFlex, T: Copy> {
    raw: NonNull<FLABuffer<NonFlex, T>>
}

#[derive(Clone, Eq, PartialEq)]
pub struct FLARef<'a, NF, T: Copy> {
    pub fixed: &'a NF,
    pub flex: &'a [T]
}

#[derive(Eq, PartialEq)]
pub struct FLARefMut<'a, NF, T: Copy> {
    pub fixed: &'a mut NF,
    pub flex: &'a mut [T]
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct FLAPtr<NF, T: Copy> {
    pub ptr_fixed: NonNull<NF>,
    pub ptr_flex: NonNull<[T]>
}

#[repr(C)]
struct FLABuffer<NF, T: Copy> {
    fixed: NF,
    len: usize,
    _phantom: PhantomData<T>,
}

#[repr(C)]
struct FLABufferHelper<NF, T: Copy> {
    fixed: NF,
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
    pub fn new_empty(fixed: NF) -> Self {
        Self {
            raw: leak_as_nonnull(Box::new(FLABuffer { fixed, len: 0, _phantom: PhantomData }))
        }
    }

    pub fn new(fixed: NF, slice: &[T]) -> Self {
        let len: usize = slice.len();
        if len == 0 {
            Self::new_empty(fixed)
        } else {
            let (layout, array_offset): (Layout, usize) = Self::compute_layout(len);
            unsafe {
                let raw: *mut FLABuffer<NF, T> = alloc(layout) as *mut _;
                std::ptr::write(raw as *mut NF, fixed);
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

    pub fn new_with_iter<I: ExactSizeIterator<Item=T>>(fiexed: NF, iter: I) -> Self {
        let len: usize = iter.len();
        if len == 0 {
            Self::new_empty(fiexed)
        } else {
            let (layout, array_offset): (Layout, usize) = Self::compute_layout(len);
            unsafe {
                let raw: *mut FLABuffer<NF, T> = alloc(layout) as *mut _;
                let arr: *mut T = (raw as *mut u8).add(array_offset) as *mut T;

                std::ptr::write(raw as *mut NF, fiexed);
                (*raw).len = len;
                for (idx, item) /*: (usize, T)*/ in iter.enumerate() {
                    std::ptr::write(arr.add(idx), item);
                }
                Self { raw: NonNull::new_unchecked(raw) }
            }
        }
    }

    pub fn fixed(&self) -> &NF {
        unsafe { &self.raw.as_ref().fixed }
    }

    pub fn fixed_mut(&mut self) -> &mut NF {
        unsafe { &mut self.raw.as_mut().fixed }
    }

    pub fn flex(&self) -> &[T] {
        let raw: *const FLABuffer<NF, T> = self.raw.as_ptr();
        let len: usize = unsafe { (*raw).len };
        if len == 0 {
            &[]
        } else {
            unsafe {
                &*slice_from_raw_parts(
                    addr_of!((*(raw as *const FLABufferHelper<NF, T>)).placeholder),
                    len
                )
            }
        }
    }

    pub fn flex_mut(&mut self) -> &mut [T] {
        let raw: *mut FLABuffer<NF, T> = self.raw.as_ptr();
        let len: usize = unsafe { (*raw).len };
        if len == 0 {
            &mut []
        } else {
            unsafe {
                &mut *slice_from_raw_parts_mut(
                    addr_of_mut!((*(raw as *mut FLABufferHelper<NF, T>)).placeholder),
                    len
                )
            }
        }
    }

    pub fn as_ref(&self) -> FLARef<NF, T> {
        let raw: *const FLABuffer<NF, T> = self.raw.as_ptr();
        let len: usize = unsafe { (*raw).len };
        if len == 0 {
            FLARef {
                fixed: unsafe { &(*raw).fixed },
                flex: &[]
            }
        } else {
            unsafe {
                FLARef {
                    fixed: &(*raw).fixed,
                    flex: &*slice_from_raw_parts(
                        addr_of!((*(raw as *const FLABufferHelper<NF, T>)).placeholder),
                        len
                    )
                }
            }
        }
    }

    pub fn as_mut(&mut self) -> FLARefMut<NF, T> {
        let raw: *mut FLABuffer<NF, T> = self.raw.as_ptr();
        let len: usize = unsafe { (*raw).len };
        if len == 0 {
            FLARefMut {
                fixed: unsafe { &mut (*raw).fixed },
                flex: &mut []
            }
        } else {
            unsafe {
                FLARefMut {
                    fixed: &mut (*raw).fixed,
                    flex: &mut *slice_from_raw_parts_mut(
                        addr_of_mut!((*(raw as *mut FLABufferHelper<NF, T>)).placeholder),
                        len
                    )
                }
            }
        }
    }

    pub fn as_ptr(&self) -> FLAPtr<NF, T> {
        let raw: *const FLABuffer<NF, T> = self.raw.as_ptr();
        unsafe {
            let len: usize = (*raw).len;
            let ptr_fixed: NonNull<NF> =
                NonNull::new_unchecked(&(*raw).fixed as *const _ as *mut _);
            let ptr_flex = if len == 0 {
                NonNull::new_unchecked(&mut [] as *mut [T])
            } else {
                NonNull::new_unchecked(slice_from_raw_parts_mut(
                    addr_of!((*(raw as *const FLABufferHelper<NF, T>)).placeholder) as *mut _,
                    len
                ))
            };
            FLAPtr { ptr_fixed, ptr_flex }
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
                let fixed_ptr: *mut ManuallyDrop<MaybeUninit<NF>> = self.raw.as_ptr() as _;
                let fixed: NF = ManuallyDrop::take(&mut *fixed_ptr).assume_init();
                drop(fixed);

                let (layout, _): (Layout, _) = Self::compute_layout((*raw).len);
                dealloc(self.raw.as_ptr() as *mut u8, layout);
            }
        }
    }
}

unsafe impl<NF, T> Send for FlexArray<NF, T> where NF: Send, T: Copy + Send {}
unsafe impl<NF, T> Sync for FlexArray<NF, T> where NF: Sync, T: Copy + Sync {}

#[cfg(test)]
mod test {
    use crate::flex::{FLARef, FLARefMut, FlexArray};

    #[test]
    fn test_zero_length() {
        let flex_array: FlexArray<String, u64> = FlexArray::new("为有牺牲多壮志".into(), &[]);
        let arr_ref: FLARef<String, u64> = flex_array.as_ref();
        assert_eq!(arr_ref.fixed, "为有牺牲多壮志");
        assert_eq!(arr_ref.flex, &[]);
    }

    #[test]
    fn test() {
        let flex_array: FlexArray<String, u64> = FlexArray::new("foo".into(), &[1, 2, 3, 4, 5]);
        let arr_ref: FLARef<String, u64> = flex_array.as_ref();
        assert_eq!(arr_ref.fixed, "foo");
        assert_eq!(arr_ref.flex, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_mutate() {
        let mut flex_array: FlexArray<String, i32> = FlexArray::new(
            "ウルトラマンエース, 宇宙のエース!".into(),
            &[1, 1, 4, 5, 1, 4]
        );
        let arr_mut_ref: FLARefMut<String, i32> = flex_array.as_mut();
        assert_eq!(arr_mut_ref.fixed, "ウルトラマンエース, 宇宙のエース!");
        assert_eq!(arr_mut_ref.flex, &[1, 1, 4, 5, 1, 4]);

        *arr_mut_ref.fixed = "ultraman Ace, u ch u no Ace!".into();
        arr_mut_ref.flex[2] = 114514;

        let arr_ref: FLARef<String, i32> = flex_array.as_ref();
        assert_eq!(arr_ref.fixed, "ultraman Ace, u ch u no Ace!");
        assert_eq!(arr_ref.flex, &[1, 1, 114514, 5, 1, 4]);
    }
}
