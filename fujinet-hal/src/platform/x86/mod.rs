#[cfg(target_arch = "x86_64")]
mod network;

#[cfg(target_arch = "x86_64")]
pub use network::*;

#[cfg(not(target_arch = "x86_64"))]
compile_error!("x86 platform implementation can only be used on x86_64 targets"); 