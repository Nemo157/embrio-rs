use core::{
    cmp,
    fmt::Debug,
    pin::PinMut,
    task::{self, Poll},
};

pub trait Read {
    type Error: Debug;

    fn poll_read(
        self: PinMut<'_, Self>,
        cx: &mut task::Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>>;
}

impl<R> Read for PinMut<'_, R>
where
    R: Read,
{
    type Error = <R as Read>::Error;

    fn poll_read(
        mut self: PinMut<'_, Self>,
        cx: &mut task::Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        <R as Read>::poll_read(PinMut::reborrow(&mut *self), cx, buf)
    }
}

impl Read for &[u8] {
    type Error = !;

    fn poll_read(
        mut self: PinMut<'_, Self>,
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
