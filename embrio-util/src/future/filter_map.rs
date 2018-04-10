use core::mem::Pin;
use futures::{task, Async, Poll, stable::{StableFuture, StableStream}};

use super::StableInfiniteStream;

pub fn filter_map<Inner, Result, Future, Callback>(
    inner: Inner,
    callback: Callback,
) -> FilterMap<Inner, Result, Future, Callback>
where
    Inner: StableStream,
    Future: StableFuture<Item = Option<Result>, Error = Inner::Error>,
    Callback: FnMut(Inner::Item) -> Future,
{
    FilterMap {
        current: None,
        inner,
        callback,
    }
}

pub struct FilterMap<Inner, Result, Future, Callback>
where
    Inner: StableStream,
    Future: StableFuture<Item = Option<Result>, Error = Inner::Error>,
    Callback: FnMut(Inner::Item) -> Future,
{
    current: Option<Future>,
    inner: Inner,
    callback: Callback,
}

trait OptionAsPin<T> {
    fn as_pin<'a>(self: Pin<'a, Self>) -> Option<Pin<'a, T>>;
}

impl<T> OptionAsPin<T> for Option<T> {
    fn as_pin<'a>(mut self: Pin<'a, Self>) -> Option<Pin<'a, T>> {
        match *unsafe { Pin::get_mut(&mut self) } {
            Some(ref mut item) => Some(unsafe { ::core::mem::transmute(Pin::new_unchecked(item)) }),
            None => None,
        }
    }
}

impl<Inner, Result, Future, Callback> StableStream for FilterMap<Inner, Result, Future, Callback>
where
    Inner: StableStream,
    Future: StableFuture<Item = Option<Result>, Error = Inner::Error>,
    Callback: FnMut(Inner::Item) -> Future,
{
    type Item = Result;
    type Error = Inner::Error;

    fn poll_next(
        mut self: Pin<Self>,
        cx: &mut task::Context,
    ) -> Poll<Option<Self::Item>, Self::Error> {
        Ok(loop {
            match pin_field!(self, current).as_pin() {
                Some(current) => {
                    match current.poll(cx)? {
                        Async::Ready(Some(item)) => {
                            break Async::Ready(Some(item));
                        }
                        Async::Pending => {
                            break Async::Pending;
                        }
                        Async::Ready(None) => (),
                    }
                }
                None => (),
            }
            unsafe {
                Pin::get_mut(&mut self).current = None;
            }
            match pin_field!(self, inner).poll_next(cx)? {
                Async::Ready(Some(item)) => {
                    unsafe {
                        let current = (Pin::get_mut(&mut self).callback)(item);
                        Pin::get_mut(&mut self).current = Some(current);
                    }
                    continue;
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

impl<Inner, Result, Future, Callback> StableInfiniteStream for FilterMap<Inner, Result, Future, Callback>
where
    Inner: StableInfiniteStream,
    Future: StableFuture<Item = Option<Result>, Error = Inner::Error>,
    Callback: FnMut(Inner::Item) -> Future,
{
}
