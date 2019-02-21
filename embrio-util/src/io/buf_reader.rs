use core::{
    pin::Pin,
    task::{Poll, Waker},
};

use embrio_core::io::{BufRead, Read};
use futures_util::ready;

pub struct BufReader<R, B> {
    reader: R,
    buffer: B,
    left: usize,
    right: usize,
}

impl<R, B> BufReader<R, B> {
    pub fn new(reader: R, buffer: B) -> Self {
        BufReader {
            reader,
            buffer,
            left: 0,
            right: 0,
        }
    }
}

impl<R: Read, B: AsMut<[u8]>> Read for BufReader<R, B> {
    type Error = R::Error;

    fn poll_read(
        self: Pin<&mut Self>,
        waker: &Waker,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        let available = ready!(self.poll_fill_buf(waker))?;
        buf.copy_from_slice(available);
        Poll::Ready(Ok(available.len()))
    }
}

impl<R: Read, B: AsMut<[u8]>> BufRead for BufReader<R, B> {
    fn poll_fill_buf<'a>(
        self: Pin<&'a mut Self>,
        waker: &Waker,
    ) -> Poll<Result<&'a [u8], Self::Error>> {
        // Safety: we re-wrap the only !Unpin field in a new PinMut
        let BufReader {
            ref mut reader,
            ref mut buffer,
            ref left,
            ref mut right,
        } = unsafe { Pin::get_unchecked_mut(self) };
        let reader = unsafe { Pin::new_unchecked(reader) };
        let buffer = buffer.as_mut();
        if let Poll::Ready(amount) =
            reader.poll_read(waker, &mut buffer[*right..])?
        {
            *right += amount;
            return Poll::Ready(Ok(&buffer[*left..*right]));
        }
        if *left == *right {
            Poll::Pending
        } else {
            Poll::Ready(Ok(&buffer[*left..*right]))
        }
    }

    fn consume(self: Pin<&mut Self>, amount: usize) {
        // Safety: we only access unpin fields
        let BufReader {
            ref mut left,
            ref mut right,
            ..
        } = unsafe { Pin::get_unchecked_mut(self) };
        assert!(amount <= *right - *left);
        *left += amount;
        if *left == *right {
            *left = 0;
            *right = 0;
        }
    }
}
