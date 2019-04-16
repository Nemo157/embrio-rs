use core::pin::Pin;

use futures_core::{future::Future, task::Poll};
use futures_util::{future::poll_fn, ready};

use embrio_core::io::Read;

#[derive(Debug)]
pub enum Error<T> {
    UnexpectedEof,
    Other(T),
}

impl<T> From<T> for Error<T> {
    fn from(err: T) -> Self {
        Error::Other(err)
    }
}

pub fn read_exact<'a, R: Read + 'a>(
    mut this: Pin<&'a mut R>,
    mut buf: impl AsMut<[u8]> + 'a,
) -> impl Future<Output = Result<(), Error<R::Error>>> + 'a {
    let mut position = 0;
    poll_fn(move |cx| {
        let buf = buf.as_mut();
        while position < buf.len() {
            let amount =
                ready!(this.as_mut().poll_read(cx, &mut buf[position..]))?;
            position += amount;
            if amount == 0 {
                Err(Error::UnexpectedEof)?;
            }
        }
        Poll::Ready(Ok(()))
    })
}
