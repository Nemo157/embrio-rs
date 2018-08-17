use core::{
    cell::UnsafeCell,
    ptr::NonNull,
    task::{LocalWaker, UnsafeWake, Waker},
};
use cortex_m::interrupt::{self, Mutex};

pub struct EmbrioWaker {
    woken: Mutex<UnsafeCell<bool>>,
}

impl EmbrioWaker {
    pub(crate) const fn new() -> Self {
        EmbrioWaker { woken: Mutex::new(UnsafeCell::new(false)) }
    }

    pub(crate) fn local_waker(&'static self) -> LocalWaker {
        unsafe { LocalWaker::new(NonNull::new_unchecked(&self as &UnsafeWake as *const _ as *mut _)) }
    }

    pub(crate) fn test_and_clear(&self) -> bool {
        interrupt::free(|cs| {
            let woken = unsafe { &mut *self.woken.borrow(cs).get() };
            let was_woken = *woken;
            *woken = false;
            was_woken
        })
    }

    pub(crate) fn sleep() {
        cortex_m::asm::wfe();
    }
}

unsafe impl UnsafeWake for &'static EmbrioWaker {
    unsafe fn clone_raw(&self) -> Waker {
        Waker::new(NonNull::new_unchecked(self as &UnsafeWake as *const _ as *mut _))
    }

    unsafe fn drop_raw(&self) {
    }

    unsafe fn wake(&self) {
        interrupt::free(|cs| {
            *self.woken.borrow(cs).get() = true;
        });
        cortex_m::asm::sev();
    }
}
