use core::task::{RawWaker, RawWakerVTable, Waker};

#[cfg(armv6m)]
mod armv6m;
#[cfg(armv6m)]
pub use self::armv6m::EmbrioWaker;

#[cfg(armv7m)]
mod armv7m;
#[cfg(armv7m)]
pub use self::armv7m::EmbrioWaker;

#[cfg(all(target_has_atomic = "ptr", not(armv7m)))]
mod default;
#[cfg(all(target_has_atomic = "ptr", not(armv7m)))]
pub use self::default::EmbrioWaker;

#[cfg(not(any(armv6m, armv7m, target_has_atomic = "ptr")))]
compile_error!("Not a supported target");

static EMBRIO_WAKER_RAW_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    |data| unsafe { (*(data as *const EmbrioWaker)).raw_waker() },
    |data| unsafe { (*(data as *const EmbrioWaker)).wake() },
    |data| unsafe { (*(data as *const EmbrioWaker)).wake() },
    |_| (/* Noop */),
);

impl EmbrioWaker {
    pub(crate) fn waker(&'static self) -> Waker {
        unsafe { Waker::from_raw(self.raw_waker()) }
    }

    pub(crate) fn raw_waker(&'static self) -> RawWaker {
        RawWaker::new(
            self as *const _ as *const (),
            &EMBRIO_WAKER_RAW_WAKER_VTABLE,
        )
    }
}
