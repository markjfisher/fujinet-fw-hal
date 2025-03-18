use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::device::network::manager::NetworkManagerImpl;
use super::X86HttpClientProvider;

// Global network manager with X86-specific HTTP client provider
static NETWORK_MANAGER: Lazy<Mutex<NetworkManagerImpl>> = Lazy::new(|| {
    Mutex::new(NetworkManagerImpl::new(Box::new(X86HttpClientProvider::new())))
});

/// Get the global network manager instance
pub fn get_network_manager() -> &'static Mutex<NetworkManagerImpl> {
    &NETWORK_MANAGER
} 