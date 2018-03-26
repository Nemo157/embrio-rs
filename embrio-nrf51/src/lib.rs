#![no_std]

#![feature(conservative_impl_trait)]
#![feature(duration_extras)]
#![feature(never_type)]
#![feature(underscore_lifetimes)]

extern crate cortex_m;
extern crate embrio_core;
extern crate futures;
extern crate nrf51;

pub mod timer;
pub mod gpio;
