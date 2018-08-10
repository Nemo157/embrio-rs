#![no_std]
#![feature(
    arbitrary_self_types,
    async_await,
    await_macro,
    const_fn,
    futures_api,
    never_type,
    pin,
)]

mod context;
mod executor;
mod spawn;
mod waker;

pub use self::{
    context::EmbrioContext, executor::Executor, waker::EmbrioWaker,
};
