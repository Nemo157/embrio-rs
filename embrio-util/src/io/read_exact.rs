use core::mem::PinMut;

use futures_core::future::Future;
use futures_core::task::Poll;
use futures_util::{ready, future::poll_fn};

use embrio_core::io::Read;

use crate::utils::Captures;

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

pub fn read_exact<'a, 'b: 'a, R: Read + 'a>(
    mut this: PinMut<'a, R>,
    buf: &'b mut [u8],
) -> impl Future<Output = Result<(), Error<R::Error>>>
         + Captures<'a>
         + Captures<'b> {
    let mut position = 0;
    poll_fn(move |cx| {
        while position < buf.len() {
            let amount = ready!(this.reborrow().poll_read(cx, &mut buf[position..]))?;
            position += amount;
            if amount == 0 {
                Err(Error::UnexpectedEof)?;
            }
        }
        Poll::Ready(Ok(()))
    })
}
