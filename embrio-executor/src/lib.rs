#![no_std]
#![feature(
    arbitrary_self_types,
    async_await,
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
