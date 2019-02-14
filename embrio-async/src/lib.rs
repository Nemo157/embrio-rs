#![no_std]
#![feature(
    arbitrary_self_types,
    async_await,
    exhaustive_patterns,
    futures_api,
    generator_trait,
    generators,
    never_type
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
    task::{self, Poll},
};
use futures_core::stream::Stream;
use futures_sink::Sink;

pub use embrio_async_dehygiene::{
    async_block, async_sink_block, async_stream_block, await, await_input,
};

pub use pin_utils::pin_mut;

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

enum FutureImplState<F, G> {
    NotStarted(F),
    Started(G),
    Invalid,
}

struct FutureImpl<F, G> {
    context: *const task::Context<'static>,
    state: FutureImplState<F, G>,
    _pinned: PhantomPinned,
}

unsafe impl<F, G> Send for FutureImpl<F, G>
where
    F: Send,
    G: Send,
{
}

#[allow(clippy::useless_transmute)]
unsafe fn eliminate_context_lifetime(
    cx: &mut task::Context<'_>,
) -> *const task::Context<'static> {
    mem::transmute(cx)
}

impl<F, G> Future for FutureImpl<F, G>
where
    F: FnOnce(UnsafeContextRef) -> G,
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
        let this = unsafe { Pin::get_unchecked_mut(self) };
        if let FutureImplState::Started(g) = &mut this.state {
            // https://github.com/rust-lang/rust-clippy/issues/2906
            unsafe {
                ptr::write(&mut this.context, eliminate_context_lifetime(cx));
                match Pin::new_unchecked(g).resume() {
                    GeneratorState::Yielded(Poll::Pending) => Poll::Pending,
                    GeneratorState::Complete(x) => Poll::Ready(x),
                }
            }
        } else if let FutureImplState::NotStarted(f) =
            mem::replace(&mut this.state, FutureImplState::Invalid)
        {
            this.state = FutureImplState::Started(f(UnsafeContextRef(
                &this.context as *const _,
            )));
            unsafe { Pin::new_unchecked(this) }.poll(cx)
        } else {
            panic!("reached invalid state")
        }
    }
}

impl<F, G> Stream for FutureImpl<F, G>
where
    F: FnOnce(UnsafeContextRef) -> G,
    G: Generator<Return = ()>,
    G::Yield: IsPoll,
{
    type Item = <G::Yield as IsPoll>::Ready;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        // Safety: See `impl Future for FutureImpl`
        let this = unsafe { Pin::get_unchecked_mut(self) };
        if let FutureImplState::Started(g) = &mut this.state {
            // https://github.com/rust-lang/rust-clippy/issues/2906
            unsafe {
                ptr::write(&mut this.context, eliminate_context_lifetime(cx));
                match Pin::new_unchecked(g).resume() {
                    GeneratorState::Yielded(x) => x.into_poll().map(Some),
                    GeneratorState::Complete(()) => Poll::Ready(None),
                }
            }
        } else if let FutureImplState::NotStarted(f) =
            mem::replace(&mut this.state, FutureImplState::Invalid)
        {
            this.state = FutureImplState::Started(f(UnsafeContextRef(
                &this.context as *const _,
            )));
            unsafe { Pin::new_unchecked(this) }.poll_next(cx)
        } else {
            panic!("reached invalid state")
        }
    }
}

enum SinkImplState<F, G> {
    NotStarted(F),
    Started(G),
    Complete,
    Invalid,
}

struct SinkImpl<T, F, G> {
    context: *const task::Context<'static>,
    item: Poll<Option<T>>,
    checked: bool,
    closed: bool,
    state: SinkImplState<F, G>,
    _pinned: PhantomPinned,
}

impl<T, F, G> SinkImpl<T, F, G> {
    unsafe fn context_ref(&self) -> UnsafeContextRef {
        UnsafeContextRef(&self.context as *const _)
    }

    unsafe fn item_ref(&mut self) -> UnsafeItemRef<T> {
        UnsafeItemRef {
            item: &mut self.item as *mut _,
            checked: &mut self.checked as *mut _,
        }
    }
}

/// This `Sink` is complete and no longer accepting items
#[derive(Debug, PartialEq, Eq)]
pub struct Complete;

impl<T, F, G> Sink<T> for SinkImpl<T, F, G>
where
    F: FnOnce(UnsafeContextRef, UnsafeItemRef<T>) -> G,
    G: Generator<Yield = Poll<!>, Return = ()>,
{
    type SinkError = Complete;

    fn poll_ready(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Result<(), Self::SinkError>> {
        // Safety: See `impl Future for FutureImpl`
        let this = unsafe { Pin::get_unchecked_mut(self) };
        if let SinkImplState::Started(g) = &mut this.state {
            unsafe {
                ptr::write(&mut this.context, eliminate_context_lifetime(cx));
                ptr::write(&mut this.item, Poll::Pending);
                ptr::write(&mut this.checked, false);
                match Pin::new_unchecked(g).resume() {
                    GeneratorState::Yielded(Poll::Pending) => {
                        if ptr::read(&this.checked) {
                            Poll::Ready(Ok(()))
                        } else {
                            Poll::Pending
                        }
                    }
                    GeneratorState::Complete(()) => {
                        this.state = SinkImplState::Complete;
                        Poll::Ready(Err(Complete))
                    }
                }
            }
        } else if let SinkImplState::NotStarted(f) =
            mem::replace(&mut this.state, SinkImplState::Invalid)
        {
            unsafe {
                let gen = f(this.context_ref(), this.item_ref());
                this.state = SinkImplState::Started(gen);
                Pin::new_unchecked(this).poll_ready(cx)
            }
        } else if let SinkImplState::Complete = this.state {
            Poll::Ready(Err(Complete))
        } else {
            panic!("reached invalid state")
        }
    }

    fn start_send(
        self: Pin<&mut Self>,
        item: T,
    ) -> Result<(), Self::SinkError> {
        // Safety: See `impl Future for FutureImpl`
        let this = unsafe { Pin::get_unchecked_mut(self) };
        if let SinkImplState::Started(g) = &mut this.state {
            unsafe {
                ptr::write(&mut this.context, ptr::null());
                ptr::write(&mut this.item, Poll::Ready(Some(item)));
                ptr::write(&mut this.checked, false);
                match Pin::new_unchecked(g).resume() {
                    GeneratorState::Yielded(Poll::Pending) => {
                        if ptr::read(&this.checked) {
                            assert!(this.item.is_pending());
                            Ok(())
                        } else {
                            panic!("start_send when sink is not ready")
                        }
                    }
                    GeneratorState::Complete(()) => {
                        this.state = SinkImplState::Complete;
                        Err(Complete)
                    }
                }
            }
        } else if let SinkImplState::NotStarted(f) =
            mem::replace(&mut this.state, SinkImplState::Invalid)
        {
            unsafe {
                let gen = f(this.context_ref(), this.item_ref());
                this.state = SinkImplState::Started(gen);
                Pin::new_unchecked(this).start_send(item)
            }
        } else if let SinkImplState::Complete = this.state {
            Err(Complete)
        } else {
            panic!("reached invalid state")
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Result<(), Self::SinkError>> {
        // Safety: See `impl Future for FutureImpl`
        let this = unsafe { Pin::get_unchecked_mut(self) };
        if let SinkImplState::Started(g) = &mut this.state {
            unsafe {
                ptr::write(&mut this.context, eliminate_context_lifetime(cx));
                ptr::write(&mut this.item, Poll::Pending);
                ptr::write(&mut this.checked, false);
                match Pin::new_unchecked(g).resume() {
                    GeneratorState::Yielded(Poll::Pending) => {
                        if ptr::read(&this.checked) {
                            Poll::Ready(Ok(()))
                        } else {
                            Poll::Pending
                        }
                    }
                    GeneratorState::Complete(()) => {
                        this.state = SinkImplState::Complete;
                        Poll::Ready(Ok(()))
                    }
                }
            }
        } else if let SinkImplState::NotStarted(f) =
            mem::replace(&mut this.state, SinkImplState::Invalid)
        {
            unsafe {
                let gen = f(this.context_ref(), this.item_ref());
                this.state = SinkImplState::Started(gen);
                Pin::new_unchecked(this).poll_flush(cx)
            }
        } else if let SinkImplState::Complete = this.state {
            Poll::Ready(Err(Complete))
        } else {
            panic!("reached invalid state")
        }
    }

    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Result<(), Self::SinkError>> {
        // Safety: See `impl Future for FutureImpl`
        let this = unsafe { Pin::get_unchecked_mut(self) };
        if let SinkImplState::Started(g) = &mut this.state {
            if this.closed {
                unsafe {
                    ptr::write(
                        &mut this.context,
                        eliminate_context_lifetime(cx),
                    );
                    ptr::write(&mut this.item, Poll::Pending);
                    ptr::write(&mut this.checked, false);
                    match Pin::new_unchecked(g).resume() {
                        GeneratorState::Yielded(Poll::Pending) => {
                            if ptr::read(&this.checked) {
                                panic!("Sink read input after closed");
                            }
                            Poll::Pending
                        }
                        GeneratorState::Complete(()) => {
                            this.state = SinkImplState::Complete;
                            Poll::Ready(Ok(()))
                        }
                    }
                }
            } else {
                unsafe {
                    ptr::write(
                        &mut this.context,
                        eliminate_context_lifetime(cx),
                    );
                    ptr::write(&mut this.item, Poll::Ready(None));
                    ptr::write(&mut this.checked, false);
                    match Pin::new_unchecked(g).resume() {
                        GeneratorState::Yielded(Poll::Pending) => {
                            if ptr::read(&this.checked) {
                                this.closed = true;
                                Poll::Ready(Ok(()))
                            } else {
                                Poll::Pending
                            }
                        }
                        GeneratorState::Complete(()) => {
                            this.state = SinkImplState::Complete;
                            Poll::Ready(Ok(()))
                        }
                    }
                }
            }
        } else if let SinkImplState::NotStarted(_) = this.state {
            Poll::Ready(Ok(()))
        } else if let SinkImplState::Complete = this.state {
            Poll::Ready(Ok(()))
        } else {
            panic!("reached invalid state")
        }
    }
}

/// `Send`-able wrapper around a `*const *const Context`
///
/// This exists to allow the generator inside a `FutureImpl` to be `Send`,
/// provided there are no other `!Send` things in the body of the generator.
pub struct UnsafeContextRef(*const *const task::Context<'static>);

impl UnsafeContextRef {
    /// Get a reference to the wrapped context
    ///
    /// This must only be called from the `await!` macro within the
    /// `make_future` function, which will in turn only be run when the
    /// `FutureImpl` has been observed to be in a `Pin`, guaranteeing that the
    /// outer `*const` remains valid.
    // https://github.com/rust-lang/rust-clippy/issues/2906
    #[allow(clippy::transmute_ptr_to_ref)]
    pub unsafe fn get_context(&mut self) -> Option<&mut task::Context<'_>> {
        if self.0.is_null() {
            None
        } else {
            mem::transmute(self.0)
        }
    }
}

unsafe impl Send for UnsafeContextRef {}

pub struct UnsafeItemRef<T> {
    item: *mut Poll<Option<T>>,
    checked: *mut bool,
}

impl<T> UnsafeItemRef<T> {
    pub unsafe fn get_item(&self) -> Poll<Option<T>> {
        ptr::write(self.checked, true);
        ptr::replace(self.item, Poll::Pending)
    }
}

unsafe impl<T> Send for UnsafeItemRef<T> {}

pub unsafe fn make_future<F, G>(f: F) -> impl Future<Output = G::Return>
where
    F: FnOnce(UnsafeContextRef) -> G,
    G: Generator<Yield = Poll<!>>,
{
    FutureImpl {
        context: ptr::null(),
        state: FutureImplState::NotStarted(f),
        _pinned: PhantomPinned,
    }
}

pub unsafe fn make_stream<T, F, G>(f: F) -> impl Stream<Item = T>
where
    F: FnOnce(UnsafeContextRef) -> G,
    G: Generator<Yield = Poll<T>, Return = ()>,
{
    FutureImpl {
        context: ptr::null(),
        state: FutureImplState::NotStarted(f),
        _pinned: PhantomPinned,
    }
}

pub unsafe fn make_sink<T, F, G>(f: F) -> impl Sink<T, SinkError = Complete>
where
    F: FnOnce(UnsafeContextRef, UnsafeItemRef<T>) -> G,
    G: Generator<Yield = Poll<!>, Return = ()>,
{
    SinkImpl {
        context: ptr::null(),
        item: Poll::Pending,
        checked: false,
        closed: false,
        state: SinkImplState::NotStarted(f),
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
