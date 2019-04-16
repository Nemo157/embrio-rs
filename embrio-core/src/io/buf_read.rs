use core::{
    pin::Pin,
    task::{self, Poll},
};

use super::Read;

pub trait BufRead: Read {
    fn poll_fill_buf<'a>(
        self: Pin<&'a mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Result<&'a [u8], Self::Error>>;

    fn consume(self: Pin<&mut Self>, amount: usize);
}
