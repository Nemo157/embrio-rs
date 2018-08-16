#![feature(async_await, await_macro, futures_api, never_type, pin)]

use embrio::io::{self, Read, Write, BufReader};
use pin_utils::pin_mut;

pub struct Error;

async fn run(input: impl Read, output: impl Write) -> Result<!, Error> {
    pin_mut!(output);
    let input = BufReader::new(input, [0; 32]);
    pin_mut!(input);
    let mut buffer = [0; 64];
    loop {
        await!(io::write_all(output.reborrow(), "Hello 👋\n > ".as_bytes())).map_err(|_| Error)?;
        match await!(io::read_until(input.reborrow(), b'\n', &mut buffer)).map_err(|_| Error)? {
            Ok(amount) => {
                await!(io::write_all(output.reborrow(), &buffer[..amount])).map_err(|_| Error)?;
            }
            Err(_) => {
                await!(io::write_all(output.reborrow(), "\nSorry, that's a bit long for me 😭\n".as_bytes())).map_err(|_| Error)?;
            }
        }
    }
}

pub fn main(input: impl Read, output: impl Write) -> Result<!, Error> {
    embrio::executor::block_on(run(input, output))
}
