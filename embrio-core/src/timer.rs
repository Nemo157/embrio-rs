use futures::stable::{StableFuture, StableStream};

use si::Time;

pub trait Timer: Sized {
    type Error;

    type Timeout: StableFuture<Item = Self, Error = Self::Error>;

    type Interval: StableStream<Item = (), Error = Self::Error>;

    fn timeout(self, duration: Time) -> Self::Timeout;

    fn interval(self, duration: Time) -> Self::Interval;
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
