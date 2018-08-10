use core::mem;

use futures_core::task::{Context, UnsafeWake};

use crate::EmbrioWaker;

struct WakerRaw {
    inner: *const UnsafeWake,
}

pub trait EmbrioContext {
    fn waker(&mut self) -> &mut EmbrioWaker;
}

impl<'a> EmbrioContext for Context<'a> {
    fn waker(&mut self) -> &mut EmbrioWaker {
        unsafe {
            let waker: &WakerRaw = mem::transmute(Context::waker(self));
            let embrio_waker = EmbrioWaker::instance();
            // Ignore vtable, it seems Rust doesn't guarantee a single vtable
            // per object->trait mapping so we need to compare just the
            // location by converting to a thin pointer.
            if (waker.inner as *const ()) == (embrio_waker as *const ()) {
                &mut *embrio_waker
            } else {
                panic!("Context does not contain an EmbrioWaker");
            }
        }
    }
}
