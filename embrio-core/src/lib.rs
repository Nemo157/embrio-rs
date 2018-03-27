#![no_std]
#![feature(arbitrary_self_types)]
#![feature(never_type)]
#![feature(pin)]

mod futures {
    extern crate futures_core;
    pub use self::futures_core::*;
}

pub mod gpio;
pub mod io;
