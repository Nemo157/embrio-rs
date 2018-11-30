#![no_std]
#![feature(
    arbitrary_self_types,
    async_await,
    futures_api,
    generator_trait,
    generators,
    pin
)]

use core::{
    task::{Poll, LocalWaker},
    future::Future,
    ops::{Generator, GeneratorState},
    pin::Pin,
    ptr,
    mem,
};

//! Must not be renamed or facaded

pub use embrio_async_dehygiene::{async_block, await};

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
