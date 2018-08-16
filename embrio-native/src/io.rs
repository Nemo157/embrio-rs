use std::io as stdio;
use std::marker::Unpin;
use std::mem::PinMut;
use embrio_core::io as embrio;
use futures_core::{task, Poll};

pub(crate) struct Std<T>(pub(crate) T);

impl<T: stdio::Read + Unpin> embrio::Read for Std<T> {
    type Error = stdio::Error;

    fn poll_read(
        self: PinMut<Self>,
        _cx: &mut task::Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>>
    {
        Poll::Ready(PinMut::get_mut(self).0.read(buf))
    }
}

impl<T: stdio::Write + Unpin> embrio::Write for Std<T> {
    type Error = stdio::Error;

    fn poll_write(
        self: PinMut<Self>,
        _cx: &mut task::Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>>
    {
        Poll::Ready(PinMut::get_mut(self).0.write(buf))
    }

    fn poll_flush(
        self: PinMut<Self>,
        _cx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>>
    {
        Poll::Ready(PinMut::get_mut(self).0.flush())
    }

    fn poll_close(
        self: PinMut<Self>,
        cx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>>
    {
        self.poll_flush(cx)
    }
}
