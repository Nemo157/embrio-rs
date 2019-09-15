#![no_std]
#![feature(
    arbitrary_self_types,
    const_fn,
    in_band_lifetimes,
    never_type,
    specialization
)]

mod zst_ref;

pub mod gpio;
pub mod timer;
pub mod uart;

use core::{cell::UnsafeCell, ptr};

use cortex_m::interrupt::{free, Mutex};

use self::{gpio::Pins, uart::Uart};

#[doc(hidden)]
pub use nrf51::interrupt;

pub struct EmbrioNrf51<'b> {
    pub pins: Pins<'b>,
    pub uart: Uart<'b>,
}

impl<'b> EmbrioNrf51<'b> {
    pub fn new(nrf51: &'b mut nrf51::Peripherals) -> EmbrioNrf51<'b> {
        let pins = Pins::new(&mut nrf51.GPIO);
        let uart = Uart::new(&mut nrf51.UART0);

        EmbrioNrf51 { pins, uart }
    }

    pub fn take() -> Option<EmbrioNrf51<'static>> {
        struct StaticPeripherals {
            nrf51: nrf51::Peripherals,
        }

        struct StaticContext {
            flag: UnsafeCell<bool>,
            peripherals: UnsafeCell<Option<StaticPeripherals>>,
        }

        // Safety: We return a non-Send `EmbrioNrf51`, so the
        // static references to the peripherals cannot leak across contexts
        unsafe impl Sync for StaticContext {}
        unsafe impl Send for StaticContext {}

        static CONTEXT: Mutex<StaticContext> = Mutex::new(StaticContext {
            flag: UnsafeCell::new(false),
            peripherals: UnsafeCell::new(None),
        });

        free(|c| {
            let nrf51 = nrf51::Peripherals::take()?;

            let context = CONTEXT.borrow(c);

            // Safety: This flag is only accessed from within this critical
            // section
            unsafe {
                let flag = context.flag.get();
                if ptr::read_volatile(flag) {
                    return None;
                }
                ptr::write_volatile(flag, true);
            }

            // Safety: The above flag guarantees the following code is only run
            // once
            let peripherals = unsafe { &mut *context.peripherals.get() };

            peripherals.replace(StaticPeripherals { nrf51 });

            let peripherals = peripherals.as_mut().unwrap();
            let nrf51 = &mut peripherals.nrf51;

            Some(EmbrioNrf51::new(nrf51))
        })
    }
}

#[interrupt]
fn UART0() {
    uart::Uart::interrupt()
}

#[interrupt]
fn TIMER0() {
    timer::Timer::<nrf51::TIMER0>::interrupt()
}

#[interrupt]
fn TIMER1() {
    timer::Timer::<nrf51::TIMER1>::interrupt()
}
