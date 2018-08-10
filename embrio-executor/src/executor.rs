use cortex_m;
use crate::{spawn::NoSpawn, EmbrioWaker};
use futures_core::{task::Context, Future, Poll};
use pin_utils::pin_mut;

pub struct Executor(cortex_m::Peripherals);

impl Executor {
    pub fn new(peripherals: cortex_m::Peripherals) -> Executor {
        // enable WFE
        unsafe {
            peripherals.SCB.scr.modify(|x| (x | 0b0001_0000));
        }

        Executor(peripherals)
    }

    pub fn block_on<F: Future>(self, future: F) -> F::Output {
        pin_mut!(future);

        let local_waker = EmbrioWaker::local_waker();
        let mut spawn = NoSpawn;
        let mut context = Context::new(&local_waker, &mut spawn);

        loop {
            if let Poll::Ready(val) = future.reborrow().poll(&mut context) {
                return val;
            }
            EmbrioWaker::wait();
        }
    }
}
