use core::{cmp, pin::PinMut};

use embrio_core::io::BufRead;
use futures_core::{future::Future, task::Poll};
use futures_util::{future::poll_fn, ready};

pub fn read_until<'a, R: BufRead + 'a>(
    mut this: PinMut<'a, R>,
    byte: u8,
    mut buf: impl AsMut<[u8]> + 'a,
) -> impl Future<Output = Result<Result<usize, usize>, R::Error>> + 'a {
    let mut position = 0;
    poll_fn(move |cx| {
        let buf = buf.as_mut();
        while position < buf.len() {
            let (done, used) = {
                let available = ready!(this.reborrow().poll_fill_buf(cx))?;
                let limit = cmp::min(available.len(), buf.len() - position);
                if let Some(i) = memchr::memchr(byte, &available[..limit]) {
                    buf[position..position + i + 1]
                        .copy_from_slice(&available[..i + 1]);
                    (true, i + 1)
                } else {
                    buf[position..(position + limit)]
                        .copy_from_slice(&available[..limit]);
                    (false, limit)
                }
            };
            this.reborrow().consume(used);
            position += used;
            if done || used == 0 {
                return Poll::Ready(Ok(Ok(position)));
            }
        }
        return Poll::Ready(Ok(Err(buf.len())));
    })
}
