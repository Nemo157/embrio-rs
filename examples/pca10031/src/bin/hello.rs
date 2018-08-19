#![no_std]
#![no_main]

#![feature(underscore_imports)]

// Link only imports, for panic implementation and interrupt vectors
#[allow(unused_imports)] // https://github.com/rust-lang/rust/issues/53128#issuecomment-414117024
use { panic_abort as _, nrf51 as _ };

use cortex_m_rt::{entry, exception, ExceptionFrame};

entry!(main);
fn main() -> ! {
    loop {
    }
}

exception!(HardFault, hard_fault);
fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

exception!(*, default_handler);
fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
