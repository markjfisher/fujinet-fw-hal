pub mod host_translator;

#[cfg(target_arch = "x86_64")]
pub mod x86;

pub use host_translator::HostTranslator;

#[cfg(target_arch = "x86_64")]
pub use x86::*; 