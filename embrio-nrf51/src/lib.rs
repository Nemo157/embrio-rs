#![no_std]

#![feature(conservative_impl_trait)]
#![feature(duration_extras)]
#![feature(never_type)]
#![feature(underscore_lifetimes)]
#![feature(in_band_lifetimes)]

extern crate cortex_m;
extern crate embrio_core;
extern crate nrf51;

mod futures {
    extern crate futures_core;
    extern crate futures_util;
    pub use self::futures_core::*;
    pub use self::futures_util::*;
}

mod zst_ref;

pub mod timer;
pub mod gpio;
