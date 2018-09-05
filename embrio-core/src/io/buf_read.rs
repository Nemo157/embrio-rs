use core::pin::PinMut;

use futures_core::{task, Poll};

use super::Read;

pub trait BufRead: Read {
    fn poll_fill_buf<'a>(
        self: PinMut<'a, Self>,
        cx: &mut task::Context,
    ) -> Poll<Result<&'a [u8], Self::Error>>;

    fn consume(self: PinMut<'_, Self>, amount: usize);
}
