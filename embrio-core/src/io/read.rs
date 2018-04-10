use core::cmp;
use core::fmt::Debug;
use core::mem::Pin;

use futures::{task, Async, Poll};

pub trait Read {
    type Error: Debug;

    fn poll_read(
        self: Pin<Self>,
        cx: &mut task::Context,
        buf: &mut [u8],
    ) -> Poll<usize, Self::Error>;
}

impl<'a, R> Read for Pin<'a, R>
where
    R: Read + 'a,
{
    type Error = <R as Read>::Error;

    fn poll_read(
        mut self: Pin<Self>,
        cx: &mut task::Context,
        buf: &mut [u8],
    ) -> Poll<usize, Self::Error> {
        // TODO: replace `unsafe { Pin::get_mut(&mut self) }` with `&mut *self` once `Pin: Unpin`
        <R as Read>::poll_read(
            Pin::borrow(unsafe { Pin::get_mut(&mut self) }),
            cx,
            buf,
        )
    }
}

impl<'a> Read for &'a [u8] {
    type Error = !;

    fn poll_read(
        mut self: Pin<Self>,
        _cx: &mut task::Context,
        buf: &mut [u8],
    ) -> Poll<usize, Self::Error> {
        let len = cmp::min(self.len(), buf.len());
        let (head, tail) = self.split_at(len);
        buf[..len].copy_from_slice(head);
        *self = tail;
        Ok(Async::Ready(len))
    }
}
