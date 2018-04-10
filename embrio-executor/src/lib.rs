#![no_std]
#![feature(never_type)]
#![feature(pin)]
#![feature(const_fn)]
#![feature(arbitrary_self_types)]

extern crate cortex_m;

#[macro_use]
extern crate futures_core;

mod futures {
    pub extern crate futures_stable as stable;
    pub use futures_core::*;
}

mod executor;
mod pin;
mod waker;
mod context;

pub use executor::Executor;
pub use waker::EmbrioWaker;
pub use context::EmbrioContext;
