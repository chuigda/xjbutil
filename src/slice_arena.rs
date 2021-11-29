use std::alloc::{alloc, dealloc, Layout};
use std::cell::UnsafeCell;
use std::mem::{align_of, size_of};
use std::ops::Deref;
use std::ptr::copy_nonoverlapping;

use crate::unchecked_intern::UncheckedCellOps;

struct ArenaDebris<const DEBRIS_SIZE: usize, const ALIGN: usize> {
    mem: *mut u8,
    usage: usize
}

impl<const DEBRIS_SIZE: usize, const ALIGN: usize> ArenaDebris<DEBRIS_SIZE, ALIGN> {
    fn new() -> Self {
        let layout: Layout = Layout::array::<u8>(DEBRIS_SIZE)
            .unwrap()
            .align_to(ALIGN)
            .unwrap();
        Self {
            mem: unsafe { alloc(layout) },
            usage: 0,
        }
    }

    fn rest<T>(&self) -> usize {
        (DEBRIS_SIZE - self.usage) / size_of::<T>()
    }

    unsafe fn allocate<T>(&mut self, count: usize) -> *mut T {
        let ret: *mut T = self.mem.offset(self.usage as isize) as *mut T;
        let alloc_bytes: usize = count * size_of::<T>();
        let alloc_bytes: usize = alloc_bytes + ALIGN - alloc_bytes % ALIGN;
        self.usage += alloc_bytes;
        ret
    }
}

impl<const DEBRIS_SIZE: usize, const ALIGN: usize> Drop for ArenaDebris<DEBRIS_SIZE, ALIGN> {
    fn drop(&mut self) {
        let layout: Layout = Layout::array::<u8>(DEBRIS_SIZE)
            .unwrap()
            .align_to(ALIGN)
            .unwrap();
        unsafe { dealloc(self.mem, layout); }
    }
}

struct FreeBlock<const ALIGN: usize> {
    size: usize,
    mem: *mut u8
}

impl<const ALIGN: usize> FreeBlock<ALIGN> {
    fn new(size: usize) -> Self {
        let layout: Layout = Layout::array::<u8>(size)
            .unwrap()
            .align_to(ALIGN)
            .unwrap();
        Self {
            size,
            mem: unsafe { alloc(layout) }
        }
    }

    fn as_mut_ptr<T>(&self) -> *mut T {
        self.mem as _
    }
}

impl<const ALIGN: usize> Drop for FreeBlock<ALIGN> {
    fn drop(&mut self) {
        let layout: Layout = Layout::array::<u8>(self.size)
            .unwrap()
            .align_to(ALIGN)
            .unwrap();
        unsafe { dealloc(self.mem, layout); }
    }
}

pub struct SliceArena<const DEBRIS_SIZE: usize, const ALIGN: usize> {
    debris: UnsafeCell<Vec<ArenaDebris<DEBRIS_SIZE, ALIGN>>>,
    free_blocks: UnsafeCell<Vec<FreeBlock<ALIGN>>>,
}

impl<const DEBRIS_SIZE: usize, const ALIGN: usize> SliceArena<DEBRIS_SIZE, ALIGN> {
    pub fn new() -> Self {
        Self {
            debris: UnsafeCell::new(vec![ArenaDebris::new()]),
            free_blocks: UnsafeCell::new(Vec::new())
        }
    }

    pub fn make<T: Copy>(&self, slice: &[T]) -> &[T] {
        assert!(align_of::<T>() < ALIGN);

        let size: usize = slice.len() * size_of::<T>();
        if size >= (DEBRIS_SIZE / 2) {
            let free_block: FreeBlock<ALIGN> = FreeBlock::new(size);
            let ptr: *mut u8 = free_block.as_mut_ptr();
            unsafe { self.free_blocks.get_mut_ref_unchecked().push(free_block); }
            let ptr: *mut T = ptr as _;
            unsafe {
                copy_nonoverlapping(slice.as_ptr(), ptr, slice.len());
                std::slice::from_raw_parts(ptr, slice.len())
            }
        } else {
            let debris: &mut Vec<_> = unsafe { self.debris.get_mut_ref_unchecked() };
            for debris in debris.iter_mut().rev() {
                if debris.rest::<T>() >= slice.len() {
                    let ptr: *mut T = unsafe { debris.allocate::<T>(slice.len()) };
                    unsafe {
                        copy_nonoverlapping(slice.as_ptr(), ptr, slice.len());
                        return std::slice::from_raw_parts(ptr, slice.len());
                    }
                }
            }

            let mut new_debris: ArenaDebris<DEBRIS_SIZE, ALIGN> = ArenaDebris::new();
            let ptr: *mut T = unsafe { new_debris.allocate::<T>(slice.len()) };
            debris.push(new_debris);
            unsafe {
                copy_nonoverlapping(slice.as_ptr(), ptr, slice.len());
                return std::slice::from_raw_parts(ptr, slice.len());
            }
        }
    }

    pub fn make_from_iter<T, I, R>(&self, iterator: I, size: usize) -> &[T]
        where T: Copy,
              I: Iterator<Item = R>,
              R: Deref<Target = T>
    {
        assert!(align_of::<T>() < ALIGN);

        if size >= (DEBRIS_SIZE / 2) {
            let free_block: FreeBlock<ALIGN> = FreeBlock::new(size);
            let ptr: *mut u8 = free_block.as_mut_ptr();
            unsafe { self.free_blocks.get_mut_ref_unchecked().push(free_block); }
            let ptr: *mut T = ptr as _;
            unsafe {
                for (i, item) in iterator.enumerate() {
                    ptr.add(i).write(*item);
                }
                std::slice::from_raw_parts(ptr, size)
            }
        } else {
            let debris: &mut Vec<_> = unsafe { self.debris.get_mut_ref_unchecked() };
            for debris in debris.iter_mut().rev() {
                if debris.rest::<T>() >= size {
                    let ptr: *mut T = unsafe { debris.allocate::<T>(size) };
                    unsafe {
                        for (i, item) in iterator.enumerate() {
                            ptr.add(i).write(*item);
                        }
                        return std::slice::from_raw_parts(ptr, size);
                    }
                }
            }

            let mut new_debris: ArenaDebris<DEBRIS_SIZE, ALIGN> = ArenaDebris::new();
            let ptr: *mut T = unsafe { new_debris.allocate::<T>(size) };
            debris.push(new_debris);
            unsafe {
                for (i, item) in iterator.enumerate() {
                    ptr.add(i).write(*item);
                }
                return std::slice::from_raw_parts(ptr, size);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::slice_arena::SliceArena;

    #[test]
    fn basic_test() {
        let arena: SliceArena<1024, 8> = SliceArena::new();
        let slice1: &[u8] = arena.make(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let slice2: &[u16] = arena.make_from_iter([1u16, 3, 1, 4, 2, 3, 3].iter(), 7);
        assert_eq!(slice1, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(slice2, &[1, 3, 1, 4, 2, 3, 3]);
    }
}
