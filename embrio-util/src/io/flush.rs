use core::mem::PinMut;

use futures_core::future::Future;
use futures_util::future::poll_fn;

use embrio_core::io::Write;

pub fn flush<W: Write>(
    mut this: PinMut<'a, W>,
) -> impl Future<Output = Result<(), W::Error>> + '_ {
    poll_fn(move |cx| this.reborrow().poll_flush(cx))
}

