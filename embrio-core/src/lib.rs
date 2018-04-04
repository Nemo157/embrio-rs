#![no_std]
#![feature(arbitrary_self_types)]
#![feature(never_type)]
#![feature(pin)]

#[macro_use]
extern crate uom;

extern crate futures;

pub mod si {
    // Until https://github.com/iliekturtles/uom/issues/60 is fixed we can't
    // allow using conversions on embedded devices, so redefine a minimal system
    // of quantities and system of units covering just what we currently need.
    //
    // In the future this will be replaced by a specific system of units based
    // on the standard `uom::si::ISQ` system of quantities.

    pub mod time {
        quantity! {
            quantity: Time; "time";
            dimension: ISQ<P1>;
            units {
                @microsecond: 1.0; "µs", "microsecond", "microseconds";
            }
        }
    }

    system! {
        quantities: ISQ {
            time: microsecond, T;
        }
        units: SI {
            mod time::Time,
        }
    }

    ISQ!(si, u32);
}

pub mod gpio;
pub mod io;
pub mod timer;
