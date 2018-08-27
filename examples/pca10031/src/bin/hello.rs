#![no_std]
#![no_main]

#![feature(underscore_imports)]

// Link only imports, for panic implementation and interrupt vectors
#[allow(unused_imports)] // https://github.com/rust-lang/rust/issues/53128#issuecomment-414117024
use { panic_abort as _, nrf51 as _ };

use cortex_m_rt::{entry, exception, ExceptionFrame};
use embrio_nrf51::{EmbrioNrf51, uart::BAUDRATEW, interrupts};

entry!(main);
fn main() -> ! {
    let mut nrf51 = EmbrioNrf51::take().unwrap();
    let mut txpin = nrf51.pins.9.output().push_pull();
    let mut rxpin = nrf51.pins.11.input().floating();
    let (tx, rx) = nrf51.uart.init(&mut txpin, &mut rxpin, BAUDRATEW::BAUD115200);
    unsafe {
        hello::main(rx, tx)
    }.unwrap();
    unreachable!()
}

exception!(HardFault, hard_fault);
fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

exception!(*, default_handler);
fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}

interrupts!();
