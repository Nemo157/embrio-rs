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
#![feature(macro_at_most_once_rep)]

pub mod embrio {
    extern crate embrio_core;
    pub use self::embrio_core::*;
}

pub extern crate futures;

pub mod fmt;
pub mod future;
pub mod io;
