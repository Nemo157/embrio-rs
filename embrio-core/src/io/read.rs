use core::cmp;
use core::fmt::Debug;
use core::mem::PinMut;

use futures_core::{task, Poll};

pub trait Read {
    type Error: Debug;

    fn poll_read(
        self: PinMut<Self>,
        cx: &mut task::Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>>;
}

impl<'a, R> Read for PinMut<'a, R>
where
    R: Read + 'a,
{
    type Error = <R as Read>::Error;

    fn poll_read(
        mut self: PinMut<Self>,
        cx: &mut task::Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        <R as Read>::poll_read(PinMut::reborrow(&mut *self), cx, buf)
    }
}

impl<'a> Read for &'a [u8] {
    type Error = !;

    fn poll_read(
        mut self: PinMut<Self>,
        _cx: &mut task::Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        let len = cmp::min(self.len(), buf.len());
        let (head, tail) = self.split_at(len);
        buf[..len].copy_from_slice(head);
        *self = tail;
        Poll::Ready(Ok(len))
    }
}
