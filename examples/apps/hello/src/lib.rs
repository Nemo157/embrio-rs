#![no_std]
#![feature(
    arbitrary_self_types,
    async_await,
    const_fn,
    generator_trait,
    generators,
    never_type,
    proc_macro_hygiene
)]

use {
    core::{cell::UnsafeCell, future::Future},
    embrio::io::{self, BufReader, Read, Write},
    embrio_async::{async_block, await},
    pin_utils::pin_mut,
};

#[derive(Debug)]
pub struct Error;

fn run(
    input: impl Read,
    output: impl Write,
) -> impl Future<Output = Result<(), Error>> {
    async_block! {
        pin_mut!(output);
        let input = BufReader::new(input, [0; 32]);
        pin_mut!(input);
        let mut buffer = [0; 64];
        loop {
            await!(io::write_all(output.as_mut(), "Hello, what's your name?\n> ")).map_err(|_| Error)?;
            await!(io::flush(output.as_mut())).map_err(|_| Error)?;
            match await!(io::read_until(input.as_mut(), b'\n', &mut buffer[..])).map_err(|_| Error)? {
                Ok(amount) => {
                    if amount == 0 {
                        await!(io::write_all(output.as_mut(), b"\n")).map_err(|_| Error)?;
                        return Ok(());
                    }
                    await!(io::write_all(output.as_mut(), "Hi ")).map_err(|_| Error)?;
                    await!(io::write_all(output.as_mut(), &buffer[..(amount - 1)])).map_err(|_| Error)?;
                    await!(io::write_all(output.as_mut(), " 👋 \n\n")).map_err(|_| Error)?;
                }
                Err(_) => {
                    await!(io::write_all(output.as_mut(), "\nSorry, that's a bit long for me 😭\n\n")).map_err(|_| Error)?;
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
