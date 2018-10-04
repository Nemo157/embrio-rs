use core::pin::Pin;

use futures_core::{task, Poll};

use super::Read;

pub trait BufRead: Read {
    fn poll_fill_buf<'a>(
        self: Pin<&'a mut Self>,
        lw: &task::LocalWaker,
    ) -> Poll<Result<&'a [u8], Self::Error>>;

    fn consume(self: Pin<&mut Self>, amount: usize);
}
