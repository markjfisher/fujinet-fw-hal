use std::sync::{Arc, Mutex};
use crate::device::network::manager::{NetworkManager, NetworkManagerImpl};

/// Context for network operations that manages dependencies
pub struct OperationsContext<M: NetworkManager> {
    pub(crate) manager: Arc<Mutex<M>>,
}

impl<M: NetworkManager> OperationsContext<M> {
    /// Create a new context with the given network manager
    pub fn new(manager: M) -> Self {
        Self {
            manager: Arc::new(Mutex::new(manager))
        }
    }
}

impl OperationsContext<NetworkManagerImpl> {
    /// Create a default context using production implementations
    pub fn default() -> Self {
        let manager = NetworkManagerImpl::new();
        Self::new(manager)
    }
} 