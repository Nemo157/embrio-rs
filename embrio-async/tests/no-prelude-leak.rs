#![no_std]
#![no_implicit_prelude]
#![feature(generators, core_panic_info)]

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
