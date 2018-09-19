use core::{
    future::Future,
    task::{Context, Poll},
};

use pin_utils::pin_mut;

use crate::{spawn::NoSpawn, waker::EmbrioWaker};

/// A `no_std` compatible, allocation-less, single-threaded futures executor;
/// targeted at supporting embedded use-cases.
///
/// See the [crate docs](crate) for more details.
pub struct Executor {
    waker: EmbrioWaker,
}

impl Executor {
    /// Create a new instance of [`Executor`].
    ///
    /// See the [crate docs](crate) for more details.
    pub const fn new() -> Executor {
        Executor {
            waker: EmbrioWaker::new(),
        }
    }

    /// Block on a specific [`Future`] until it completes, returning its output
    /// when it does.
    ///
    /// See the [crate docs](crate) for more details.
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
