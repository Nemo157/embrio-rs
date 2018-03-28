use core::mem::Pin;
use core::u32;

use cortex_m;
use futures::{Async, stable::StableFuture, task::{Context, LocalMap, Waker}};

use pin::pinned;
use waker::WFEWaker;

pub struct Executor(cortex_m::Peripherals);

impl Executor {
    pub fn new(peripherals: cortex_m::Peripherals) -> Executor {
        // enable WFE
        unsafe {
            peripherals.SCB.scr.modify(|x| (x | 0b00010000));
        }

        Executor(peripherals)
    }

    pub fn block_on<F: StableFuture>(
        self,
        future: F,
    ) -> Result<F::Item, F::Error> {
        let (mut map, waker) = (LocalMap::new(), Waker::from(WFEWaker));
        let mut context = Context::without_spawn(&mut map, &waker);

        pinned(future, |mut future| {
            loop {
                let future = Pin::borrow(&mut future);
                if let Async::Ready(val) = future.poll(&mut context)? {
                    return Ok(val);
                }
                // Clear all pending interrupts
                // TODO: armv7-m allows for a device specific number of interrupts
                unsafe {
                    self.0.NVIC.icpr[0].write(u32::MAX);
                }
                cortex_m::asm::wfe();
            }
        })
    }
}
