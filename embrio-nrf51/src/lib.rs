#![no_std]

#![feature(conservative_impl_trait)]
#![feature(never_type)]
#![feature(underscore_lifetimes)]
#![feature(duration_extras)]

extern crate cortex_m;
extern crate futures;
extern crate nrf51;

pub mod timer;
pub mod gpio;
