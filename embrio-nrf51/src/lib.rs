#![no_std]
#![feature(arbitrary_self_types)]
#![feature(conservative_impl_trait)]
#![feature(core_intrinsics)]
#![feature(duration_extras)]
#![feature(in_band_lifetimes)]
#![feature(never_type)]
#![feature(pin)]
#![feature(underscore_lifetimes)]

extern crate cortex_m;
extern crate nrf51;

mod embrio {
    extern crate embrio_core;
    pub use self::embrio_core::*;
}

mod futures {
    extern crate futures_core;
    extern crate futures_util;
    pub use self::futures_core::*;
    pub use self::futures_util::*;
}

mod zst_ref;

pub mod gpio;
pub mod timer;
pub mod uart;
