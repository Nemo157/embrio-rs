#![feature(
    arbitrary_self_types,
    async_await,
    await_macro,
    generator_trait,
    generators,
    proc_macro_hygiene
)]

use embrio_async::{async_block, async_stream_block, await};
use ergo_pin::ergo_pin;
use futures::{executor::block_on, stream::StreamExt};
use futures_test::future::FutureTestExt;

#[test]
fn smoke() {
    let future = async_block! {
        await!(async { 5 }.pending_once())
    };
    assert_eq!(block_on(future), 5);
}

#[test]
#[ergo_pin]
fn smoke_stream() {
    let mut stream = pin!(async_stream_block! {
        yield await!(async { 5 }.pending_once());
        yield await!(async { 6 }.pending_once());
    });
    assert_eq!(block_on(stream.next()), Some(5));
    assert_eq!(block_on(stream.next()), Some(6));
    assert_eq!(block_on(stream.next()), None);
}
