use core::{
    cmp,
    fmt::Debug,
    pin::Pin,
    task::{self, Poll},
};

pub trait Read {
    type Error: Debug;

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>>;
}

impl<R> Read for Pin<&mut R>
where
    R: Read,
{
    type Error = <R as Read>::Error;

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        <R as Read>::poll_read(Pin::get_mut(self).as_mut(), cx, buf)
    }
}

impl Read for &[u8] {
    type Error = !;

    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        let len = cmp::min(self.len(), buf.len());
        let (head, tail) = self.split_at(len);
        buf[..len].copy_from_slice(head);
        *self = tail;
        Poll::Ready(Ok(len))
    }
}
