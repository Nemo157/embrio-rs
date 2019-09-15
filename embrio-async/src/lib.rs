#![no_std]
#![feature(exhaustive_patterns, generator_trait, generators, never_type)]
// TODO: Figure out to hygienically have a loop between proc-macro and library
// crates
//! This crate must not be renamed or facaded because it's referred to by name
//! from some proc-macros.

use core::{
    future::Future,
    marker::PhantomPinned,
    mem::{self, MaybeUninit},
    ops::{Generator, GeneratorState},
    pin::Pin,
    ptr,
    task::{self, Poll},
    hint::unreachable_unchecked,
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
    NotStarted(C),
    Started(FutureImpl<G>),
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

struct FutureImpl<G> {
    context: MaybeUninit<*mut task::Context<'static>>,
    state: MaybeUninit<G>,
    _pinned: PhantomPinned,
}

pub trait UnsafeDefault: Sized {
    unsafe fn unsafe_default() -> Self;
}

impl<G> UnsafeDefault for FutureImpl<G> {
    unsafe fn unsafe_default() -> Self {
        Self::uninit()
    }
}

impl<G> Drop for FutureImpl<G> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.state.as_mut_ptr());
        }
    }
}

impl<G> FutureImpl<G> {
    unsafe fn uninit() -> Self {
        FutureImpl {
            context: MaybeUninit::uninit(),
            state: MaybeUninit::uninit(),
            _pinned: PhantomPinned,
        }
    }

    fn init<C: FnOnce(UnsafeContextRef) -> G>(self: Pin<&mut Self>, c: C) -> Pin<&mut PinnedFutureImpl<G>> {
        // calling this multiple times will leak a generator
        let this = unsafe { Pin::get_unchecked_mut(self) };
        this.state = MaybeUninit::new(c(UnsafeContextRef(this.context.as_mut_ptr())));
        unsafe { this.pinned_unchecked() }
    }

    unsafe fn pinned_unchecked(&mut self) -> Pin<&mut PinnedFutureImpl<G>> {
        Pin::new_unchecked(mem::transmute(self))
    }
}

#[repr(transparent)]
struct PinnedFutureImpl<G> {
    inner: FutureImpl<G>,
}

impl<G> PinnedFutureImpl<G> {
    fn state(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Pin<&mut G> {
        unsafe {
            let this = Pin::get_unchecked_mut(self);
            this.inner.context = MaybeUninit::new(loosen_context_lifetime(cx));
            Pin::new_unchecked(&mut *this.inner.state.as_mut_ptr())
        }
    }
}

unsafe impl<G> Send for FutureImpl<G>
where
    G: Send,
{
}

impl<G> Future for PinnedFutureImpl<G>
where
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

impl<G> Stream for PinnedFutureImpl<G>
where
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

impl<C, G> FutureImplState<C, G> where
    C: FnOnce(UnsafeContextRef) -> G,
{
    fn start(self, f: Pin<&mut FutureImpl<G>>) -> Pin<&mut PinnedFutureImpl<G>> {
        // if you're able to move self, it has not yet been pinned and thus must be NotStarted
        match self {
            FutureImplState::NotStarted(c) => f.init(c),
            #[cfg(debug_assertions)]
            FutureImplState::Started(..) => unreachable!(),
            #[cfg(not(debug_assertions))]
            FutureImplState::Started(..) => unsafe { unreachable_unchecked() },
        }
    }

    fn started(self: Pin<&mut Self>) -> Pin<&mut PinnedFutureImpl<G>> {
        let this = unsafe { Pin::get_unchecked_mut(self) };
        match this {
            FutureImplState::Started(started) => unsafe { started.pinned_unchecked() },
            FutureImplState::NotStarted(..) => {
                let (c, pinned) = unsafe {
                    let c = mem::replace(this, FutureImplState::Started(FutureImpl::uninit()));
                    (match c {
                        FutureImplState::NotStarted(c) => c,
                        _ => unreachable_unchecked(),
                    }, match this {
                        FutureImplState::Started(pinned) => Pin::new_unchecked(pinned),
                        _ => unreachable_unchecked(),
                    })
                };
                pinned.init(c)
            },
        }
    }
}

impl<C, G> Future for FutureImplState<C, G>
where
    C: FnOnce(UnsafeContextRef) -> G,
    G: Generator<Yield = Poll<!>>,
{
    type Output = G::Return;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Self::Output> {
        // Safety: Trust me 😉
        // TODO: Actual reasons this is safe (briefly, we trust the function
        // passed to make_future to only use the pointer we gave it when we
        // resume the generator it returned, during that time we have updated it
        // to the context reference we just got, the pointer is a
        // self-reference from the generator back into our state, but we don't
        // create it until we have observed ourselves in a pin so we know we
        // can't have moved between creating the pointer and the generator ever
        // using the pointer so it is safe to dereference).
        self.started().poll(cx)
    }
}

impl<C, G> Stream for FutureImplState<C, G>
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
        // Safety: See `impl Future for FutureImpl`
        self.started().poll_next(cx)
    }
}

pub trait PinnableStream {
    type StreamStorage: UnsafeDefault;
    type StreamItem;
    type Stream: Stream<Item=Self::StreamItem>;

    fn pin(self, f: Pin<&mut Self::StreamStorage>) -> Pin<&mut Self::Stream> where Self: Sized;
}

pub trait PinnableFuture {
    type FutureStorage: UnsafeDefault;
    type FutureOutput;
    type Future: Future<Output=Self::FutureOutput>;

    fn pin(self, f: Pin<&mut Self::FutureStorage>) -> Pin<&mut Self::Future> where Self: Sized;
}

impl<C, G> PinnableFuture for FutureImplState<C, G> where
    C: FnOnce(UnsafeContextRef) -> G,
    G: Generator<Yield = Poll<!>>,
{
    type FutureStorage = FutureImpl<G>;
    type FutureOutput = G::Return;
    type Future = PinnedFutureImpl<G>;

    fn pin(self, f: Pin<&mut Self::FutureStorage>) -> Pin<&mut Self::Future> {
        self.start(f)
    }
}

impl<C, G> PinnableStream for FutureImplState<C, G> where
    C: FnOnce(UnsafeContextRef) -> G,
    G: Generator<Return = ()>,
    G::Yield: IsPoll,
{
    type StreamStorage = FutureImpl<G>;
    type StreamItem = <G::Yield as IsPoll>::Ready;
    type Stream = PinnedFutureImpl<G>;

    fn pin(self, f: Pin<&mut Self::StreamStorage>) -> Pin<&mut Self::Stream> {
        self.start(f)
    }
}

/// `Send`-able wrapper around a `*mut *mut Context`
///
/// This exists to allow the generator inside a `FutureImpl` to be `Send`,
/// provided there are no other `!Send` things in the body of the generator.
pub struct UnsafeContextRef(*mut *mut task::Context<'static>);

impl UnsafeContextRef {
    /// Get a reference to the wrapped context
    ///
    /// This must only be called from the `await!` macro within the
    /// `make_future` function, which will in turn only be run when the
    /// `FutureImpl` has been observed to be in a `Pin`, guaranteeing that the
    /// outer `*const` remains valid.
    // https://github.com/rust-lang/rust-clippy/issues/2906
    pub unsafe fn get_context(&mut self) -> &mut task::Context<'_> {
        constrain_context_lifetime(&mut **self.0)
    }
}

unsafe impl Send for UnsafeContextRef {}

pub unsafe fn make_future<C, G>(c: C) -> impl Future<Output = G::Return> + PinnableFuture<FutureOutput = G::Return>
where
    C: FnOnce(UnsafeContextRef) -> G,
    G: Generator<Yield = Poll<!>>,
{
    FutureImplState::NotStarted(c)
}

pub unsafe fn make_stream<T, C, G>(c: C) -> impl Stream<Item = T> + PinnableStream<StreamItem = T>
where
    C: FnOnce(UnsafeContextRef) -> G,
    G: Generator<Yield = Poll<T>, Return = ()>,
{
    FutureImplState::NotStarted(c)
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
