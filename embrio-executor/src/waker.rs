// TODO: Make this much much _MUCH_ safer to use

use core::{
    sync::atomic::{AtomicBool, Ordering},
    ptr::NonNull,
};
use futures_core::{
    task::{LocalWaker, UnsafeWake, Waker},
};

pub struct EmbrioWaker<'a> {
    woken: &'a AtomicBool,
}

impl<'a> EmbrioWaker<'a> {
    pub(crate) fn new(woken: &'a AtomicBool) -> Self {
        EmbrioWaker { woken }
    }

    pub(crate) unsafe fn waker(&self) -> Waker {
        Waker::new(NonNull::new_unchecked(self as &UnsafeWake as *const _ as *mut _))
    }

    pub(crate) unsafe fn local_waker(&self) -> LocalWaker {
        LocalWaker::new(NonNull::new_unchecked(self as &UnsafeWake as *const _ as *mut _))
    }
}

unsafe impl<'a> UnsafeWake for EmbrioWaker<'a> {
    unsafe fn clone_raw(&self) -> Waker {
        self.waker()
    }

    unsafe fn drop_raw(&self) {
    }

    unsafe fn wake(&self) {
        self.woken.store(true, Ordering::SeqCst)
    }
}
