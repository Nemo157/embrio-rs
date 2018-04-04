use core::mem::Pin;

use futures::stable::StableStream;
use futures::{task, Poll};

pub trait StableInfiniteStream: StableStream {
    fn poll_next(
        self: Pin<Self>,
        cx: &mut task::Context,
    ) -> Poll<Self::Item, Self::Error> {
        <Self as StableStream>::poll_next(self, cx)
            .map(|async| async.map(|item| item.expect("infinite stream")))
    }
}
