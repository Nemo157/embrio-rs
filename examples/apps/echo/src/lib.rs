#![feature(async_await, await_macro, futures_api, never_type, pin)]

use embrio::io::{self, Read, Write, BufReader};
use pin_utils::pin_mut;

pub struct Error;

pub async fn main(input: impl Read<Error = !>, output: impl Write<Error = !>) -> Result<!, Error> {
    pin_mut!(output);
    let input = BufReader::new(input, [0; 32]);
    pin_mut!(input);
    let mut buffer = [0; 64];
    loop {
        await!(io::write_all(output.reborrow(), "Hello 👋\n > ".as_bytes()))?;
        match await!(io::read_until(input.reborrow(), b'\n', &mut buffer))? {
            Ok(amount) => {
                await!(io::write_all(output.reborrow(), &buffer[..amount]))?;
            }
            Err(_) => {
                await!(io::write_all(output.reborrow(), "\nSorry, that's a bit long for me 😭\n".as_bytes()))?;
            }
        }
    }
}

impl From<!> for Error {
    fn from(_: !) -> Error {
        Error
    }
}

impl<E> From<embrio::io::write_all::Error<E>> for Error {
    fn from(_: embrio::io::write_all::Error<E>) -> Error {
        Error
    }
}
