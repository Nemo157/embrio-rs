#![no_std]
#![feature(
    arbitrary_self_types,
    async_await,
    const_fn,
    core_intrinsics,
    futures_api,
    in_band_lifetimes,
    never_type,
    pin,
    specialization,
)]

mod zst_ref;

pub mod gpio;
pub mod timer;
pub mod uart;

use nrf51::interrupt;

interrupt!(UART0, self::uart::Uart::interrupt);
interrupt!(TIMER0, self::timer::Timer::<nrf51::TIMER0>::interrupt);
interrupt!(TIMER1, self::timer::Timer::<nrf51::TIMER1>::interrupt);
