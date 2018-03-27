use core::mem::Pin;

use futures::prelude::{async_block_pinned};
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

pub fn read_exact<'a, R: Read + 'a>(this: &'a mut Pin<'a, R>, buf: &'a mut [u8]) -> impl StableFuture<Item=(), Error=Error<R::Error>> + 'a {
    async_block_pinned! {
        let mut position = 0;
        while position < buf.len() {
            let amount = await_poll!(|cx| Pin::borrow(this).poll_read(cx, &mut buf[position..]))?;
            position += amount;
            if amount == 0 {
                Err(Error::UnexpectedEof)?;
            }
        }
        Ok(())
    }
}

pub fn write_all<'a, W: Write + 'a>(this: &'a mut Pin<'a, W>, buf: &'a [u8]) -> impl StableFuture<Item=(), Error=Error<W::Error>> + 'a {
    async_block_pinned! {
        let mut position = 0;
        while position < buf.len() {
            let amount = await_poll!(|cx| Pin::borrow(this).poll_write(cx, &buf[position..]))?;
            position += amount;
            if amount == 0 {
                Err(Error::WriteZero)?;
            }
        }
        Ok(())
    }
}

pub fn flush<'a, W: Write + 'a>(this: &'a mut Pin<'a, W>) -> impl StableFuture<Item=(), Error=W::Error> + 'a {
    async_block_pinned! {
        await_poll!(|cx| Pin::borrow(this).poll_flush(cx))?;
        Ok(())
    }
}

pub fn close<'a, W: Write + 'a>(this: &'a mut Pin<'a, W>) -> impl StableFuture<Item=(), Error=W::Error> + 'a {
    async_block_pinned! {
        await_poll!(|cx| Pin::borrow(this).poll_close(cx))?;
        Ok(())
    }
}
