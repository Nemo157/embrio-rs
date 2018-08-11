use core::mem::PinMut;

use futures_core::future::Future;
use futures_core::task::Poll;
use futures_util::{ready, future::poll_fn};

use embrio_core::io::{Read, Write};

pub mod read_exact {
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
}

pub mod write_all {
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
}

pub trait Captures<'a> {}

impl<T: ?Sized> Captures<'_> for T {}

pub fn read_exact<'a, 'b: 'a, R: Read + 'a>(
    mut this: PinMut<'a, R>,
    buf: &'b mut [u8],
) -> impl Future<Output = Result<(), self::read_exact::Error<R::Error>>>
         + Captures<'a>
         + Captures<'b> {
    let mut position = 0;
    poll_fn(move |cx| {
        while position < buf.len() {
            let amount = ready!(this.reborrow().poll_read(cx, &mut buf[position..]))?;
            position += amount;
            if amount == 0 {
                Err(self::read_exact::Error::UnexpectedEof)?;
            }
        }
        Poll::Ready(Ok(()))
    })
}

pub fn write_all<'a, 'b: 'a, W: Write + 'a>(
    mut this: PinMut<'a, W>,
    buf: &'b [u8],
) -> impl Future<Output = Result<(), self::write_all::Error<W::Error>>>
         + Captures<'a>
         + Captures<'b> {
    let mut position = 0;
    poll_fn(move |cx| {
        while position < buf.len() {
            let amount = ready!(this.reborrow().poll_write(cx, &buf[position..]))?;
            position += amount;
            if amount == 0 {
                Err(self::write_all::Error::WriteZero)?;
            }
        }
        Poll::Ready(Ok(()))
    })
}

pub fn flush<W: Write>(
    mut this: PinMut<'a, W>,
) -> impl Future<Output = Result<(), W::Error>> + '_ {
    poll_fn(move |cx| this.reborrow().poll_flush(cx))
}

pub fn close<W: Write>(
    mut this: PinMut<W>,
) -> impl Future<Output = Result<(), W::Error>> + '_ {
    poll_fn(move |cx| this.reborrow().poll_close(cx))
}
