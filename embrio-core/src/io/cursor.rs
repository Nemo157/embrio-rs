use core::cmp;
use core::fmt;
use core::marker::Unpin;
use core::mem::Pin;

use futures::{task, Async, Poll};

use io::Write;

pub struct Cursor<T: AsMut<[u8]>> {
    inner: T,
    position: usize,
}

impl<T: AsMut<[u8]>> Cursor<T> {
    pub fn new(inner: T) -> Cursor<T> {
        Cursor {
            inner,
            position: 0,
        }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn position(&self) -> usize {
        self.position
    }
}

impl<T: AsMut<[u8]>> Write for Cursor<T> where Self: Unpin {
    type Error = !;

    fn poll_write(mut self: Pin<Self>, _cx: &mut task::Context, buf: &[u8]) -> Poll<usize, Self::Error> {
        let len = {
            let position = self.position;
            let inner = &mut self.inner.as_mut()[position..];
            let len = cmp::min(inner.len(), buf.len());
            inner[..len].copy_from_slice(&buf[..len]);
            len
        };
        self.position += len;
        Ok(Async::Ready(len))
    }

    fn poll_flush(self: Pin<Self>, _cx: &mut task::Context) -> Poll<(), Self::Error> {
        Ok(Async::Ready(()))
    }

    fn poll_close(self: Pin<Self>, _cx: &mut task::Context) -> Poll<(), Self::Error> {
        Ok(Async::Ready(()))
    }
}

impl<T: AsMut<[u8]>> fmt::Write for Cursor<T> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let inner = &mut self.inner.as_mut()[self.position..];
        let len = cmp::min(inner.len(), s.len());
        if len != s.len() {
            panic!("Overflow writing fmt string");
        }
        inner[..len].copy_from_slice(s.as_bytes());
        self.position += len;
        Ok(())
    }
}
