use crate::device::network::protocols::{
    ProtocolHandlerFactory,
    ProtocolHandler,
    NetworkProtocol,
    ProtocolRegistry,
    HttpProtocol,
};
use super::http_client::DefaultHttpClientProvider;
use std::sync::Arc;

/// Factory for creating HTTP protocol handlers
pub struct HttpProtocolFactory {
    provider: Arc<DefaultHttpClientProvider>,
}

impl ProtocolHandlerFactory for HttpProtocolFactory {
    fn create_handler(&self) -> Box<dyn ProtocolHandler> {
        Box::new(HttpProtocol::new(self.provider.clone()))
    }
}

/// Create a protocol registry with platform-specific handlers
pub fn create_protocol_registry() -> ProtocolRegistry {
    let mut registry = ProtocolRegistry::new();
    
    // Register HTTP protocol handler
    let provider = Arc::new(DefaultHttpClientProvider);
    registry.register(NetworkProtocol::Http, Box::new(HttpProtocolFactory { provider }));
    
    registry
} 