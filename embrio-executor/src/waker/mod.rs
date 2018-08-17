#[cfg(all(target_arch = "arm", target_has_atomic = "cas"))]
mod thumbv7m;

#[cfg(all(target_arch = "arm", not(target_has_atomic = "cas")))]
mod thumbv6m;

#[cfg(not(target_arch = "arm"))]
mod default;

#[cfg(all(target_arch = "arm", target_has_atomic = "cas"))]
pub use self::thumbv7m::EmbrioWaker;

#[cfg(all(target_arch = "arm", not(target_has_atomic = "cas")))]
pub use self::thumbv6m::EmbrioWaker;

#[cfg(not(target_arch = "arm"))]
pub use self::default::EmbrioWaker;
