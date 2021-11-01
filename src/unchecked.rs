//! Unchecked counterparts to standard library components

/// Provides unchecked variant of `std::option::Option`
///
/// The `UncheckedOption` is provided as an unsafe counterpart to `std::option::Option`, with
/// no checks or guarantees. User must guarantee the correctness on themselves.
#[cfg(debug_assertions)]
pub struct UncheckedOption<T> {
    inner: Option<T>
}

#[cfg(debug_assertions)]
impl<T> UncheckedOption<T> {
    /// Create an `UncheckedOption` containing given value `t`.
    pub fn new(t: T) -> Self {
        Self {
            inner: Some(t)
        }
    }

    /// Create an empty `UncheckedOption`
    pub fn new_none() -> Self {
        Self {
            inner: None
        }
    }

    /// Assuming the `UncheckedOption` containing a value, take out the item stored in
    /// `UncheckedOption`.
    ///
    /// # Safety
    /// The `UncheckedOption` must really contains a `T`. If not, this function will panic in
    /// debug build, cause undefined behavior in release build.
    pub unsafe fn take(&mut self) -> T {
        self.inner.take().unwrap()
    }

    /// Assuming the `UncheckedOption` containing a value, get an immutable reference to the item
    /// stored in `UncheckedOption`.
    ///
    /// # Safety
    /// The `UncheckedOption` must really contains a `T`. If not, this function will panic in
    /// debug build, cause undefined behavior in release build.
    pub unsafe fn get_ref(&self) -> &T {
        self.inner.as_ref().unwrap()
    }

    /// Assuming the `UncheckedOption` containing a value, get an mutable reference to the item
    /// stored in `UncheckedOption`.
    ///
    /// # Safety
    /// The `UncheckedOption` must really contains a `T`. If not, this function will panic in
    /// debug build, cause undefined behavior in release build.
    pub unsafe fn get_mut(&mut self) -> &mut T {
        self.inner.as_mut().unwrap()
    }

    /// Assuming the `UncheckedOption` is empty, put a `T` object into it.
    ///
    /// # Safety
    /// The `UncheckedOption` must be empty. If not, this function will panic in
    /// debug build, or cause potential resource leaks in release build. This function does not have
    /// UB, but still marked as `unsafe` in order to remind user.
    pub unsafe fn set(&mut self, t: T) {
        let origin: Option<T> = self.inner.replace(t);
        assert!(origin.is_none());
    }
}

#[cfg(not(debug_assertions))]
use std::mem::{MaybeUninit, replace};

#[cfg(not(debug_assertions))]
pub struct UncheckedOption<T> {
    inner: MaybeUninit<T>
}

#[cfg(not(debug_assertions))]
impl<T> UncheckedOption<T> {
    pub fn new(t: T) -> Self {
        Self {
            inner: MaybeUninit::new(t)
        }
    }

    pub fn new_none() -> Self {
        Self {
            inner: MaybeUninit::uninit()
        }
    }

    pub unsafe fn take(&mut self) -> T {
        let ret: MaybeUninit<T> = replace(&mut self.inner, MaybeUninit::uninit());
        ret.assume_init()
    }

    pub unsafe fn get_ref(&self) -> &T {
        &*self.inner.as_ptr()
    }

    pub unsafe fn get_mut(&mut self) -> &mut T {
        &mut *self.inner.as_mut_ptr()
    }

    pub unsafe fn set(&mut self, t: T) {
        let _ = replace(&mut self.inner, MaybeUninit::new(t));
    }
}

use std::cell::UnsafeCell;

/// Unchecked operations added to `UnsafeCell`
pub trait UncheckedCellOps {
    type Target;

    /// Assume the Rust aliasing model invariants are hold, gets an immutable reference from given
    /// `UnsafeCell` without checking.
    ///
    /// This function is equivalent to the following code:
    /// ```rust,ignore
    /// let ptr: *const T = unsafe_cell.get() as *const T;
    /// let imm_ref: &T = unsafe { &*ptr };
    /// ```
    ///
    /// # Safety
    /// If another mutable reference already exists, calling this function would immediately trigger
    /// undefined behavior.
    unsafe fn get_ref_unchecked(&self) -> &Self::Target;

    /// Assume the Rust aliasing model invariants are hold, gets a mutable reference from given
    /// `UnsafeCell` without checking.
    ///
    /// This function is equivalent to the following code:
    /// ```rust,ignore
    /// let ptr: *mut T = unsafe_cell.get();
    /// let mut_ref: &mut T = unsafe { &mut *ptr };
    /// ```
    ///
    /// # Safety
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

/// Unchecked counterpart to `std::convert::From`
pub trait UnsafeFrom<T> {
    unsafe fn unsafe_from(data: T) -> Self;
}

/// Unchecked counterpart to `std::convert::Into`
pub trait UnsafeInto<T> {
    unsafe fn unsafe_into(self) -> T;
}

pub struct UncheckedSend<T> { inner: T }

unsafe impl<T> Send for UncheckedSend<T> {}

impl<T> UncheckedSend<T> {
    pub unsafe fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn as_ref(&self) -> &T {
        &self.inner
    }

    pub fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn as_ptr(&self) -> *const T {
        self.as_ref() as *const T
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.as_mut() as *mut T
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

pub struct UncheckedSendSync<T> { inner: T }

unsafe impl<T> Send for UncheckedSendSync<T> {}
unsafe impl<T> Sync for UncheckedSendSync<T> {}

impl<T> UncheckedSendSync<T> {
    pub unsafe fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn as_ref(&self) -> &T {
        &self.inner
    }

    pub fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn as_ptr(&self) -> *const T {
        self.as_ref() as *const T
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.as_mut() as *mut T
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

#[cfg(feature = "async")] use std::pin::Pin;
#[cfg(feature = "async")] use std::task::{Context, Poll};
#[cfg(feature = "async")] use futures::Future;

#[cfg(feature = "async")]
pub struct UncheckedSendFut<FUT> {
    fut: Pin<Box<FUT>>
}

#[cfg(feature = "async")]
impl<FUT> UncheckedSendFut<FUT> {
    pub unsafe fn new(fut: FUT) -> Self {
        Self { fut: Box::pin(fut) }
    }
}

#[cfg(feature = "async")]
impl<F, R> Future for UncheckedSendFut<F>
    where F: Future<Output=R>
{
    type Output = R;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::as_mut(&mut self.fut).poll(cx)
    }
}

#[cfg(feature = "async")] impl<F> Unpin for UncheckedSendFut<F> {}
#[cfg(feature = "async")] unsafe impl<F> Send for UncheckedSendFut<F> {}
#[cfg(feature = "async")] unsafe impl<F> Sync for UncheckedSendFut<F> {}
