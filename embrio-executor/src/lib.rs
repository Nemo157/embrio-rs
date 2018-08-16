#![no_std]
#![feature(
    arbitrary_self_types,
    async_await,
    const_fn,
    futures_api,
    never_type,
    pin,
)]

use futures_core::Future;

mod executor;
mod spawn;
mod waker;

pub use self::{
    executor::Executor, waker::EmbrioWaker,
};

pub fn block_on<F: Future>(future: F) -> F::Output {
    Executor::new().block_on(future)
}
