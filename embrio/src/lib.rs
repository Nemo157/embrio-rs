#![no_std]

extern crate embrio_core;
extern crate embrio_util;

#[cfg(feature = "executor")]
extern crate embrio_executor;

#[cfg(feature = "nrf51")]
extern crate embrio_nrf51;

pub mod fmt {
    pub use embrio_util::{await_write, await_writeln};
}

pub mod gpio {
    pub use embrio_core::gpio::Output;
}

pub mod timer {
    pub use embrio_core::timer::Timer;
}

pub mod io {
    pub use embrio_core::io::{void, Cursor, Read, Write};
    pub use embrio_util::io::{close, flush, read_exact, read_until, write_all, BufReader};
}

#[cfg(feature = "executor")]
pub use embrio_executor::Executor;

#[cfg(feature = "nrf51")]
pub mod nrf51 {
    pub mod timer {
        pub use embrio_nrf51::timer::Timer;
    }

    pub mod gpio {
        pub use embrio_nrf51::gpio::{Pin, Pins};

        pub mod mode {
            pub use embrio_nrf51::gpio::mode::{
                Disabled, Floating, Input, InputMode, OpenDrain, Output,
                OutputMode, PinMode, PullDown, PullUp, PushPull,
            };
        }
    }

    pub mod uart {
        pub use embrio_nrf51::uart::{Uart, BAUDRATEW};
    }
}
