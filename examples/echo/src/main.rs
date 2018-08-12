#![no_std]
#![no_main]

extern crate panic_abort;
extern crate nrf51;

use cortex_m_rt::{entry, exception, ExceptionFrame};

entry!(main);
fn main() -> ! {
    examples_echo::main();
}

exception!(HardFault, hard_fault);
fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

exception!(*, default_handler);
fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
