use core::{
    cell::UnsafeCell,
    task::{RawWaker, RawWakerVTable, Waker},
};

use cortex_m::interrupt::{self, Mutex};

pub struct EmbrioWaker {
    woken: Mutex<UnsafeCell<bool>>,
}

static EMBRIO_WAKER_RAW_WAKER_VTABLE: RawWakerVTable = RawWakerVTable {
    clone: { |data| unsafe { (*(data as *const EmbrioWaker)).raw_waker() } }
        as fn(*const ()) -> RawWaker
        as unsafe fn(*const ()) -> RawWaker,
    wake: { |data| unsafe { (*(data as *const EmbrioWaker)).wake() } }
        as fn(*const ()) as unsafe fn(*const ()),
    drop: {
        |_| (/* Noop */)
    } as fn(*const ()) as unsafe fn(*const ()),
};

impl EmbrioWaker {
    pub(crate) const fn new() -> Self {
        EmbrioWaker {
            woken: Mutex::new(UnsafeCell::new(false)),
        }
    }

    pub(crate) fn waker(&'static self) -> Waker {
        unsafe { Waker::new_unchecked(self.raw_waker()) }
    }

    pub(crate) fn raw_waker(&'static self) -> RawWaker {
        RawWaker::new(
            self as *const _ as *const (),
            &EMBRIO_WAKER_RAW_WAKER_VTABLE,
        )
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
