//! Utility functions for [`StableFuture`](::futures::StableFuture). These
//! should hopefully all be subsumed by upstream utility functions eventually.

macro_rules! pin_field {
    ($pin:expr, $field:ident) => {
        unsafe { Pin::map(&mut $pin, |s| &mut s.$field) }
    };
}

macro_rules! pin_fields {
    ($pin:expr, ($($field:ident),+ $(,)?)) => {
        unsafe {
            let s = Pin::get_mut(&mut $pin);
            ($(Pin::new_unchecked(&mut s.$field),)+)
        }
    };
}

macro_rules! unpin_field {
    ($pin:expr, $field:ident) => {
        // TODO: This should be able to use the safe DerefMut impl for Unpin
        // fields, and in the meantime is definitely unsafe
        unsafe { Pin::get_mut(&mut Pin::map(&mut $pin, |s| &mut s.$field)) }
    };
}

mod filter;
mod filter_map;
mod first;
mod join;
mod select;
mod traits;

pub use self::filter::filter;
pub use self::filter_map::filter_map;
pub use self::first::first;
pub use self::join::join;
pub use self::select::select;
pub use self::traits::StableInfiniteStream;
