use core::{
    cmp,
    fmt::Debug,
    mem,
    pin::Pin,
    task::{Poll, Waker},
};

pub trait Write {
    type Error: Debug;

    fn poll_write(
        self: Pin<&mut Self>,
        waker: &Waker,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>>;

    fn poll_flush(
        self: Pin<&mut Self>,
        waker: &Waker,
    ) -> Poll<Result<(), Self::Error>>;

    fn poll_close(
        self: Pin<&mut Self>,
        waker: &Waker,
    ) -> Poll<Result<(), Self::Error>>;
}

impl<W> Write for Pin<&mut W>
where
    W: Write,
{
    type Error = <W as Write>::Error;

    fn poll_write(
        self: Pin<&mut Self>,
        waker: &Waker,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>> {
        <W as Write>::poll_write(Pin::get_mut(self).as_mut(), waker, buf)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        waker: &Waker,
    ) -> Poll<Result<(), Self::Error>> {
        <W as Write>::poll_flush(Pin::get_mut(self).as_mut(), waker)
    }

    fn poll_close(
        self: Pin<&mut Self>,
        waker: &Waker,
    ) -> Poll<Result<(), Self::Error>> {
        <W as Write>::poll_close(Pin::get_mut(self).as_mut(), waker)
    }
}

impl Write for &mut [u8] {
    type Error = !;

    fn poll_write(
        mut self: Pin<&mut Self>,
        _waker: &Waker,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>> {
        let len = cmp::min(self.len(), buf.len());
        let (head, tail) = mem::replace(&mut *self, &mut []).split_at_mut(len);
        head.copy_from_slice(&buf[..len]);
        *self = tail;
        Poll::Ready(Ok(len))
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        _waker: &Waker,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: Pin<&mut Self>,
        _waker: &Waker,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}
