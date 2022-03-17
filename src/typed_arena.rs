use std::alloc::{Layout, alloc, dealloc};
use std::cell::UnsafeCell;

use unchecked_unwrap::UncheckedUnwrap;

use crate::unchecked_intern::UncheckedCellOps;

struct ArenaDebris<T, const DEBRIS_SIZE: usize> {
    mem: *mut T,
    usage: usize
}

impl<T, const DEBRIS_SIZE: usize> ArenaDebris<T, DEBRIS_SIZE> {
    fn new() -> Self {
        let layout: Layout = Layout::array::<T>(DEBRIS_SIZE)
            .unwrap();
        Self {
            mem: unsafe { alloc(layout) as _ },
            usage: 0
        }
    }

    #[inline] fn has_rest(&self) -> bool {
        self.usage < DEBRIS_SIZE
    }

    unsafe fn allocate(&mut self, data: T) -> *mut T {
        let ptr = self.mem.add(self.usage);
        self.usage += 1;
        ptr.write(data);
        ptr
    }
}

impl<T, const DEBRIS_SIZE: usize> Drop for ArenaDebris<T, DEBRIS_SIZE> {
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.usage {
                let ptr = self.mem.add(i);
                ptr.drop_in_place();
            }

            let layout: Layout = Layout::array::<T>(DEBRIS_SIZE)
                .unwrap();
            dealloc(self.mem as _, layout);
        }
    }
}

pub struct ArenaPtr<T> {
    ptr: *mut T,
    from_arena: *const ()
}

impl<T> Clone for ArenaPtr<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            from_arena: self.from_arena
        }
    }
}

impl<T> Copy for ArenaPtr<T> {}

impl<T> ArenaPtr<T> {
    pub fn get<'a>(&self, arena: &'a impl IntoArenaPtr) -> &'a T {
        assert_eq!(self.from_arena, IntoArenaPtr::into(arena));
        unsafe { &*self.ptr }
    }

    pub fn get_mut<'a>(&self, arena: &'a mut impl IntoArenaPtr) -> &'a mut T {
        assert_eq!(self.from_arena, IntoArenaPtr::into(arena));
        unsafe { &mut *self.ptr }
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    pub unsafe fn get_unchecked<'a>(&self, arena: &'a impl IntoArenaPtr) -> &'a T {
        debug_assert_eq!(self.from_arena, IntoArenaPtr::into(arena));
        &*self.ptr
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    pub unsafe fn get_unchecked_mut<'a>(&self, arena: &'a mut impl IntoArenaPtr) -> &'a mut T {
        debug_assert_eq!(self.from_arena, IntoArenaPtr::into(arena));
        &mut *self.ptr
    }

    #[cfg(not(feature = "strict-sound"))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub fn get_tricky<'a>(&self, arena: &'a impl IntoArenaPtr) -> &'a T {
        unsafe { self.get_unchecked(arena) }
    }

    #[cfg(not(feature = "strict-sound"))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub fn get_tricky_mut<'a>(&self, arena: &'a mut impl IntoArenaPtr) -> &'a mut T {
        unsafe { self.get_unchecked_mut(arena) }
    }
}

pub trait IntoArenaPtr {
    #[inline(always)]
    fn into(&self) -> *const () {
        self as *const _ as *const ()
    }
}

pub struct TypedArena<T, const DEBRIS_SIZE: usize> {
    debris: UnsafeCell<Vec<ArenaDebris<T, DEBRIS_SIZE>>>
}

impl<T, const DEBRIS_SIZE: usize> TypedArena<T, DEBRIS_SIZE> {
    pub fn new() -> Self {
        Self {
            debris: UnsafeCell::new(vec![ArenaDebris::new()])
        }
    }

    pub fn make(&self, data: T) -> ArenaPtr<T> {
        unsafe {
            let debris: &mut Vec<ArenaDebris<T, DEBRIS_SIZE>> =
                self.debris.get_mut_ref_unchecked();
            let mut last_piece: &mut ArenaDebris<T, DEBRIS_SIZE> =
                debris.last_mut().unchecked_unwrap();
            if !last_piece.has_rest() {
                debris.push(ArenaDebris::new());
                last_piece = debris.last_mut().unchecked_unwrap();
            }
            ArenaPtr {
                ptr: last_piece.allocate(data),
                from_arena: IntoArenaPtr::into(self)
            }
        }
    }
}

impl<T, const DEBRIS_SIZE: usize> IntoArenaPtr for TypedArena<T, DEBRIS_SIZE> {}

#[cfg(test)]
mod test {
    use crate::rand_intern::random_string;
    use crate::typed_arena::{ArenaPtr, TypedArena};

    #[test]
    fn test() {
        let arena: TypedArena<String, 16> = TypedArena::new();
        let mut vec_source: Vec<String> = Vec::new();
        let mut arena_ptr: Vec<ArenaPtr<String>> = Vec::new();

        for _ in 0..40 {
            let s: String = random_string(32);
            vec_source.push(s.clone());
            arena_ptr.push(arena.make(s));
        }

        for i in 0..40 {
            let s: &String = arena_ptr[i].get(&arena);
            assert_eq!(s, &vec_source[i]);
        }
    }

    #[test]
    fn test2() {
        let mut arena: TypedArena<String, 16> = TypedArena::new();
        let ptr1: ArenaPtr<String> = arena.make(String::from("hello"));
        let ptr2: ArenaPtr<String> = arena.make(String::from("world"));

        let r1: &String = ptr1.get(&arena);
        let r2: &String = ptr2.get(&arena);

        eprintln!("{:?}", r1);
        eprintln!("{:?}", r2);

        let r1: &mut String = ptr1.get_mut(&mut arena);
        *r1 = String::from("hello world");

        let r1: &String = ptr1.get(&arena);
        let r2: &String = ptr2.get(&arena);
        eprintln!("{:?}", r1);
        eprintln!("{:?}", r2);
    }
}
