use core::mem::PinMut;

use super::Read;

use futures_core::{task, Poll};

pub trait BufRead: Read {
    fn poll_fill_buf<'a>(
        self: PinMut<'a, Self>,
        cx: &mut task::Context,
    ) -> Poll<Result<&'a [u8], Self::Error>>;

    fn consume(self: PinMut<'_, Self>, amount: usize);
}
