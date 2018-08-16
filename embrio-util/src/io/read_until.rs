use core::cmp;
use core::mem::PinMut;

use futures_core::future::Future;
use futures_core::task::Poll;
use futures_util::{ready, future::poll_fn};

use embrio_core::io::BufRead;

use crate::utils::Captures;

pub fn read_until<'a, 'b: 'a, R: BufRead + 'a>(
    mut this: PinMut<'a, R>,
    byte: u8,
    buf: &'b mut [u8],
) -> impl Future<Output = Result<Result<usize, usize>, R::Error>>
         + Captures<'a>
         + Captures<'b> {
    let mut position = 0;
    poll_fn(move |cx| {
        while position < buf.len() {
            let (done, used) = {
                let available = ready!(this.reborrow().poll_fill_buf(cx))?;
                let limit = cmp::min(available.len(), buf.len() - position);
                if let Some(i) = memchr::memchr(byte, &available[..limit]) {
                    buf[position..position + i + 1].copy_from_slice(&available[..i + 1]);
                    (true, i + 1)
                } else {
                    buf[position..(position + limit)].copy_from_slice(&available[..limit]);
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
