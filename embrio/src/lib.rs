#![no_std]

extern crate embrio_core;
extern crate embrio_util;

#[cfg(feature = "executor")]
extern crate embrio_executor;

#[cfg(feature = "nrf51")]
extern crate embrio_nrf51;

pub mod gpio {
    pub use embrio_core::gpio::Output;
}

pub mod io {
    pub use embrio_core::io::{sink, Read, Write, Cursor};
    pub use embrio_util::io::{read_exact, write_all, flush, close};
}

#[cfg(feature = "executor")]
pub mod executor {
    pub use embrio_executor::Executor;
}

#[cfg(feature = "nrf51")]
pub mod nrf51 {
    pub mod timer {
        pub use embrio_nrf51::timer::Timer;
    }

    pub mod gpio {
        pub use embrio_nrf51::gpio::{Pin, Pins};

        pub mod mode {
            pub use embrio_nrf51::gpio::mode::{
                InputMode, OutputMode, PinMode,
                Floating, PullUp, PullDown, PushPull, OpenDrain,
                Disabled, Input, Output,
            };
        }
    }

    pub mod uart {
        pub use embrio_nrf51::uart::{Uart, BAUDRATEW};
    }
}
