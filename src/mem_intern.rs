use std::ptr::NonNull;

/// Move the given object to heap, returning a pointer to it.
///
/// This function is equivalent to the following code:
/// ```rust,ignore
/// # use std::ptr::NonNull;
/// let boxed: Box<T> = Box::new(data);
/// let ptr: *mut T = Box::into_raw(boxed);
/// let ptr: NonNull<T> = unsafe { NonNull::new_unchecked(ptr) };
/// ```
#[inline] pub fn move_to_heap<T>(data: T) -> NonNull<T> {
    let boxed: Box<T> = Box::new(data);
    leak_as_nonnull(boxed)
}

/// "Leak" the content in the `Box` but returns `NonNull` instead.
///
/// This function is equivalent to the following code:
/// ```rust,ignore
/// # use std::ptr::NonNull;
/// let ptr: *mut T = Box::into_raw(boxed);
/// let ptr: NonNull<T> = unsafe { NonNull::new_unchecked(ptr) };
/// ```
#[inline] pub fn leak_as_nonnull<T>(boxed: Box<T>) -> NonNull<T>
    where T: ?Sized
{
    let ptr: *mut T = Box::into_raw(boxed);
    unsafe { NonNull::new_unchecked(ptr) }
}

/// Assuming the pointer was previously "leaked" out from a `Box`, reclaim it and put it back
/// into a `Box`.
///
/// This function is equivalent to the following code:
/// ```rust,ignore
/// let boxed: Box<T> = unsafe { Box::from_raw(raw_ptr.as_ptr()) };
/// ```
///
/// # Safety
/// The reclaimed pointer must be previously leaked from a `Box`.
#[inline] pub unsafe fn reclaim_as_boxed<T>(raw_ptr: NonNull<T>) -> Box<T>
    where T: ?Sized
{
    Box::from_raw(raw_ptr.as_ptr())
}
