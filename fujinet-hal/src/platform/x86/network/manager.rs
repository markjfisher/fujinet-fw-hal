use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::device::network::manager::{NetworkManager, NetworkManagerImpl};

// Global network manager instance with platform-specific HTTP client provider
static NETWORK_MANAGER: Lazy<Mutex<NetworkManagerImpl>> = Lazy::new(|| {
    Mutex::new(NetworkManagerImpl::new())
});

/// Get the global network manager instance
pub fn get_network_manager() -> &'static Mutex<impl NetworkManager> {
    &NETWORK_MANAGER
} 