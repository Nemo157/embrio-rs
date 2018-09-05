#![no_std]
#![feature(
    arbitrary_self_types,
    const_fn,
    futures_api,
    generator_trait,
    generators,
    never_type,
    option_replace,
    pin,
)]

use core::{
    cell::UnsafeCell,
    future::Future,
    ops::{Generator, GeneratorState},
    pin::PinMut,
    ptr::NonNull,
    task::{self, Poll},
};

use embrio::io::{self, BufReader, Read, Write};
use pin_utils::pin_mut;

#[derive(Debug)]
pub struct Error;

static mut TASK_CONTEXT: Option<NonNull<task::Context>> = None;

macro_rules! aweight {
    ($e:expr) => {{
        let mut pinned = $e;
        loop {
            // Safety: Not much, probably safe because of the restriction on
            // `main`, but I can't be bothered guaranteeing that. Should work in
            // the current examples.
            let polled = unsafe {
                let mut cx = TASK_CONTEXT.take().unwrap();
                let pin = PinMut::new_unchecked(&mut pinned);
                let polled = Future::poll(pin, cx.as_mut());
                assert!(TASK_CONTEXT.replace(cx).is_none());
                polled
            };
            if let Poll::Ready(x) = polled {
                break x;
            }
            yield
        }
    }};
}

macro_rules! asink {
    ($($s:stmt);*) => {{
        struct FutureImpl<G>(G);

        impl<G: Generator<Yield = ()>> Future for FutureImpl<G> {
            type Output = G::Return;
            fn poll(
                self: PinMut<Self>,
                cx: &mut task::Context
            ) -> Poll<Self::Output>
            {
                // Safety: Not much, probably safe because of the restriction on
                // `main`, but I can't be bothered guaranteeing that. Should
                // work in the current examples.
                unsafe {
                    assert!(TASK_CONTEXT.replace(NonNull::new_unchecked(cx as *mut task::Context as *mut () as *mut task::Context<'static>)).is_none());
                    let poll = match PinMut::get_mut_unchecked(self).0.resume() {
                        GeneratorState::Yielded(()) => Poll::Pending,
                        GeneratorState::Complete(x) => Poll::Ready(x),
                    };
                    assert!(TASK_CONTEXT.take().is_some());
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

fn run(
    input: impl Read,
    output: impl Write,
) -> impl Future<Output = Result<(), Error>> {
    asink! {
        pin_mut!(output);
        let input = BufReader::new(input, [0; 32]);
        pin_mut!(input);
        let mut buffer = [0; 64];
        loop {
            aweight!(io::write_all(output.reborrow(), "Hello, what's your name?\n> ")).map_err(|_| Error)?;
            aweight!(io::flush(output.reborrow())).map_err(|_| Error)?;
            match aweight!(io::read_until(input.reborrow(), b'\n', &mut buffer[..])).map_err(|_| Error)? {
                Ok(amount) => {
                    if amount == 0 {
                        aweight!(io::write_all(output.reborrow(), b"\n")).map_err(|_| Error)?;
                        return Ok(());
                    }
                    aweight!(io::write_all(output.reborrow(), "Hi ")).map_err(|_| Error)?;
                    aweight!(io::write_all(output.reborrow(), &buffer[..(amount - 1)])).map_err(|_| Error)?;
                    aweight!(io::write_all(output.reborrow(), " 👋 \n\n")).map_err(|_| Error)?;
                }
                Err(_) => {
                    aweight!(io::write_all(output.reborrow(), "\nSorry, that's a bit long for me 😭\n\n")).map_err(|_| Error)?;
                }
            }
        }
    }
}

/// # Safety
///
/// This function can only be called _once_ in the entire lifetime of a process.
pub unsafe fn main(input: impl Read, output: impl Write) -> Result<(), Error> {
    struct Unsync<T>(UnsafeCell<T>);
    impl<T> Unsync<T> {
        const fn new(value: T) -> Self {
            Unsync(UnsafeCell::new(value))
        }
        unsafe fn get_mut_unchecked(&self) -> &mut T {
            &mut *self.0.get()
        }
    }
    unsafe impl<T> Sync for Unsync<T> {}
    static EXECUTOR: Unsync<embrio::Executor> =
        Unsync::new(embrio::Executor::new());
    EXECUTOR.get_mut_unchecked().block_on(run(input, output))
}
