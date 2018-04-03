//! Utility functions for [`StableFuture`](::futures::StableFuture). These
//! should hopefully all be subsumed by upstream utility functions eventually.

macro_rules! pin_field {
    ($pin:expr, $field:ident) => {
        unsafe { Pin::map(&mut $pin, |s| &mut s.$field) }
    };
}

mod join;

pub use self::join::join;
