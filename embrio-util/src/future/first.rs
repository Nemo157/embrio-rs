use core::mem::Pin;
use futures::{task, Poll, stable::StableFuture};

use super::StableInfiniteStream;

pub fn first<Inner>(
    inner: Inner,
) -> impl StableFuture<Item = Inner::Item, Error = Inner::Error>
where
    Inner: StableInfiniteStream,
{
    First { inner }
}

struct First<Inner>
where
    Inner: StableInfiniteStream,
{
    inner: Inner,
}

impl<Inner> StableFuture for First<Inner>
where
    Inner: StableInfiniteStream,
{
    type Item = Inner::Item;
    type Error = Inner::Error;

    fn poll(
        mut self: Pin<Self>,
        cx: &mut task::Context,
    ) -> Poll<Self::Item, Self::Error> {
        StableInfiniteStream::poll_next(pin_field!(self, inner), cx)
    }
}
