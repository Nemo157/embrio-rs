#![no_std]
#![feature(
    arbitrary_self_types,
    async_await,
    futures_api,
    generator_trait,
    generators
)]
// TODO: Figure out to hygienically have a loop between proc-macro and library
// crates
//! This crate must not be renamed or facaded because it's referred to by name
//! from some proc-macros.

use core::{
    future::Future,
    marker::PhantomPinned,
    mem,
    ops::{Generator, GeneratorState},
    pin::Pin,
    ptr,
    task::{LocalWaker, Poll},
};

pub use embrio_async_dehygiene::{async_block, await};

enum FutureImplState<F, G> {
    NotStarted(F),
    Started(G),
    Invalid,
}

struct FutureImpl<F, G> {
    local_waker: *const LocalWaker,
    state: FutureImplState<F, G>,
    _pinned: PhantomPinned,
}

impl<F, G> Future for FutureImpl<F, G>
where
    F: FnOnce(UnsafeWakeRef) -> G,
    G: Generator<Yield = ()>,
{
    type Output = G::Return;

    fn poll(self: Pin<&mut Self>, lw: &LocalWaker) -> Poll<Self::Output> {
        // Safety: Trust me 😉
        // TODO: Actual reasons this is safe (briefly, we trust the function
        // passed to make_future to only use the pointer we gave it when we
        // resume the generator it returned, during that time we have updated it
        // to the local waker reference we just got, the pointer is a
        // self-reference from the generator back into our state, but we don't
        // create it until we have observed ourselves in a pin so we know we
        // can't have moved between creating the pointer and the generator ever
        // using the pointer so it is safe to dereference).
        let this = unsafe { Pin::get_unchecked_mut(self) };
        if let FutureImplState::Started(g) = &mut this.state {
            unsafe {
                this.local_waker = lw as *const _;
                match g.resume() {
                    GeneratorState::Yielded(()) => Poll::Pending,
                    GeneratorState::Complete(x) => Poll::Ready(x),
                }
            }
        } else if let FutureImplState::NotStarted(f) =
            mem::replace(&mut this.state, FutureImplState::Invalid)
        {
            this.state = FutureImplState::Started(f(UnsafeWakeRef(
                &this.local_waker as *const _,
            )));
            unsafe { Pin::new_unchecked(this) }.poll(lw)
        } else {
            panic!("reached invalid state")
        }
    }
}

unsafe impl<F, G> Send for FutureImpl<F, G>
where
    F: Send,
    G: Send,
{
}

/// `Send`-able wrapper around a `*const *const LocalWaker`
///
/// This exists to allow the generator inside a `FutureImpl` to be `Send`,
/// provided there are no other `!Send` things in the body of the generator.
pub struct UnsafeWakeRef(*const *const LocalWaker);

impl UnsafeWakeRef {
    /// Get a reference to the wrapped waker
    ///
    /// This must only be called from the `await!`
    /// `make_future` function, which will in turn only be run when the
    /// `FutureImpl` has been observed to be in a `Pin`, guaranteeing that the
    /// outer `*const` remains valid.
    pub unsafe fn get_waker(&self) -> &LocalWaker {
        &**self.0
    }
}

unsafe impl Send for UnsafeWakeRef {}

pub unsafe fn make_future<F, G>(f: F) -> impl Future<Output = G::Return>
where
    F: FnOnce(UnsafeWakeRef) -> G,
    G: Generator<Yield = ()>,
{
    FutureImpl {
        local_waker: ptr::null(),
        state: FutureImplState::NotStarted(f),
        _pinned: PhantomPinned,
    }
}

fn _check_send() -> impl Future<Output = u8> + Send {
    unsafe {
        make_future(move |lw_ref| {
            move || {
                if false {
                    yield
                }

                let _lw_ref = lw_ref;

                5
            }
        })
    }
}
