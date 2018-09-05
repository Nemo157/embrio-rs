use core::{
    ptr::NonNull,
    sync::atomic::{AtomicBool, Ordering},
    task::{LocalWaker, UnsafeWake, Waker},
};

pub struct EmbrioWaker {
    woken: AtomicBool,
}

impl EmbrioWaker {
    pub(crate) const fn new() -> Self {
        EmbrioWaker {
            woken: AtomicBool::new(false),
        }
    }

    pub(crate) fn local_waker(&'static self) -> LocalWaker {
        unsafe {
            LocalWaker::new(NonNull::new_unchecked(
                &self as &UnsafeWake as *const _ as *mut _,
            ))
        }
    }

    pub(crate) fn test_and_clear(&self) -> bool {
        self.woken.swap(false, Ordering::AcqRel)
    }

    pub(crate) fn sleep() {}
}

unsafe impl UnsafeWake for &'static EmbrioWaker {
    unsafe fn clone_raw(&self) -> Waker {
        Waker::new(NonNull::new_unchecked(
            self as &UnsafeWake as *const _ as *mut _,
        ))
    }

    unsafe fn drop_raw(&self) {}

    unsafe fn wake(&self) {
        self.woken.store(true, Ordering::Release)
    }
}
