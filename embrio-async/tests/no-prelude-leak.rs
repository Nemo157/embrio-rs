#![no_std]
#![no_implicit_prelude]
#![feature(generators)]

extern crate embrio_async;

use embrio_async::embrio_async;

// This is using no_implicit_prelude to test that the macros don't accidentally
// refer directly to any paths from core's implicitly injected prelude and
// instead everything is going through the internal re-export.

#[embrio_async]
#[test]
fn smoke() {
    let future = async { async { async { 5 }.await }.await };
    {
        use ::core::panic;
        ::core::assert_eq!(::futures::executor::block_on(future), 5);
    }
}

#[embrio_async]
#[test]
fn smoke_stream() {
    let future = async {
        let stream = async {
            yield async { 5usize }.await;
            yield async { 6usize }.await;
        };
        ::pin_utils::pin_mut!(stream);
        let mut sum = 0usize;
        while let ::core::option::Option::Some(val) =
            ::futures::stream::StreamExt::next(&mut stream).await
        {
            sum += val;
        }
        sum
    };
    {
        use ::core::panic;
        ::core::assert_eq!(::futures::executor::block_on(future), 11);
    }
}

#[embrio_async]
#[test]
fn smoke_sink() {
    let future = async {
        let mut sum = 0;
        {
            let slow = async move |i| i;
            let stream = async {
                yield async { slow(5) }.await;
                yield async { slow(6) }.await;
            };
            let sink = async |yield input| {
                while let ::core::option::Option::Some(future) = input.await {
                    sum += future.await;
                }
                sum += 7;
            };
            ::pin_utils::pin_mut!(sink);
            let stream = ::futures::stream::StreamExt::map(
                stream,
                ::core::result::Result::Ok,
            );
            ::futures::stream::StreamExt::forward(stream, sink).await.unwrap();
        }
        sum
    };
    {
        use ::std::panic;
        ::std::assert_eq!(::futures::executor::block_on(future), 18);
    }
}

#[embrio_async]
#[test]
fn smoke_sink_typed() {
    let future = async {
        let mut sum = 0;
        {
            let stream = async {
                yield 5;
                yield 6;
            };
            let sink = async |yield input: u32| {
                while let ::core::option::Option::Some(value) = input.await {
                    sum += value;
                }
                sum += 7;
            };
            ::pin_utils::pin_mut!(sink);
            let stream = ::futures::stream::StreamExt::map(
                stream,
                ::core::result::Result::Ok,
            );
            ::futures::stream::StreamExt::forward(stream, sink).await.unwrap();
        }
        sum
    };
    {
        use ::std::panic;
        ::std::assert_eq!(::futures::executor::block_on(future), 18);
    }
}
