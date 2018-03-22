#![no_std]
#![feature(never_type)]
#![feature(pin)]

extern crate cortex_m;

mod futures {
    extern crate futures_core;
    pub extern crate futures_stable as stable;
    pub use self::futures_core::*;
}

mod executor;
mod pin;
mod waker;

pub use executor::Executor;
