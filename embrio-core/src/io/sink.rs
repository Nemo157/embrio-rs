use core::mem::Pin;

use futures::{task, Async, Poll};

use io::Write;

pub struct Sink {
    _marker: (),
}

pub fn sink() -> Sink {
    Sink { _marker: () }
}

impl Write for Sink {
    type Error = !;

    fn poll_write(
        self: Pin<Self>,
        _cx: &mut task::Context,
        buf: &[u8],
    ) -> Poll<usize, Self::Error> {
        Ok(Async::Ready(buf.len()))
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
