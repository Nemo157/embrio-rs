pub mod read_exact;
pub mod read_until;
pub mod buf_reader;
pub mod write_all;
mod flush;

use core::mem::PinMut;

use futures_core::future::Future;
use futures_util::future::poll_fn;

use embrio_core::io::Write;

pub use self::{
    read_until::read_until,
    read_exact::read_exact,
    write_all::write_all,
    flush::flush,
    buf_reader::BufReader,
};

existential type Close<'a, W: Write>: Future<Output = Result<(), W::Error>> + 'a;

pub trait WriteExt: Write {
    fn close<'a>(mut self: PinMut<'a, Self>) -> Close<'a, Self> where Self: Sized {
        poll_fn(move |cx| self.reborrow().poll_close(cx))
    }
}
