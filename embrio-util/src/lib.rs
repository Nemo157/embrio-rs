#![no_std]
#![feature(arbitrary_self_types)]
#![feature(core_intrinsics)]
#![feature(duration_extras)]
#![feature(generators)]
#![feature(in_band_lifetimes)]
#![feature(never_type)]
#![feature(pin)]
#![feature(proc_macro)]
#![feature(underscore_lifetimes)]

mod embrio {
    extern crate embrio_core;
    pub use self::embrio_core::*;
}

extern crate futures;

pub mod future;
pub mod io;
