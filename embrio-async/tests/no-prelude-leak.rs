#![no_implicit_prelude]
#![feature(
    arbitrary_self_types,
    async_await,
    await_macro,
    futures_api,
    generator_trait,
    generators,
    proc_macro_hygiene
)]

// This is using no_implicit_prelude to test that the macros don't accidentally
// refer directly to any paths from core's implicitly injected prelude and
// instead everything is going through the internal re-export.

#[test]
fn smoke() {
    let future = async {
        ::std::await!(::embrio_async::async_block! {
            ::embrio_async::await!(async { 5 })
        })
    };
    ::std::assert_eq!(::futures::executor::block_on(future), 5);
}

#[test]
fn smoke_stream() {
    let future = async {
        let stream = ::embrio_async::async_stream_block! {
            yield ::embrio_async::await!(async { 5 });
            yield ::embrio_async::await!(async { 6 });
        };
        ::pin_utils::pin_mut!(stream);
        let mut sum = 0;
        while let ::std::option::Option::Some(val) =
            ::std::await!(::futures::stream::StreamExt::next(&mut stream))
        {
            sum += val;
        }
        sum
    };
    ::std::assert_eq!(::futures::executor::block_on(future), 11);
}
