use std::await;
use core::mem::PinMut;

use futures::future::{Future, poll_fn};

use embrio_core::io::{Read, Write};

#[derive(Debug)]
pub enum Error<T> {
    UnexpectedEof,
    WriteZero,
    Other(T),
}

impl<T> From<T> for Error<T> {
    fn from(err: T) -> Self {
        Error::Other(err)
    }
}

pub trait Captures<'a> {}

impl<T: ?Sized> Captures<'_> for T {}

pub fn read_exact<'a, 'b: 'a, R: Read + 'a>(
    mut this: PinMut<'a, R>,
    buf: &'b mut [u8],
) -> impl Future<Output = Result<(), Error<R::Error>>>
         + Captures<'a>
         + Captures<'b> {
    async move {
        let mut position = 0;
        while position < buf.len() {
            let amount = await!(poll_fn(|cx| {
                this.reborrow().poll_read(cx, &mut buf[position..])
            }))?;
            position += amount;
            if amount == 0 {
                Err(Error::UnexpectedEof)?;
            }
        }
        Ok(())
    }
}

pub fn write_all<'a, 'b: 'a, W: Write + 'a>(
    mut this: PinMut<'a, W>,
    buf: &'b [u8],
) -> impl Future<Output = Result<(), Error<W::Error>>>
         + Captures<'a>
         + Captures<'b> {
    async move {
        let mut position = 0;
        while position < buf.len() {
            let amount = await!(poll_fn(|cx| {
                this.reborrow().poll_write(cx, &buf[position..])
            }))?;
            position += amount;
            if amount == 0 {
                Err(Error::WriteZero)?;
            }
        }
        Ok(())
    }
}

pub fn flush<W: Write>(
    mut this: PinMut<'a, W>,
) -> impl Future<Output = Result<(), W::Error>> + '_ {
    async move {
        await!(poll_fn(|cx| this.reborrow().poll_flush(cx)))?;
        Ok(())
    }
}

pub fn close<W: Write>(
    mut this: PinMut<W>,
) -> impl Future<Output = Result<(), W::Error>> + '_ {
    async move {
        await!(poll_fn(|cx| this.reborrow().poll_close(cx)))?;
        Ok(())
    }
}
