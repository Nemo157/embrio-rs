#![no_std]
#![no_main]

#![feature(underscore_imports)]

// Link only imports, for panic implementation and interrupt vectors
#[allow(unused_imports)] // https://github.com/rust-lang/rust/issues/53128#issuecomment-414117024
use { panic_abort as _, nrf51 as _ };

use cortex_m_rt::{entry, exception, ExceptionFrame};
use embrio_nrf51::{gpio::Pins, uart::{Uart, BAUDRATEW}, interrupts};

entry!(main);
fn main() -> ! {
    let core_peripherals = nrf51::CorePeripherals::take().unwrap();
    let mut peripherals = nrf51::Peripherals::take().unwrap();
    let pins = Pins::new(&mut peripherals.GPIO);
    let mut txpin = pins.9.output().push_pull();
    let mut rxpin = pins.11.input().floating();
    let uart = Uart::new(
        peripherals.UART0,
        &mut txpin,
        &mut rxpin,
        BAUDRATEW::BAUD115200,
        core_peripherals.NVIC,
    );
    let (tx, rx) = uart.split();
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
