use core::{future::Future, pin::Pin};

use embrio_core::io::Write;
use futures_util::future::poll_fn;

pub fn flush<W: Write>(
    mut this: Pin<&mut W>,
) -> impl Future<Output = Result<(), W::Error>> + '_ {
    poll_fn(move |cx| this.as_mut().poll_flush(cx))
}
