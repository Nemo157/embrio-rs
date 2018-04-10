#![no_std]
#![feature(arbitrary_self_types)]
#![feature(core_intrinsics)]
#![feature(duration_extras)]
#![feature(in_band_lifetimes)]
#![feature(never_type)]
#![feature(pin)]
#![feature(underscore_lifetimes)]
#![feature(specialization)]
#![allow(unknown_lints)]

extern crate cortex_m;
extern crate nrf51;

#[macro_use]
extern crate futures_core;

mod embrio {
    extern crate embrio_core;
    pub extern crate embrio_executor as executor;
    pub use self::embrio_core::*;
}

mod futures {
    extern crate futures_util;
    pub use futures_core::*;
    pub use self::futures_util::*;
}

mod zst_ref;

pub mod gpio;
pub mod timer;
pub mod uart;
