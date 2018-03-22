extern crate embrio_executor;
extern crate embrio_nrf51;

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
