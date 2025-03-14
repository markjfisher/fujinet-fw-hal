mod platform;

#[cfg(target_arch = "x86_64")]
pub mod x86;

pub use platform::Platform;

#[cfg(target_arch = "x86_64")]
pub use x86::*;