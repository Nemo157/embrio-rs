use core::sync::atomic::{AtomicBool, Ordering};

pub struct EmbrioWaker {
    woken: AtomicBool,
}

impl EmbrioWaker {
    pub(crate) const fn new() -> Self {
        EmbrioWaker {
            woken: AtomicBool::new(false),
        }
    }

    pub(crate) fn wake(&self) {
        self.woken.store(true, Ordering::Release);
        // we send an event in case this was a non-interrupt driven wake
        cortex_m::asm::sev();
    }

    pub(crate) fn test_and_clear(&self) -> bool {
        self.woken.swap(false, Ordering::AcqRel)
    }

    pub(crate) fn sleep() {
        cortex_m::asm::wfe();
    }
}
