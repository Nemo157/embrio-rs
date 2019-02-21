use core::{
    pin::Pin,
    task::{Poll, Waker},
};

use super::Read;

pub trait BufRead: Read {
    fn poll_fill_buf<'a>(
        self: Pin<&'a mut Self>,
        waker: &Waker,
    ) -> Poll<Result<&'a [u8], Self::Error>>;

    fn consume(self: Pin<&mut Self>, amount: usize);
}
