//! Unchecked counterparts to standard library components

pub use crate::unchecked_intern::{UncheckedCellOps, UncheckedOption};

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
pub struct UncheckedSendFut<R: 'static> {
    fut: Pin<Box<dyn Future<Output = R>>>
}

#[cfg(feature = "async")]
impl<R: 'static> UncheckedSendFut<R> {
    pub unsafe fn new(fut: impl Future<Output = R> + 'static) -> Self {
        Self { fut: Box::pin(fut) }
    }
}

#[cfg(feature = "async")]
impl<R: 'static> Future for UncheckedSendFut<R> {
    type Output = R;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::as_mut(&mut self.fut).poll(cx)
    }
}

#[cfg(feature = "async")] impl<R: 'static> Unpin for UncheckedSendFut<R> {}
#[cfg(feature = "async")] unsafe impl<R: 'static> Send for UncheckedSendFut<R> {}
#[cfg(feature = "async")] unsafe impl<R: 'static> Sync for UncheckedSendFut<R> {}

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
