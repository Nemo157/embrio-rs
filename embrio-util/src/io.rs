use core::mem::Pin;

use futures::future::poll_fn;
use futures::prelude::{async_block_pinned, await};
use futures::stable::StableFuture;

use embrio::io::{Read, Write};

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

impl<'a, T: ?Sized> Captures<'a> for T {}

pub fn read_exact<'a, 'b: 'a, R: Read + 'a>(
    mut this: Pin<'a, R>,
    buf: &'b mut [u8],
) -> impl StableFuture<Item = (), Error = Error<R::Error>>
         + Captures<'a>
         + Captures<'b> {
    async_block_pinned! {
        let mut position = 0;
        while position < buf.len() {
            let amount = await!(poll_fn(|cx| {
                Pin::borrow(&mut this).poll_read(cx, &mut buf[position..])
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
    mut this: Pin<'a, W>,
    buf: &'b [u8],
) -> impl StableFuture<Item = (), Error = Error<W::Error>>
         + Captures<'a>
         + Captures<'b> {
    async_block_pinned! {
        let mut position = 0;
        while position < buf.len() {
            let amount = await!(poll_fn(|cx| {
                Pin::borrow(&mut this).poll_write(cx, &buf[position..])
            }))?;
            position += amount;
            if amount == 0 {
                Err(Error::WriteZero)?;
            }
        }
        Ok(())
    }
}

pub fn flush<'a, W: Write>(
    mut this: Pin<'a, W>,
) -> impl StableFuture<Item = (), Error = W::Error> + 'a {
    async_block_pinned! {
        await!(poll_fn(|cx| Pin::borrow(&mut this).poll_flush(cx)))?;
        Ok(())
    }
}

pub fn close<'a, W: Write>(
    mut this: Pin<'a, W>,
) -> impl StableFuture<Item = (), Error = W::Error> + 'a {
    async_block_pinned! {
        await!(poll_fn(|cx| Pin::borrow(&mut this).poll_close(cx)))?;
        Ok(())
    }
}
