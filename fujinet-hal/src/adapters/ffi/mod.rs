// Declare modules first
pub mod device;
pub mod network;
pub mod error;

#[cfg(test)]
pub mod test_operations;

// Then re-export what we want to be public
pub use device::*;
pub use network::*;
pub use error::*;
