#![no_std]
#![feature(arbitrary_self_types)]
#![feature(core_intrinsics)]
#![feature(duration_extras)]
#![feature(in_band_lifetimes)]
#![feature(never_type)]
#![feature(pin)]
#![feature(underscore_lifetimes)]
#![feature(specialization)]

extern crate cortex_m;
extern crate nrf51;
#[macro_use]
extern crate uom;

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

mod units {
    ISQ! {
        ::uom::si,
        u32,
        (meter, kilogram, microsecond, ampere, kelvin, mole, candela)
    }
}

mod zst_ref;

pub mod gpio;
pub mod timer;
pub mod uart;
