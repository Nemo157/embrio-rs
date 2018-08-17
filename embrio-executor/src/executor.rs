use crate::{spawn::NoSpawn, EmbrioWaker};
use futures_core::{task::Context, Future, Poll};
use pin_utils::pin_mut;

pub struct Executor {
    waker: EmbrioWaker,
}

impl Executor {
    pub const fn new() -> Executor {
        Executor { waker: EmbrioWaker::new() }
    }

    pub fn block_on<F: Future>(&'static mut self, future: F) -> F::Output {
        pin_mut!(future);

        let local_waker = self.waker.local_waker();
        let mut spawn = NoSpawn;
        let mut context = Context::new(&local_waker, &mut spawn);

        loop {
            if let Poll::Ready(val) = future.reborrow().poll(&mut context) {
                return val;
            } else {
                while !self.waker.test_and_clear() {
                    EmbrioWaker::sleep()
                }
            }
        }
    }
}
