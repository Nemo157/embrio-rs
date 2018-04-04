use core::mem::Pin;
use futures::{task, Async, Poll, future::Either, stable::StableFuture};

pub fn select<Left, Right>(
    left: Left,
    right: Right,
) -> impl StableFuture<
    Item = Either<Left::Item, Right::Item>,
    Error = Either<Left::Error, Right::Error>,
>
where
    Left: StableFuture,
    Right: StableFuture,
{
    Select { left, right }
}

struct Select<Left, Right>
where
    Left: StableFuture,
    Right: StableFuture,
{
    left: Left,
    right: Right,
}

impl<Left, Right> StableFuture for Select<Left, Right>
where
    Left: StableFuture,
    Right: StableFuture,
{
    type Item = Either<Left::Item, Right::Item>;
    type Error = Either<Left::Error, Right::Error>;

    fn poll(
        mut self: Pin<Self>,
        cx: &mut task::Context,
    ) -> Poll<Self::Item, Self::Error> {
        if let Async::Ready(item) = pin_field!(self, left)
            .poll(cx)
            .map(|async| async.map(Either::Left))
            .map_err(Either::Left)?
        {
            Ok(Async::Ready(item))
        } else if let Async::Ready(item) = pin_field!(self, right)
            .poll(cx)
            .map(|async| async.map(Either::Right))
            .map_err(Either::Right)?
        {
            Ok(Async::Ready(item))
        } else {
            Ok(Async::Pending)
        }
    }
}
