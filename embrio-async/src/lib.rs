#![no_std]
#![feature(exhaustive_patterns, generator_trait, generators, never_type)]
// TODO: Figure out to hygienically have a loop between proc-macro and library
// crates
//! This crate must not be renamed or facaded because it's referred to by name
//! from some proc-macros.

use core::{
    future::Future,
    hint::unreachable_unchecked,
    marker::PhantomPinned,
    mem::{self, MaybeUninit},
    ops::{Generator, GeneratorState},
    pin::Pin,
    ptr::{self, NonNull},
    task::{self, Poll},
};
use futures_core::stream::Stream;

pub use embrio_async_macros::embrio_async;

#[doc(hidden)]
/// Dummy trait for capturing additional lifetime bounds on `impl Trait`s
pub trait Captures<'a> {}
impl<'a, T: ?Sized> Captures<'a> for T {}

unsafe fn loosen_context_lifetime<'a>(
    context: &'a mut task::Context<'_>,
) -> &'a mut task::Context<'static> {
    mem::transmute(context)
}

unsafe fn constrain_context_lifetime<'a>(
    context: &'a mut task::Context<'static>,
) -> &'a mut task::Context<'a> {
    mem::transmute(context)
}

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
enum FutureImplState<C, G> {
    NotStarted(MaybeUninit<C>),
    Started(G),
}

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
struct FutureImpl<C, G> {
    context: MaybeUninit<NonNull<task::Context<'static>>>,
    state: FutureImplState<C, G>,
    _pinned: PhantomPinned,
}

impl<C, G> Drop for FutureImpl<C, G> {
    fn drop(&mut self) {
        if let FutureImplState::NotStarted(c) = &mut self.state {
            // MaybeUninit implies ManuallyDrop, but we must always be initialized
            unsafe { ptr::drop_in_place(c.as_mut_ptr()) }
        }
    }
}

unsafe impl<C, G> Send for FutureImpl<C, G>
where
    C: Send,
    G: Send,
{
}

impl<C, G> FutureImpl<C, G>
where
    C: FnOnce(UnsafeContextRef) -> G,
{
    fn state(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Pin<&mut G> {
        // Safety: Trust me 😉
        // TODO: Actual reasons this is safe (briefly, we trust the function
        // passed to make_future to only use the pointer we gave it when we
        // resume the generator it returned, during that time we have updated it
        // to the context reference we just got, the pointer is a
        // self-reference from the generator back into our state, but we don't
        // create it until we have observed ourselves in a pin so we know we
        // can't have moved between creating the pointer and the generator ever
        // using the pointer so it is safe to dereference).
        let this = unsafe { Pin::get_unchecked_mut(self) };
        if let FutureImplState::NotStarted(c) = &mut this.state {
            let c = unsafe { c.as_ptr().read() };
            let gen = c(UnsafeContextRef(this.context.as_mut_ptr()));
            this.state = FutureImplState::Started(gen);
        }

        unsafe {
            this.context
                .as_mut_ptr()
                .write(NonNull::from(loosen_context_lifetime(cx)));
        }
        if let FutureImplState::Started(g) = &mut this.state {
            unsafe { Pin::new_unchecked(g) }
        } else {
            unsafe { unreachable_unchecked() }
        }
    }
}

impl<C, G> Future for FutureImpl<C, G>
where
    C: FnOnce(UnsafeContextRef) -> G,
    G: Generator<Yield = Poll<!>>,
{
    type Output = G::Return;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Self::Output> {
        match self.state(cx).resume() {
            GeneratorState::Yielded(Poll::Pending) => Poll::Pending,
            GeneratorState::Complete(x) => Poll::Ready(x),
        }
    }
}

impl<C, G> Stream for FutureImpl<C, G>
where
    C: FnOnce(UnsafeContextRef) -> G,
    G: Generator<Return = ()>,
    G::Yield: IsPoll,
{
    type Item = <G::Yield as IsPoll>::Ready;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match self.state(cx).resume() {
            GeneratorState::Yielded(x) => x.into_poll().map(Some),
            GeneratorState::Complete(()) => Poll::Ready(None),
        }
    }
}

/// `Send`-able wrapper around a `*mut *mut Context`
///
/// This exists to allow the generator inside a `FutureImpl` to be `Send`,
/// provided there are no other `!Send` things in the body of the generator.
pub struct UnsafeContextRef(*mut NonNull<task::Context<'static>>);

impl UnsafeContextRef {
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
        constrain_context_lifetime((*self.0).as_mut())
    }
}

unsafe impl Send for UnsafeContextRef {}

/// # Safety
///
/// This must only be called by the `#[embrio_async]` proc-macro.
///
/// (The provided function and generator must obey safety invariants documented
/// elsewhere).
pub unsafe fn make_future<C, G>(c: C) -> impl Future<Output = G::Return>
where
    C: FnOnce(UnsafeContextRef) -> G,
    G: Generator<Yield = Poll<!>>,
{
    FutureImpl {
        context: MaybeUninit::uninit(),
        state: FutureImplState::NotStarted(MaybeUninit::new(c)),
        _pinned: PhantomPinned,
    }
}

/// # Safety
///
/// This must only be called by the `#[embrio_async]` proc-macro.
///
/// (The provided function and generator must obey safety invariants documented
/// elsewhere).
pub unsafe fn make_stream<T, C, G>(c: C) -> impl Stream<Item = T>
where
    C: FnOnce(UnsafeContextRef) -> G,
    G: Generator<Yield = Poll<T>, Return = ()>,
{
    FutureImpl {
        context: MaybeUninit::uninit(),
        state: FutureImplState::NotStarted(MaybeUninit::new(c)),
        _pinned: PhantomPinned,
    }
}

fn _check_send() -> impl Future<Output = u8> + Send {
    unsafe {
        make_future(move |lw_ref| {
            move || {
                if false {
                    yield Poll::Pending
                }

                let _lw_ref = lw_ref;

                5
            }
        })
    }
}
