use core::{
    sync::atomic::{AtomicBool, Ordering},
    task::{RawWaker, RawWakerVTable, Waker},
};

pub struct EmbrioWaker {
    woken: AtomicBool,
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
            woken: AtomicBool::new(false),
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
        self.woken.store(true, Ordering::Release)
    }

    pub(crate) fn test_and_clear(&self) -> bool {
        self.woken.swap(false, Ordering::AcqRel)
    }

    pub(crate) fn sleep() {}
}
