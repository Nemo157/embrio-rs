#![no_std]
#![feature(
    arbitrary_self_types,
    async_await,
    const_fn,
    core_intrinsics,
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
    pub fn new(
        cortex_m: &'b mut cortex_m::Peripherals,
        nrf51: &'b mut nrf51::Peripherals,
    ) -> EmbrioNrf51<'b> {
        let pins = Pins::new(&mut nrf51.GPIO);
        let uart = Uart::new(&mut nrf51.UART0, &mut cortex_m.NVIC);

        EmbrioNrf51 { pins, uart }
    }

    pub fn take() -> Option<EmbrioNrf51<'static>> {
        struct StaticPeripherals {
            cortex_m: cortex_m::Peripherals,
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
            let cortex_m = cortex_m::Peripherals::take()?;
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

            peripherals.replace(StaticPeripherals { cortex_m, nrf51 });

            let peripherals = peripherals.as_mut().unwrap();
            let cortex_m = &mut peripherals.cortex_m;
            let nrf51 = &mut peripherals.nrf51;

            Some(EmbrioNrf51::new(cortex_m, nrf51))
        })
    }
}

/// This **MUST** be called in any binary that depends on this crate, for some
/// reason linking the interrupt handlers in when they're defined in a
/// dependency doesn't work.
#[macro_export]
macro_rules! interrupts {
    () => {
        $crate::interrupt!(UART0, $crate::uart::Uart::interrupt);
        $crate::interrupt!(
            TIMER0,
            $crate::timer::Timer::<nrf51::TIMER0>::interrupt
        );
        $crate::interrupt!(
            TIMER1,
            $crate::timer::Timer::<nrf51::TIMER1>::interrupt
        );
    };
}
