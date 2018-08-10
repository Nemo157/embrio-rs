#![no_std]
#![feature(
    arbitrary_self_types,
    async_await,
    await_macro,
    const_fn,
    core_intrinsics,
    duration_extras,
    futures_api,
    in_band_lifetimes,
    never_type,
    pin,
    specialization,
)]

mod zst_ref;

pub mod gpio;
pub mod timer;
pub mod uart;
