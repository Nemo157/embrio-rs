#![no_std]
#![feature(
    arbitrary_self_types,
    async_await,
    const_fn,
    core_intrinsics,
    futures_api,
    in_band_lifetimes,
    never_type,
    option_replace,
    pin,
    specialization,
)]

mod zst_ref;

pub mod gpio;
pub mod timer;
pub mod uart;

#[doc(hidden)]
pub use nrf51::interrupt;

/// This **MUST** be called in any binary that depends on this crate, for some
/// reason linking the interrupt handlers in when they're defined in a
/// dependency doesn't work.
#[macro_export]
macro_rules! interrupts {
    () => {
        $crate::interrupt!(UART0, $crate::uart::Uart::interrupt);
        $crate::interrupt!(TIMER0, $crate::timer::Timer::<nrf51::TIMER0>::interrupt);
        $crate::interrupt!(TIMER1, $crate::timer::Timer::<nrf51::TIMER1>::interrupt);
    }
}

use self::gpio::Pins;
use self::uart::Uart;

pub struct EmbrioNrf51<'b> {
    pub pins: Pins<'b>,
    pub uart: Uart<'b>,
}

impl<'b> EmbrioNrf51<'b> {
    pub fn new(
        cortex_m: &'b mut cortex_m::Peripherals,
        nrf51: &'b mut nrf51::Peripherals,
    ) -> EmbrioNrf51<'b> {
        let pins = Pins::new(&mut nrf51.GPIO);
        let uart = Uart::new(&mut nrf51.UART0, &mut cortex_m.NVIC);

        EmbrioNrf51 { pins, uart }
    }
}
