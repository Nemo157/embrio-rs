use core::{
    cmp,
    fmt::Debug,
    pin::Pin,
    task::{Poll, Waker},
};

pub trait Read {
    type Error: Debug;

    fn poll_read(
        self: Pin<&mut Self>,
        waker: &Waker,
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
        waker: &Waker,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        <R as Read>::poll_read(Pin::get_mut(self).as_mut(), waker, buf)
    }
}

impl Read for &[u8] {
    type Error = !;

    fn poll_read(
        mut self: Pin<&mut Self>,
        _waker: &Waker,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        let len = cmp::min(self.len(), buf.len());
        let (head, tail) = self.split_at(len);
        buf[..len].copy_from_slice(head);
        *self = tail;
        Poll::Ready(Ok(len))
    }
}
