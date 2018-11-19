use core::{
    ptr::NonNull,
    sync::atomic::{AtomicBool, Ordering},
    task::{LocalWaker, Waker, RawWaker, RawWakerVTable},
};

pub struct EmbrioWaker {
    woken: AtomicBool,
}

static EMBRIO_WAKER_RAW_WAKER_VTABLE: RawWakerVTable = RawWakerVTable {
    clone: {
        |data| (data as *const EmbrioWaker as &'static EmbrioWaker).raw_waker()
    } as fn(*const ()) -> RawWaker as unsafe fn(*const ()) -> RawWaker,
    into_waker: {
        |data| Some((data as *const EmbrioWaker as &'static EmbrioWaker).raw_waker())
    } as fn(*const ()) -> Option<RawWaker> as unsafe fn(*const ()) -> Option<RawWaker>,
    wake: {
        |data| (data as *const EmbrioWaker as &'static EmbrioWaker).wake()
    } as fn(*const ()) as unsafe fn(*const ())
    drop_fn: { |data| (/* Noop */) } as fn(*const ()) as unsafe fn(*const ())
};

impl EmbrioWaker {
    pub(crate) const fn new() -> Self {
        EmbrioWaker {
            woken: AtomicBool::new(false),
        }
    }

    pub(crate) fn local_waker(&'static self) -> LocalWaker {
        unsafe {
            LocalWaker::new_unchecked(self.raw_waker())
        }
    }

    pub(crate) fn test_and_clear(&self) -> bool {
        self.woken.swap(false, Ordering::AcqRel)
    }

    pub(crate) fn sleep() {
        cortex_m::asm::wfe();
    }

    pub(crate) fn wake(&self) {
        self.woken.store(true, Ordering::Release);
        // we send an event in case this was a non-interrupt driven wake
        cortex_m::asm::sev();
    }

    // TODO: Is this unsafe in any way?
    pub(crate) fn raw_waker(&'static self) -> RawWaker {
        RawWaker {
            data: self as *const Self as *const (),
            vtable: &EMBRIO_WAKER_RAW_WAKER_VTABLE
        }
    }
}
