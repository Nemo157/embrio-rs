use core::sync::atomic::{AtomicBool, Ordering};
use crate::{spawn::NoSpawn, EmbrioWaker};
use futures_core::{task::Context, Future, Poll};
use pin_utils::pin_mut;

pub struct Executor {
    _reserved: (),
}

impl Executor {
    pub fn new() -> Executor {
        Executor { _reserved: () }
    }

    pub fn block_on<F: Future>(self, future: F) -> F::Output {
        pin_mut!(future);

        let woken = AtomicBool::new(false);
        let waker = EmbrioWaker::new(&woken);
        let local_waker = unsafe { waker.local_waker() };
        let mut spawn = NoSpawn;
        let mut context = Context::new(&local_waker, &mut spawn);

        loop {
            if let Poll::Ready(val) = future.reborrow().poll(&mut context) {
                return val;
            } else {
                while !woken.load(Ordering::SeqCst) {
                    // WFE
                }
            }
        }
    }
}
