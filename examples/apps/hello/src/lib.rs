#![no_std]
#![feature(generators)]

use {
    core::cell::UnsafeCell,
    embrio::io::{self, BufReader, Read, Write},
    embrio_async::embrio_async,
    pin_utils::pin_mut,
};

#[derive(Debug)]
pub struct Error;

#[embrio_async]
async fn run(input: impl Read, output: impl Write) -> Result<(), Error> {
    pin_mut!(output);
    let input = BufReader::new(input, [0; 32]);
    pin_mut!(input);
    let mut buffer = [0; 64];
    loop {
        io::write_all(output.as_mut(), "Hello, what's your name?\n> ")
            .await
            .map_err(|_| Error)?;
        io::flush(output.as_mut()).await.map_err(|_| Error)?;
        match io::read_until(input.as_mut(), b'\n', &mut buffer[..])
            .await
            .map_err(|_| Error)?
        {
            Ok(amount) => {
                if amount == 0 {
                    io::write_all(output.as_mut(), b"\n")
                        .await
                        .map_err(|_| Error)?;
                    return Ok(());
                }
                io::write_all(output.as_mut(), "Hi ")
                    .await
                    .map_err(|_| Error)?;
                io::write_all(output.as_mut(), &buffer[..(amount - 1)])
                    .await
                    .map_err(|_| Error)?;
                io::write_all(output.as_mut(), " 👋 \n\n")
                    .await
                    .map_err(|_| Error)?;
            }
            Err(_) => {
                io::write_all(
                    output.as_mut(),
                    "\nSorry, that's a bit long for me 😭\n\n",
                )
                .await
                .map_err(|_| Error)?;
            }
        }
    }
}

/// # Safety
///
/// This function can only be called _once_ in the entire lifetime of a process.
pub unsafe fn main(input: impl Read, output: impl Write) -> Result<(), Error> {
    struct RacyCell<T>(UnsafeCell<T>);
    impl<T> RacyCell<T> {
        const fn new(value: T) -> Self {
            RacyCell(UnsafeCell::new(value))
        }
        #[allow(clippy::mut_from_ref)]
        unsafe fn get_mut_unchecked(&self) -> &mut T {
            &mut *self.0.get()
        }
    }
    unsafe impl<T> Sync for RacyCell<T> {}
    static EXECUTOR: RacyCell<embrio::Executor> =
        RacyCell::new(embrio::Executor::new());
    EXECUTOR.get_mut_unchecked().block_on(run(input, output))
}
