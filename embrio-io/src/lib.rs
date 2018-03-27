#![no_std]
#![feature(arbitrary_self_types)]
#![feature(never_type)]
#![feature(pin)]

mod futures {
    extern crate futures_core;
    pub extern crate futures_stable as stable;
    pub use self::futures_core::*;
}

mod read;
mod write;
mod sink;

pub use read::Read;
pub use write::Write;
pub use sink::sink;
