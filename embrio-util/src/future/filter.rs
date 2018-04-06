use core::marker::Unpin;
use core::mem::Pin;
use futures::{task, Async, Poll, stable::StableStream};

use super::StableInfiniteStream;

pub fn filter<Inner, Callback>(
    inner: Inner,
    callback: Callback,
) -> Filter<Inner, Callback>
where
    Inner: StableStream,
    Callback: for<'a> FnMut(&'a Inner::Item) -> bool + Unpin,
{
    Filter {
        inner,
        callback,
    }
}

pub struct Filter<Inner, Callback>
where
    Inner: StableStream,
    Callback: for<'a> FnMut(&'a Inner::Item) -> bool + Unpin,
{
    inner: Inner,
    callback: Callback,
}

impl<Inner, Callback> StableStream for Filter<Inner, Callback>
where
    Inner: StableStream,
    Callback: for<'a> FnMut(&'a Inner::Item) -> bool + Unpin,
{
    type Item = Inner::Item;
    type Error = Inner::Error;

    fn poll_next(
        mut self: Pin<Self>,
        cx: &mut task::Context,
    ) -> Poll<Option<Self::Item>, Self::Error> {
        Ok(loop {
            match pin_field!(self, inner).poll_next(cx)? {
                Async::Ready(Some(item)) => {
                    if unpin_field!(self, callback)(&item) {
                        break Async::Ready(Some(item));
                    } else {
                        continue;
                    }
                }
                Async::Ready(None) => {
                    break Async::Ready(None);
                }
                Async::Pending => {
                    break Async::Pending;
                }
            }
        })
    }
}

impl<Inner, Callback> StableInfiniteStream for Filter<Inner, Callback>
where
    Inner: StableInfiniteStream,
    Callback: for<'a> FnMut(&'a Inner::Item) -> bool + Unpin,
{
}
