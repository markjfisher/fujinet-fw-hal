use std::collections::HashMap;
use crate::device::DeviceResult;
use crate::device::DeviceError;
use super::ProtocolHandler;

/// Factory trait for creating protocol handlers
pub trait ProtocolHandlerFactory: Send + Sync {
    fn create_handler(&self) -> Box<dyn ProtocolHandler>;
}

/// Supported network protocols
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum NetworkProtocol {
    Http, // Represents both HTTP and HTTPS
    Tcp,  // Represents TCP
    // Add other protocols as needed
}

impl NetworkProtocol {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "http" | "https" => Some(NetworkProtocol::Http),
            "tcp" => Some(NetworkProtocol::Tcp),
            _ => None,
        }
    }
}

/// Registry for protocol handlers
/// Maps protocol types to their factories
/// Lives in device layer but accepts platform-specific factories
pub struct ProtocolRegistry {
    factories: HashMap<NetworkProtocol, Box<dyn ProtocolHandlerFactory>>,
}

impl ProtocolRegistry {
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    /// Register a factory for a specific protocol
    pub fn register(&mut self, protocol: NetworkProtocol, factory: Box<dyn ProtocolHandlerFactory>) {
        self.factories.insert(protocol, factory);
    }

    /// Create a handler for the specified protocol
    pub fn create_handler(&self, protocol: NetworkProtocol) -> DeviceResult<Box<dyn ProtocolHandler>> {
        self.factories
            .get(&protocol)
            .ok_or(DeviceError::UnsupportedProtocol)
            .map(|factory| factory.create_handler())
    }

    /// Check if a protocol is supported
    pub fn supports_protocol(&self, protocol: &NetworkProtocol) -> bool {
        self.factories.contains_key(protocol)
    }

    /// Get list of supported protocols
    pub fn supported_protocols(&self) -> Vec<NetworkProtocol> {
        self.factories.keys().cloned().collect()
    }
}

impl Default for ProtocolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use super::super::ConnectionStatus;

    // Simple mock protocol for testing
    struct MockProtocol;
    
    #[async_trait]
    impl ProtocolHandler for MockProtocol {
        fn as_any(&self) -> &dyn std::any::Any { self }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

        async fn open(&mut self, _endpoint: &str) -> DeviceResult<()> {
            Ok(())
        }

        async fn close(&mut self) -> DeviceResult<()> {
            Ok(())
        }

        async fn read(&mut self, _buf: &mut [u8]) -> DeviceResult<usize> {
            Ok(0)
        }

        async fn write(&mut self, buf: &[u8]) -> DeviceResult<usize> {
            Ok(buf.len())
        }

        async fn status(&self) -> DeviceResult<ConnectionStatus> {
            Ok(ConnectionStatus::Connected)
        }

        async fn available(&self) -> DeviceResult<usize> {
            Ok(0)
        }
    }

    struct MockFactory;
    
    impl ProtocolHandlerFactory for MockFactory {
        fn create_handler(&self) -> Box<dyn ProtocolHandler> {
            Box::new(MockProtocol)
        }
    }

    #[tokio::test]
    async fn test_protocol_registry() {
        let mut registry = ProtocolRegistry::new();
        
        // Test registration
        registry.register(NetworkProtocol::Http, Box::new(MockFactory));
        assert!(registry.supports_protocol(&NetworkProtocol::Http));
        
        // Test unsupported protocol
        assert!(!registry.supports_protocol(&NetworkProtocol::Tcp));
        
        // Test handler creation
        let handler = registry.create_handler(NetworkProtocol::Http);
        assert!(handler.is_ok());
        
        // Test unsupported protocol error
        let handler = registry.create_handler(NetworkProtocol::Tcp);
        assert!(handler.is_err());

        // Test supported protocols list
        let protocols = registry.supported_protocols();
        assert_eq!(protocols.len(), 1);
        assert_eq!(protocols[0], NetworkProtocol::Http);
    }
} 