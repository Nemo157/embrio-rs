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
    task::{Poll, LocalWaker},
    future::Future,
    ops::{Generator, GeneratorState},
    pin::Pin,
    ptr,
    mem,
};

#[doc(hidden)]
pub use core as _core;

enum FutureImplState<F, G> {
    NotStarted(F),
    Started(G),
    Invalid,
}

struct FutureImpl<F, G> {
    local_waker: *const LocalWaker,
    state: FutureImplState<F, G>
}

impl<F, G> Future for FutureImpl<F, G>
where
    F: FnOnce(*const *const LocalWaker) -> G,
    G: Generator<Yield = ()>
{
    type Output = G::Return;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        let this = unsafe { Pin::get_mut_unchecked(self) };
        if let FutureImplState::Started(g) = &mut this.state {
            unsafe {
                this.local_waker = lw as *const _;
                match g.resume() {
                    GeneratorState::Yielded(()) => Poll::Pending,
                    GeneratorState::Complete(x) => Poll::Ready(x),
                }
            }
        } else if let FutureImplState::NotStarted(f) = mem::replace(&mut this.state, FutureImplState::Invalid) {
            this.state = FutureImplState::Started(f(&this.local_waker as *const _));
            unsafe { Pin::new_unchecked(this) }.poll(lw)
        } else {
            panic!("reached invalid state")
        }
    }
}

pub fn make_future<F, G>(f: F) -> impl Future<Output = G::Return>
where
    F: FnOnce(*const *const LocalWaker) -> G,
    G: Generator<Yield = ()>,
{
    FutureImpl {
        local_waker: ptr::null(),
        state: FutureImplState::NotStarted(f),
    }
}

#[macro_export]
macro_rules! await {
    ($lw:ident, $e:expr) => {{
        let mut pinned = $e;
        loop {
            // Safety: Trust me
            let polled = unsafe {
                let pin = $crate::_core::pin::Pin::new_unchecked(&mut pinned);
                $crate::_core::future::Future::poll(pin, &**$lw)
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
    ($lw:ident, { $($s:stmt);* }) => {{
        $crate::make_future(move |$lw| {
            static move || {
                if false { yield }
                $($s)*
            }
        })
    }}
}
