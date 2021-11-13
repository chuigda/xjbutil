//! Unchecked counterparts to standard library components

use std::cell::UnsafeCell;

pub use crate::unchecked_intern::UncheckedOption;

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

#[cfg(feature = "async")] use std::future::Future;
#[cfg(feature = "async")] use std::pin::Pin;
#[cfg(feature = "async")] use std::task::{Context, Poll};

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

#[cfg(feature = "async")]
pub struct UncheckedSendFutUnpin<FUT: Unpin> {
    fut: FUT
}

#[cfg(feature = "async")]
impl<FUT: Unpin> UncheckedSendFutUnpin<FUT> {
    pub unsafe fn new(fut: FUT) -> Self {
        Self { fut }
    }
}

#[cfg(feature = "async")]
impl<F, R> Future for UncheckedSendFutUnpin<F>
    where F: Future<Output=R> + Unpin
{
    type Output = R;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.fut).poll(cx)
    }
}

#[cfg(feature = "async")] impl<F: Unpin> Unpin for UncheckedSendFutUnpin<F> {}
#[cfg(feature = "async")] unsafe impl<F: Unpin> Send for UncheckedSendFutUnpin<F> {}
#[cfg(feature = "async")] unsafe impl<F: Unpin> Sync for UncheckedSendFutUnpin<F> {}
