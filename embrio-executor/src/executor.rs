use core::mem::Pin;

use cortex_m;
use futures::{Async, stable::StableFuture, task::{Context, LocalMap}};

use pin::pinned;
use EmbrioWaker;

pub struct Executor(cortex_m::Peripherals);

impl Executor {
    pub fn new(peripherals: cortex_m::Peripherals) -> Executor {
        // enable WFE
        unsafe {
            peripherals
                .SCB
                .scr
                .modify(|x| (x | 0b0001_0000));
        }

        Executor(peripherals)
    }

    pub fn block_on<F: StableFuture>(
        self,
        future: F,
    ) -> Result<F::Item, F::Error> {
        let (mut map, waker) = (LocalMap::new(), EmbrioWaker::waker());
        let mut context = Context::without_spawn(&mut map, &waker);

        pinned(future, |mut future| {
            loop {
                let future = Pin::borrow(&mut future);
                match future.poll(&mut context) {
                    Ok(Async::Ready(val)) => return Ok(val),
                    Ok(Async::Pending) => (),
                    Err(e) => return Err(e),
                }
                EmbrioWaker::wait();
            }
        })
    }
}
