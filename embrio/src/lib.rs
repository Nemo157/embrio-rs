#![no_std]
#![feature(use_extern_macros)]

extern crate embrio_core;
extern crate embrio_util;

#[cfg(feature = "executor")]
extern crate embrio_executor;

#[cfg(feature = "nrf51")]
extern crate embrio_nrf51;

pub mod si {
    pub use embrio_core::si::{Dimension, Quantity, Time, Unit, Units};

    pub mod time {
        pub use embrio_core::si::time::{description, microsecond, Conversion,
                                        Time, Unit};
    }
}

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
    pub use embrio_core::io::{sink, Cursor, Read, Write};
    pub use embrio_util::io::{close, flush, read_exact, write_all, Error};
}

pub mod future {
    pub use embrio_util::future::{filter, filter_map, first, join, select,
                                  StableInfiniteStream};
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
            pub use embrio_nrf51::gpio::mode::{Disabled, Floating, Input,
                                               InputMode, OpenDrain, Output,
                                               OutputMode, PinMode, PullDown,
                                               PullUp, PushPull};
        }
    }

    pub mod uart {
        pub use embrio_nrf51::uart::{Uart, BAUDRATEW};
    }
}
