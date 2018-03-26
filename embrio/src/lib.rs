extern crate embrio_core;

#[cfg(feature = "executor")]
extern crate embrio_executor;

#[cfg(feature = "nrf51")]
extern crate embrio_nrf51;

pub mod gpio {
    pub use embrio_core::gpio::Output;
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
        pub use embrio_nrf51::gpio::digital::Pin;
        pub use embrio_nrf51::gpio::Pins;
    }
}
