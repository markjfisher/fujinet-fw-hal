pub mod operations;

// Re-export the device layer's NetworkManager trait and get_network_manager function
pub use crate::device::network::manager::{NetworkManager, get_network_manager as get_manager};
pub use operations::*; 