use core::{future::Future, pin::PinMut};

use embrio_core::io::Write;
use futures_util::future::poll_fn;

pub fn close<W: Write>(
    mut this: PinMut<'_, W>,
) -> impl Future<Output = Result<(), W::Error>> + '_ {
    poll_fn(move |cx| this.reborrow().poll_close(cx))
}
