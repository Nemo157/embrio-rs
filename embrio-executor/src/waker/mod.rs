#[cfg(armv6m)]
mod armv6m;
#[cfg(armv6m)]
pub use self::armv6m::EmbrioWaker;

#[cfg(armv7m)]
mod armv7m;
#[cfg(armv7m)]
pub use self::armv7m::EmbrioWaker;

#[cfg(all(target_has_atomic = "cas", not(armv7m)))]
mod default;
#[cfg(all(target_has_atomic = "cas", not(armv7m)))]
pub use self::default::EmbrioWaker;

#[cfg(not(any(armv6m, armv7m, target_has_atomic = "cas")))]
compile_error!("Not a supported target");
