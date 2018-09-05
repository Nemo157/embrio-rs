use core::pin::PinMut;

use futures_core::future::Future;
use futures_core::task::Poll;
use futures_util::{ready, future::poll_fn};

use embrio_core::io::Write;

#[derive(Debug)]
pub enum Error<T> {
    WriteZero,
    Other(T),
}

impl<T> From<T> for Error<T> {
    fn from(err: T) -> Self {
        Error::Other(err)
    }
}

pub fn write_all<'a, W: Write + 'a>(
    mut this: PinMut<'a, W>,
    buf: impl AsRef<[u8]> + 'a,
) -> impl Future<Output = Result<(), Error<W::Error>>> + 'a {
    let mut position = 0;
    poll_fn(move |cx| {
        let buf = buf.as_ref();
        while position < buf.len() {
            let amount = ready!(this.reborrow().poll_write(cx, &buf[position..]))?;
            position += amount;
            if amount == 0 {
                Err(Error::WriteZero)?;
            }
        }
        Poll::Ready(Ok(()))
    })
}
