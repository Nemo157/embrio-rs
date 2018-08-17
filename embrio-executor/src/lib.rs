#![no_std]
#![feature(
    arbitrary_self_types,
    async_await,
    cfg_target_has_atomic,
    const_fn,
    futures_api,
    never_type,
    pin,
)]

mod executor;
mod spawn;
mod waker;

pub use self::{
    executor::Executor, waker::EmbrioWaker,
};
