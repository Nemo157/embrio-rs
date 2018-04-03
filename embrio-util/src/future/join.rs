use core::{marker::Unpin, mem::Pin};
use futures::{task, Async, Poll, future::Either, stable::StableFuture};

pub fn join<Left, Right>(
    left: Left,
    right: Right,
) -> impl StableFuture<
    Item = (Left::Item, Right::Item),
    Error = Either<Left::Error, Right::Error>,
>
where
    Left: StableFuture,
    Right: StableFuture,
    Left::Item: Unpin,
    Right::Item: Unpin,
{
    Join {
        left,
        right,
        left_result: None,
        right_result: None,
    }
}

struct Join<Left, Right>
where
    Left: StableFuture,
    Right: StableFuture,
    Left::Item: Unpin,
    Right::Item: Unpin,
{
    left: Left,
    right: Right,
    left_result: Option<Left::Item>,
    right_result: Option<Right::Item>,
}

impl<Left, Right> StableFuture for Join<Left, Right>
where
    Left: StableFuture,
    Right: StableFuture,
    Left::Item: Unpin,
    Right::Item: Unpin,
{
    type Item = (Left::Item, Right::Item);
    type Error = Either<Left::Error, Right::Error>;

    fn poll(
        mut self: Pin<Self>,
        cx: &mut task::Context,
    ) -> Poll<Self::Item, Self::Error> {
        Ok(loop {
            match (
                pin_field!(self, left_result).take(),
                pin_field!(self, right_result).take(),
            ) {
                (Some(l), Some(r)) => {
                    break Async::Ready((l, r));
                }
                (None, Some(r)) => {
                    if let Async::Ready(l) = pin_field!(self, left)
                        .poll(cx)
                        .map_err(Either::Left)?
                    {
                        break Async::Ready((l, r));
                    } else {
                        *pin_field!(self, right_result) = Some(r);
                        break Async::Pending;
                    }
                }
                (Some(l), None) => {
                    if let Async::Ready(r) = pin_field!(self, right)
                        .poll(cx)
                        .map_err(Either::Right)?
                    {
                        break Async::Ready((l, r));
                    } else {
                        *pin_field!(self, left_result) = Some(l);
                        break Async::Pending;
                    }
                }
                (None, None) => {
                    if let Async::Ready(l) = pin_field!(self, left)
                        .poll(cx)
                        .map_err(Either::Left)?
                    {
                        *pin_field!(self, left_result) = Some(l);
                        continue;
                    }
                    if let Async::Ready(r) = pin_field!(self, right)
                        .poll(cx)
                        .map_err(Either::Right)?
                    {
                        *pin_field!(self, right_result) = Some(r);
                        continue;
                    }
                    break Async::Pending;
                }
            }
        })
    }
}
