use core::fmt::Debug;
use core::{cmp, mem, mem::PinMut};

use futures_core::{task, Poll};

pub trait Write {
    type Error: Debug;

    fn poll_write(
        self: PinMut<Self>,
        cx: &mut task::Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>>;

    fn poll_flush(
        self: PinMut<Self>,
        cx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>>;

    fn poll_close(
        self: PinMut<Self>,
        cx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>>;
}

impl<'a, W> Write for PinMut<'a, W>
where
    W: Write + 'a,
{
    type Error = <W as Write>::Error;

    fn poll_write(
        mut self: PinMut<Self>,
        cx: &mut task::Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>> {
        <W as Write>::poll_write(PinMut::reborrow(&mut *self), cx, buf)
    }

    fn poll_flush(
        mut self: PinMut<Self>,
        cx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>> {
        <W as Write>::poll_flush(PinMut::reborrow(&mut *self), cx)
    }

    fn poll_close(
        mut self: PinMut<Self>,
        cx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>> {
        <W as Write>::poll_close(PinMut::reborrow(&mut *self), cx)
    }
}

impl<'a> Write for &'a mut [u8] {
    type Error = !;

    fn poll_write(
        mut self: PinMut<Self>,
        _cx: &mut task::Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>> {
        let len = cmp::min(self.len(), buf.len());
        let (head, tail) = mem::replace(&mut *self, &mut []).split_at_mut(len);
        head.copy_from_slice(&buf[..len]);
        *self = tail;
        Poll::Ready(Ok(len))
    }

    fn poll_flush(
        self: PinMut<Self>,
        _cx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: PinMut<Self>,
        _cx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}
