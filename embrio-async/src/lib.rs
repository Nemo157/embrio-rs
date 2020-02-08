#![no_std]
#![feature(exhaustive_patterns, generator_trait, generators, never_type)]
// TODO: Figure out to hygienically have a loop between proc-macro and library
// crates
//! This crate must not be renamed or facaded because it's referred to by name
//! from some proc-macros.

use core::{
    future::Future,
    mem,
    ops::{Generator, GeneratorState},
    pin::Pin,
    task::{self, Poll},
    ptr::NonNull,
};
use futures_core::stream::Stream;

pub use embrio_async_macros::embrio_async;

#[doc(hidden)]
/// Dummy trait for capturing additional lifetime bounds on `impl Trait`s
pub trait Captures<'a> {}
impl<'a, T: ?Sized> Captures<'a> for T {}

trait IsPoll {
    type Ready;

    fn into_poll(self) -> Poll<Self::Ready>;
}

impl<T> IsPoll for Poll<T> {
    type Ready = T;

    fn into_poll(self) -> Poll<<Self as IsPoll>::Ready> {
        self
    }
}

// `C` is a closure that is passed to `::embrio_async::make_future`. `G` is a generator that is
// returned by `C`.
// An `async` block is tranformed into a `FutureImpl`, where initially `C` is a closure of the form
// below, which when called returns a generator `G`.
//
// ```
// |mut _embrio_async_contex_argument: UnsafeContextRef| -> [static generator@...] {
//     static move || {
//         if false {
//             yield ::core::task::Poll::Pending
//         }
//         {
//             ...
//         }
//     }
// }
// ```
//
// when `FutureImpl` is first `poll`ed (or `poll_next`ed), `FutureImpl`'s `state` field goes from
// `FutureImplState::NotStarted(C)` to `FutureImplState::Started(G)` and the generator executes
// till the first yield point.
pin_project_lite::pin_project! {
    struct FutureImpl<G> {
        #[pin]
        generator: G,
    }
}

impl<G> Future for FutureImpl<G>
where
    G: Generator<UnsafeContextRef, Yield = Poll<!>>,
{
    type Output = G::Return;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Self::Output> {
        match self.project().generator.resume(unsafe { UnsafeContextRef::new(cx) }) {
            GeneratorState::Yielded(Poll::Pending) => Poll::Pending,
            GeneratorState::Complete(x) => Poll::Ready(x),
        }
    }
}

impl<G> Stream for FutureImpl<G>
where
    G: Generator<UnsafeContextRef, Return = ()>,
    <G as Generator<UnsafeContextRef>>::Yield: IsPoll,
{
    type Item = <G::Yield as IsPoll>::Ready;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match self.project().generator.resume(unsafe { UnsafeContextRef::new(cx) }) {
            GeneratorState::Yielded(x) => x.into_poll().map(Some),
            GeneratorState::Complete(()) => Poll::Ready(None),
        }
    }
}

/// `Send`-able wrapper around a `*mut Context`
///
/// This exists to allow the generator inside a `FutureImpl` to be `Send`,
/// provided there are no other `!Send` things in the body of the generator.
pub struct UnsafeContextRef(NonNull<task::Context<'static>>);

impl UnsafeContextRef {
    pub unsafe fn new(cx: &mut task::Context<'_>) -> Self {
        unsafe fn eliminate_context_lifetimes<'a>(
            context: &mut task::Context<'_>,
        ) -> NonNull<task::Context<'static>> {
            mem::transmute(context)
        }

        UnsafeContextRef(eliminate_context_lifetimes(cx))
    }

    /// Get a reference to the wrapped context
    ///
    /// # Safety
    ///
    /// This must only be called from the `await!` macro within the
    /// `make_future` function, which will in turn only be run when the
    /// `FutureImpl` has been observed to be in a `Pin`, guaranteeing that the
    /// outer `*const` remains valid.
    // https://github.com/rust-lang/rust-clippy/issues/2906
    pub unsafe fn get_context(&mut self) -> &mut task::Context<'_> {
        unsafe fn reattach_context_lifetimes<'a>(
            context: NonNull<task::Context<'static>>,
        ) -> &'a mut task::Context<'a> {
            mem::transmute(context)
        }

        reattach_context_lifetimes(self.0)
    }
}

unsafe impl Send for UnsafeContextRef {}

/// # Safety
///
/// This must only be called by the `#[embrio_async]` proc-macro.
///
/// (The provided generator must obey safety invariants documented elsewhere).
pub unsafe fn make_future<G>(generator: G) -> impl Future<Output = G::Return>
where
    G: Generator<UnsafeContextRef, Yield = Poll<!>>,
{
    FutureImpl { generator }
}

/// # Safety
///
/// This must only be called by the `#[embrio_async]` proc-macro.
///
/// (The provided generator must obey safety invariants documented elsewhere).
pub unsafe fn make_stream<T, G>(generator: G) -> impl Stream<Item = T>
where
    G: Generator<UnsafeContextRef, Return = (), Yield = Poll<T>>,
{
    FutureImpl { generator }
}

fn _check_send() -> impl Future<Output = u8> + Send {
    unsafe {
        make_future(move |_: UnsafeContextRef| {
            if false {
                let _ = yield Poll::Pending;
            }

            5
        })
    }
}
