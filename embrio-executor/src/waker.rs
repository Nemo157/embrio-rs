use core::{ptr, convert::From};

use futures::task::{UnsafeWake, Waker};

pub struct WFEWaker;

unsafe impl UnsafeWake for WFEWaker {
    unsafe fn clone_raw(&self) -> Waker {
        Waker::from(WFEWaker)
    }

    unsafe fn drop_raw(&self) {
        // No-op, we're a ZST and just use NULL as our pointer
    }

    unsafe fn wake(&self) {
        // No-op, we use WFE instructions instead
    }
}

impl From<WFEWaker> for Waker {
    fn from(_: WFEWaker) -> Waker {
        unsafe {
            Waker::new(ptr::null_mut() as *mut WFEWaker as *mut UnsafeWake)
        }
    }
}
