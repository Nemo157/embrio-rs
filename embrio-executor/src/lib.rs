#![no_std]
#![feature(
    arbitrary_self_types,
    async_await,
    cfg_target_has_atomic,
    const_fn,
    never_type
)]

//! A `no_std` compatible, allocation-less, single-threaded futures executor;
//! targeted at supporting embedded use-cases.
//!
//! # Targets
//!
//! There are two primary targets supported at the moment: `thumbv6m`,
//! `thumbv7m`. These two targets use the builtin event system to allow for
//! sleeping while the future is idle waiting on input.
//!
//! There is also basic support for any target that has full atomic support.
//! This is less efficient as it uses a busy-loop while idle, but is useful for
//! testing code on a "native" target (i.e. your machine).
//!
//! # Safety
//!
//! You'll note that the signature for [`Executor::block_on`] takes `&'static
//! mut self`. This is necessary because we can't use reference counting for the
//! wakers like normal allocation-using executors. Instead we have a
//! pre-allocated waker stored in the executor and hand a reference to that out
//! to any future running on it. Because a waker must be an `Arc`-like type,
//! even after the future completes we can't know that there isn't still a live
//! reference, necessitating it to have a `'static` lifetime.
//!
//! There are two ways to use this executor, either statically
//! allocate it or dynamically allocate it and leak it (e.g. with `Box::leak` if
//! you have `std` allocation available). You need a mutable reference, so if
//! statically allocating will need to use either a `static mut` or some target
//! specific form of interior borrowing. If using `static mut` you must be
//! careful of re-entrancy to ensure you don't accidentally create multiple
//! mutable references to the executor.
//!
//! # Examples
//!
//! ```
//! #![feature(const_fn, async_await)]
//! use embrio_executor::Executor;
//!
//! static mut EXECUTOR: Executor = Executor::new();
//!
//! // Safety: We are in a non-reentrant context, so this is the only reference
//! // to the executor that will ever exist.
//! let executor = unsafe { &mut EXECUTOR };
//!
//! assert_eq!(executor.block_on(async { 5 }), 5);
//! ```

mod executor;
mod waker;

pub use self::executor::Executor;
