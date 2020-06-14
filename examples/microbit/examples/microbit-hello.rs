#![no_std]
#![no_main]
// workaround https://github.com/rust-embedded/cortex-m-rt/issues/225
#![allow(clippy::missing_safety_doc)]

// Link only imports, for panic implementation and interrupt vectors
use {nrf51 as _, panic_abort as _};

use cortex_m_rt::{entry, exception, ExceptionFrame};
use embrio_nrf51::{uart::BAUDRATE_A, EmbrioNrf51};

#[entry]
fn main() -> ! {
    let mut nrf51 = EmbrioNrf51::take().unwrap();
    let mut txpin = nrf51.pins.24.output().push_pull();
    let mut rxpin = nrf51.pins.25.input().floating();
    let (tx, rx) =
        nrf51
            .uart
            .init(&mut txpin, &mut rxpin, BAUDRATE_A::BAUD115200);
    unsafe { hello::main(rx, tx) }.unwrap();
    unreachable!()
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
