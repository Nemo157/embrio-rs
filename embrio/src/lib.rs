extern crate embrio_executor;
extern crate embrio_nrf51;

pub mod executor {
    pub use embrio_executor::Executor;
}

#[cfg(feature = "nrf51")]
pub mod nrf51 {
}
