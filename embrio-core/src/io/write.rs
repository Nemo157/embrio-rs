use core::{cmp, mem, mem::Pin};

use futures::{task, Async, Poll};

pub trait Write {
    type Error;

    fn poll_write(
        self: Pin<Self>,
        cx: &mut task::Context,
        buf: &[u8],
    ) -> Poll<usize, Self::Error>;

    fn poll_flush(
        self: Pin<Self>,
        cx: &mut task::Context,
    ) -> Poll<(), Self::Error>;

    fn poll_close(
        self: Pin<Self>,
        cx: &mut task::Context,
    ) -> Poll<(), Self::Error>;
}

impl<'a> Write for &'a mut [u8] {
    type Error = !;

    fn poll_write(
        mut self: Pin<Self>,
        _cx: &mut task::Context,
        buf: &[u8],
    ) -> Poll<usize, Self::Error> {
        let len = cmp::min(self.len(), buf.len());
        let (head, tail) = mem::replace(&mut *self, &mut []).split_at_mut(len);
        head.copy_from_slice(&buf[..len]);
        *self = tail;
        Ok(Async::Ready(len))
    }

    fn poll_flush(
        self: Pin<Self>,
        _cx: &mut task::Context,
    ) -> Poll<(), Self::Error> {
        Ok(Async::Ready(()))
    }

    fn poll_close(
        self: Pin<Self>,
        _cx: &mut task::Context,
    ) -> Poll<(), Self::Error> {
        Ok(Async::Ready(()))
    }
}
