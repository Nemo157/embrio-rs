use core::pin::PinMut;

use futures_core::task::{self, Poll};
use futures_util::ready;

use embrio_core::io::{Read, BufRead};

pub struct BufReader<R, B> {
    reader: R,
    buffer: B,
    left: usize,
    right: usize,
}

impl<R, B> BufReader<R, B> {
    pub fn new(reader: R, buffer: B) -> Self {
        BufReader { reader, buffer, left: 0, right: 0 }
    }
}

impl<R: Read, B: AsMut<[u8]>> Read for BufReader<R, B> {
    type Error = R::Error;

    fn poll_read(
        self: PinMut<'_, Self>,
        cx: &mut task::Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>>
    {
        let available = ready!(self.poll_fill_buf(cx))?;
        buf.copy_from_slice(available);
        Poll::Ready(Ok(available.len()))
    }
}

impl<R: Read, B: AsMut<[u8]>> BufRead for BufReader<R, B> {
    fn poll_fill_buf<'a>(
        self: PinMut<'a, Self>,
        cx: &mut task::Context,
    ) -> Poll<Result<&'a [u8], Self::Error>>
    {
        // Safety: we re-wrap the only !Unpin field in a new PinMut
        let BufReader { ref mut reader, ref mut buffer, ref left, ref mut right } = unsafe { PinMut::get_mut_unchecked(self) };
        let mut reader = unsafe { PinMut::new_unchecked(reader) };
        let buffer = buffer.as_mut();
        loop {
            if let Poll::Ready(amount) = reader.reborrow().poll_read(cx, &mut buffer[*right..])? {
                *right += amount;
            } else {
                break;
            }
            return Poll::Ready(Ok(&buffer[*left..*right]));
        }
        if *left == *right {
            Poll::Pending
        } else {
            Poll::Ready(Ok(&buffer[*left..*right]))
        }
    }

    fn consume(self: PinMut<'_, Self>, amount: usize) {
        // Safety: we only access unpin fields
        let BufReader { ref mut left, ref mut right, .. } = unsafe { PinMut::get_mut_unchecked(self) };
        assert!(amount <= *right - *left);
        *left += amount;
        if *left == *right {
            *left = 0;
            *right = 0;
        }
    }
}
