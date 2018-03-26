#[cfg(feature = "executor")]
pub mod executor {
    extern crate embrio_executor;

    pub use self::embrio_executor::Executor;
}

#[cfg(feature = "nrf51")]
pub mod nrf51 {
    extern crate embrio_nrf51;

    pub mod timer {
        pub use super::embrio_nrf51::timer::Timer;
    }

    pub mod gpio {
        pub use super::embrio_nrf51::gpio::digital::Pin;
        pub use super::embrio_nrf51::gpio::Pins;
    }
}
