//! Customized box structure for avoiding certain `std::boxed::Box` issues
//!
//! See <https://users.rust-lang.org/t/suspicious-undefined-hehaviour-report-about-stacked-borrows/62633/5>

use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;

use crate::mem_intern::{leak_as_nonnull, reclaim_as_boxed};

/// The customized `Box` replacement
#[repr(transparent)]
pub struct Korobka<T: ?Sized>(NonNull<T>, PhantomData<T>);

impl<T: ?Sized> Drop for Korobka<T> {
    fn drop(&mut self) {
        let boxed: Box<T> = unsafe { reclaim_as_boxed(self.0) };
        drop(boxed);
    }
}

impl<T: Sized> Korobka<T> {
    #[inline(always)] pub fn new(t: T) -> Self {
        Self(leak_as_nonnull(Box::new(t)), PhantomData::default())
    }
}

impl<T: ?Sized> Korobka<T> {
    pub const fn as_ptr(&self) -> *const T {
        self.0.as_ptr() as *const _
    }

    pub const fn as_nonnull(&self) -> NonNull<T> {
        self.0
    }
}

impl<T: ?Sized> AsRef<T> for Korobka<T> {
    fn as_ref(&self) -> &T {
        unsafe { self.0.as_ref() }
    }
}

impl<T: ?Sized> Borrow<T> for Korobka<T> {
    fn borrow(&self) -> &T {
        self.as_ref()
    }
}

impl<T: ?Sized> Deref for Korobka<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T: ?Sized> From<Box<T>> for Korobka<T> {
    fn from(boxed: Box<T>) -> Self {
        Self (leak_as_nonnull(boxed), PhantomData::default())
    }
}

impl<T> Hash for Korobka<T> where T: ?Sized + Hash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe {
            self.0.as_ref().hash(state);
        }
    }
}

impl<T> PartialEq for Korobka<T> where T: ?Sized + PartialEq {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.0.as_ref().eq(other.0.as_ref()) }
    }
}

impl<T> Eq for Korobka<T> where T: ?Sized + Eq + PartialEq {}

#[cfg(test)]
mod test {
    use std::ptr::NonNull;

    use crate::korobka::Korobka;

    #[test]
    fn test_korobka() {
        let korobka: Korobka<String> = Korobka::new("114514".into());
        assert_eq!(korobka.as_ref(), "114514");

        let ptr: NonNull<String> = korobka.as_nonnull();
        let v: Vec<Korobka<String>> = vec![korobka];

        eprintln!("v[0].as_ref() = {}", v[0].as_ref());
        eprintln!("ptr.as_ref() = {}", unsafe { ptr.as_ref() });
    }
}
