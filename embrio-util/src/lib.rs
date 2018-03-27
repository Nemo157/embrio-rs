#![no_std]
#![feature(arbitrary_self_types)]
#![feature(conservative_impl_trait)]
#![feature(core_intrinsics)]
#![feature(duration_extras)]
#![feature(in_band_lifetimes)]
#![feature(never_type)]
#![feature(pin)]
#![feature(underscore_lifetimes)]
#![feature(proc_macro)]
#![feature(generators)]

mod embrio {
    extern crate embrio_core;
    pub use self::embrio_core::*;
}

extern crate futures_await as futures;

#[macro_use]
mod macros;

pub mod io;
