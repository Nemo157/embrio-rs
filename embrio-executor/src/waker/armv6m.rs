use core::cell::UnsafeCell;

use cortex_m::interrupt::{self, Mutex};

pub struct EmbrioWaker {
    woken: Mutex<UnsafeCell<bool>>,
}

impl EmbrioWaker {
    pub(crate) const fn new() -> Self {
        EmbrioWaker {
            woken: Mutex::new(UnsafeCell::new(false)),
        }
    }

    pub(crate) fn wake(&self) {
        interrupt::free(|cs| unsafe {
            *self.woken.borrow(cs).get() = true;
        });
        cortex_m::asm::sev();
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
