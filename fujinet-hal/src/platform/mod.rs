mod platform;

#[cfg(target_arch = "x86_64")]
mod x86;

pub mod network;

pub use platform::Platform;
pub use network::http_client::create_http_client;

#[cfg(target_arch = "x86_64")]
pub use x86::*;