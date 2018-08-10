use core::time::Duration;
use futures_core::{future::Future, stream::Stream};

pub trait Timer: Sized {
    type Error;

    type Timeout: Future<Output = Result<Self, Self::Error>>;

    type Interval: Stream<Item = Result<(), Self::Error>>;

    fn timeout(self, duration: Duration) -> Self::Timeout;

    fn interval(self, duration: Duration) -> Self::Interval;
}

/* TODO: Use this
pub trait Timer {
    type Error;

    fn timeout(
        self: Pin<Self>,
        duration: Time,
    ) -> impl StableFuture<Item = (), Error = Self::Error> + '_;

    fn interval(
        self: Pin<Self>,
        duration: Time,
    ) -> impl StableStream<Item = (), Error = Self::Error> + '_;
}
*/
