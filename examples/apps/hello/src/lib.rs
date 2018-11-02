#![no_std]
#![feature(
    arbitrary_self_types,
    const_fn,
    futures_api,
    generator_trait,
    generators,
    never_type,
    pin
)]

use core::{
    cell::UnsafeCell,
    future::Future,
    ops::{Generator, GeneratorState},
    pin::Pin,
    ptr::NonNull,
    task::{self, Poll},
};

use embrio::io::{self, BufReader, Read, Write};
use pin_utils::pin_mut;

#[derive(Debug)]
pub struct Error;

static mut LOCAL_WAKER: Option<NonNull<task::LocalWaker>> = None;

macro_rules! aweight {
    ($e:expr) => {{
        let mut pinned = $e;
        loop {
            // Safety: Not much, probably safe because of the restriction on
            // `main`, but I can't be bothered guaranteeing that. Should work in
            // the current examples.
            let polled = unsafe {
                let mut lw = LOCAL_WAKER.take().unwrap();
                let pin = Pin::new_unchecked(&mut pinned);
                let polled = Future::poll(pin, lw.as_mut());
                assert!(LOCAL_WAKER.replace(lw).is_none());
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
                self: Pin<&mut Self>,
                lw: &task::LocalWaker
            ) -> Poll<Self::Output>
            {
                // Safety: Not much, probably safe because of the restriction on
                // `main`, but I can't be bothered guaranteeing that. Should
                // work in the current examples.
                #[allow(clippy::cast_ptr_alignment)]
                unsafe {
                    assert!(LOCAL_WAKER.replace(NonNull::new_unchecked(lw as *const _ as *mut task::LocalWaker)).is_none());
                    let poll = match Pin::get_mut_unchecked(self).0.resume() {
                        GeneratorState::Yielded(()) => Poll::Pending,
                        GeneratorState::Complete(x) => Poll::Ready(x),
                    };
                    assert!(LOCAL_WAKER.take().is_some());
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
            aweight!(io::write_all(output.as_mut(), "Hello, what's your name?\n> ")).map_err(|_| Error)?;
            aweight!(io::flush(output.as_mut())).map_err(|_| Error)?;
            match aweight!(io::read_until(input.as_mut(), b'\n', &mut buffer[..])).map_err(|_| Error)? {
                Ok(amount) => {
                    if amount == 0 {
                        aweight!(io::write_all(output.as_mut(), b"\n")).map_err(|_| Error)?;
                        return Ok(());
                    }
                    aweight!(io::write_all(output.as_mut(), "Hi ")).map_err(|_| Error)?;
                    aweight!(io::write_all(output.as_mut(), &buffer[..(amount - 1)])).map_err(|_| Error)?;
                    aweight!(io::write_all(output.as_mut(), " 👋 \n\n")).map_err(|_| Error)?;
                }
                Err(_) => {
                    aweight!(io::write_all(output.as_mut(), "\nSorry, that's a bit long for me 😭\n\n")).map_err(|_| Error)?;
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
        #[allow(clippy::mut_from_ref)]
        unsafe fn get_mut_unchecked(&self) -> &mut T {
            &mut *self.0.get()
        }
    }
    unsafe impl<T> Sync for Unsync<T> {}
    static EXECUTOR: Unsync<embrio::Executor> =
        Unsync::new(embrio::Executor::new());
    EXECUTOR.get_mut_unchecked().block_on(run(input, output))
}
