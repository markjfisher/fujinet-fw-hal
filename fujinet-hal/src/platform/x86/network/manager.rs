use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::device::network::manager::{NetworkManager, NetworkManagerImpl};
use super::protocol_factory::create_protocol_registry;

// Global network manager instance with platform-specific HTTP client provider
static NETWORK_MANAGER: Lazy<Mutex<NetworkManagerImpl>> = Lazy::new(|| {
    // Create registry with platform-specific protocol handlers
    let registry = create_protocol_registry();
    Mutex::new(NetworkManagerImpl::with_registry(registry))
});

/// Get the global network manager instance
pub fn get_network_manager() -> &'static Mutex<impl NetworkManager> {
    &NETWORK_MANAGER
}