#![no_std]
#![feature(
    arbitrary_self_types,
    async_await,
    futures_api,
    generator_trait,
    generators,
    pin
)]

//! # Safety
//!
//! The current implementation is single-thread safe only. You must ensure that
//! all futures created via these macros are only polled on a single thread at a
//! time. That's not "any future created by these macros", that's "all futures
//! created by these macros", at no point can any pair of futures that came out
//! of [`async_block!`](crate::async_block) be polled on different threads.

use core::{
    ptr::NonNull,
    task,
};

#[doc(hidden)]
pub use core as _core;

pub static mut LOCAL_WAKER: Option<NonNull<task::LocalWaker>> = None;

#[macro_export]
macro_rules! await {
    ($e:expr) => {{
        let mut pinned = $e;
        loop {
            // Safety: See note on crate
            let polled = unsafe {
                let mut lw = $crate::LOCAL_WAKER.take().unwrap();
                let pin = $crate::_core::pin::Pin::new_unchecked(&mut pinned);
                let polled = $crate::_core::future::Future::poll(pin, lw.as_mut());
                // Note: using assert_eq here because assert has hygiene issues
                // on 2018, see https://github.com/rust-lang/rust/issues/56389
                $crate::_core::assert_eq!($crate::LOCAL_WAKER.replace(lw).is_none(), true);
                polled
            };
            if let $crate::_core::task::Poll::Ready(x) = polled {
                break x;
            }
            yield
        }
    }};
}

#[macro_export]
macro_rules! async_block {
    ($($s:stmt);*) => {{
        struct FutureImpl<G>(G);

        impl<G> $crate::_core::future::Future for FutureImpl<G>
        where
            G: $crate::_core::ops::Generator<Yield = ()>
        {
            type Output = G::Return;

            fn poll(
                self: $crate::_core::pin::Pin<&mut Self>,
                lw: &$crate::_core::task::LocalWaker
            ) -> $crate::_core::task::Poll<Self::Output>
            {
                // Safety: See note on crate
                #[allow(clippy::cast_ptr_alignment)]
                unsafe {
                    let lw = $crate::_core::ptr::NonNull::new_unchecked(lw as *const _ as *mut _);
                    $crate::_core::assert_eq!($crate::LOCAL_WAKER.replace(lw).is_none(), true);
                    let poll = match $crate::_core::pin::Pin::get_mut_unchecked(self).0.resume() {
                        $crate::_core::ops::GeneratorState::Yielded(())
                            => $crate::_core::task::Poll::Pending,
                        $crate::_core::ops::GeneratorState::Complete(x)
                            => $crate::_core::task::Poll::Ready(x),
                    };
                    $crate::_core::assert_eq!($crate::LOCAL_WAKER.take().is_some(), true);
                    poll
                }
            }
        }

        FutureImpl(static move || {
            if false { yield }
            $($s)*
        })
    }}
}
