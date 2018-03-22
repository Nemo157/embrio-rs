use core::mem::Pin;

pub fn pinned<T, F, R>(mut data: T, f: F) -> R
    where F: FnOnce(Pin<T>) -> R
{
    f(unsafe { Pin::new_unchecked(&mut data) })
}
