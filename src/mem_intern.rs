use std::ptr::NonNull;

#[inline] pub fn move_to_heap<T>(data: T) -> NonNull<T> {
    let boxed: Box<T> = Box::new(data);
    leak_as_nonnull(boxed)
}

#[inline] pub fn leak_as_nonnull<T>(boxed: Box<T>) -> NonNull<T>
    where T: ?Sized
{
    let ptr: *mut T = Box::into_raw(boxed);
    unsafe { NonNull::new_unchecked(ptr) }
}

#[inline] pub unsafe fn reclaim_as_boxed<T>(raw_ptr: NonNull<T>) -> Box<T>
    where T: ?Sized
{
    Box::from_raw(raw_ptr.as_ptr())
}
