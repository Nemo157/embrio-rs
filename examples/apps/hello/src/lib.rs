#![feature(async_await, await_macro, futures_api, never_type, pin)]

use embrio::io::{self, Read, Write, BufReader};
use pin_utils::pin_mut;

#[derive(Debug)]
pub struct Error;

async fn run(input: impl Read, output: impl Write) -> Result<(), Error> {
    pin_mut!(output);
    let input = BufReader::new(input, [0; 32]);
    pin_mut!(input);
    let mut buffer = [0; 64];
    loop {
        await!(io::write_all(output.reborrow(), "Hello, what's your name?\n> ")).map_err(|_| Error)?;
        await!(io::flush(output.reborrow())).map_err(|_| Error)?;
        match await!(io::read_until(input.reborrow(), b'\n', &mut buffer)).map_err(|_| Error)? {
            Ok(amount) => {
                if amount == 0 {
                    await!(io::write_all(output.reborrow(), b"\n")).map_err(|_| Error)?;
                    return Ok(());
                }
                await!(io::write_all(output.reborrow(), "Hi ")).map_err(|_| Error)?;
                await!(io::write_all(output.reborrow(), &buffer[..(amount - 1)])).map_err(|_| Error)?;
                await!(io::write_all(output.reborrow(), " 👋 \n\n")).map_err(|_| Error)?;
            }
            Err(_) => {
                await!(io::write_all(output.reborrow(), "\nSorry, that's a bit long for me 😭\n\n")).map_err(|_| Error)?;
            }
        }
    }
}

pub fn main(input: impl Read, output: impl Write) -> Result<(), Error> {
    embrio::executor::block_on(run(input, output))
}
