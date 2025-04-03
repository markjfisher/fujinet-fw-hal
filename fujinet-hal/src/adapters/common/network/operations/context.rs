use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use crate::device::network::manager::{NetworkManager, NetworkManagerImpl};

/// Context for network operations that manages dependencies
pub struct OperationsContext<M: NetworkManager> {
    pub(crate) manager: Arc<Mutex<M>>,
    pub(crate) runtime: Arc<Runtime>,
}

impl<M: NetworkManager> OperationsContext<M> {
    /// Create a new context with the given network manager
    pub fn new(manager: M) -> Self {
        // Create a new runtime for async operations
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");
        Self::new_with_runtime(manager, runtime)
    }

    /// Create a new context with the given network manager and runtime
    pub fn new_with_runtime(manager: M, runtime: Runtime) -> Self {
        Self {
            manager: Arc::new(Mutex::new(manager)),
            runtime: Arc::new(runtime),
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