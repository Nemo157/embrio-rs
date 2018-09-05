use core::pin::PinMut;
use crate::io::Write;
use futures_core::{task, Poll};

pub struct Void {
    _marker: (),
}

pub fn void() -> Void {
    Void { _marker: () }
}

impl Write for Void {
    type Error = !;

    fn poll_write(
        self: PinMut<'_, Self>,
        _cx: &mut task::Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>> {
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(
        self: PinMut<'_, Self>,
        _cx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: PinMut<'_, Self>,
        _cx: &mut task::Context,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}
