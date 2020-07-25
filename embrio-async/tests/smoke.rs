#![no_std]
#![feature(generators, async_closure)]

use embrio_async::embrio_async;
use ergo_pin::ergo_pin;
use futures::{executor::block_on, stream::StreamExt};
use futures_test::future::FutureTestExt;

#[embrio_async]
#[test]
fn smoke() {
    let future = async { async { 5 }.pending_once().await };
    assert_eq!(block_on(future), 5);
}

#[embrio_async]
#[test]
fn test_async_block() {
    let f = async { 5usize };

    let f2 = async { f.await };
    assert_eq!(block_on(f2), 5);
}

#[test]
#[ergo_pin]
#[embrio_async]
fn smoke_stream() {
    let mut stream = pin!(async {
        yield async { 5 }.pending_once().await;
        yield async { 6 }.pending_once().await;
    });
    assert_eq!(block_on(stream.next()), Some(5));
    assert_eq!(block_on(stream.next()), Some(6));
    assert_eq!(block_on(stream.next()), None);
}

#[derive(Eq, PartialEq, Debug)]
enum Either<L, R> {
    Left(L),
    Right(R),
}

#[embrio_async]
async fn a_number_and_string<'a>(
    n: &usize,
    s: &'a str,
) -> Either<usize, &'a str> {
    if *n % 2 == 0 {
        Either::Left(*n)
    } else {
        Either::Right(s)
    }
}

#[embrio_async]
async fn a_wait_thing() -> Either<usize, &'static str> {
    a_number_and_string(&5, "Hello, world!").await
}

#[embrio_async]
async fn anonymous_lifetime(f: &mut core::fmt::Formatter<'_>) {
    let _ = write!(f, "Hello, world!");
}

#[test]
fn smoke_async_fn() {
    assert_eq!(block_on(a_wait_thing()), Either::Right("Hello, world!"));
}

#[allow(dead_code)]
struct Foo(usize);

impl Foo {
    #[embrio_async]
    async fn bar(&self) -> &usize {
        &self.0
    }
    #[embrio_async]
    async fn baz(&mut self) -> &usize {
        &self.0
    }
    #[embrio_async]
    async fn spam(&mut self, f: fn(&mut usize)) {
        f(&mut self.0)
    }
}

#[embrio_async]
#[test]
fn smoke_sink() {
    let future = async {
        let mut sum = 0;
        {
            let slow = move |i| async move { i };
            let stream = async {
                yield async { slow(5) }.await;
                yield async { slow(6) }.await;
            };
            let sink = async || -> Result<(), ()> {
                while let Some(future) = yield () {
                    sum += future.await;
                }
                sum += 7;
                Ok(())
            };
            stream.map(Ok).forward(sink).await.unwrap();
        }
        sum
    };
    assert_eq!(block_on(future), 18);
}

#[embrio_async]
#[test]
fn smoke_sink_typed() {
    let future = async {
        let mut sum = 0;
        {
            let stream = async {
                yield Ok(5);
                yield Ok(6);
            };
            let sink = async |_: Result<u32, u64>| -> Result<(), u64> {
                while let Some(value) = (yield) {
                    sum += value?;
                }
                sum += 7;
                Ok(())
            };
            stream.map(Ok).forward(sink).await.unwrap();
        }
        sum
    };
    assert_eq!(block_on(future), 18);
}
