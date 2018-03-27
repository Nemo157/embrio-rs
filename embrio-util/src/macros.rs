#[macro_export]
macro_rules! await_poll {
    (|$cx:ident| $e:expr) => ({
        loop {
            let poll = ::futures::__rt::in_ctx(|$cx| $e);
            // Allow for #[feature(never_type)] and Future<Error = !>
            #[allow(unreachable_code, unreachable_patterns)]
            match poll {
                ::futures::__rt::core::result::Result::Ok(::futures::__rt::Async::Ready(e)) => {
                    break ::futures::__rt::core::result::Result::Ok(e)
                }
                ::futures::__rt::core::result::Result::Ok(::futures::__rt::Async::Pending) => {}
                ::futures::__rt::core::result::Result::Err(e) => {
                    break ::futures::__rt::core::result::Result::Err(e)
                }
            }
            yield ::futures::__rt::Async::Pending
        }
    })
}
