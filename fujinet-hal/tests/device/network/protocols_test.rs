use fujinet_hal::device::network::protocols::{NetworkProtocol, ProtocolRegistry, ProtocolHandlerFactory, ProtocolHandler};
use fujinet_hal::device::DeviceResult;
use fujinet_hal::device::network::protocols::ConnectionStatus;
use async_trait::async_trait;

// Mock HTTP protocol for testing
struct MockHttpProtocol;

#[async_trait]
impl ProtocolHandler for MockHttpProtocol {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    
    async fn open(&mut self, _: &str) -> DeviceResult<()> { Ok(()) }
    async fn close(&mut self) -> DeviceResult<()> { Ok(()) }
    async fn read(&mut self, _: &mut [u8]) -> DeviceResult<usize> { Ok(0) }
    async fn write(&mut self, buf: &[u8]) -> DeviceResult<usize> { Ok(buf.len()) }
    async fn status(&self) -> DeviceResult<ConnectionStatus> { Ok(ConnectionStatus::Connected) }
    async fn available(&self) -> DeviceResult<usize> { Ok(0) }
}

struct MockHttpFactory;

impl ProtocolHandlerFactory for MockHttpFactory {
    fn create_handler(&self) -> Box<dyn ProtocolHandler> {
        Box::new(MockHttpProtocol)
    }
}

#[tokio::test]
async fn test_protocol_registration_and_creation() {
    let mut registry = ProtocolRegistry::new();
    
    // Register HTTP protocol
    registry.register(NetworkProtocol::Http, Box::new(MockHttpFactory));
    
    // Verify protocol support
    assert!(registry.supports_protocol(&NetworkProtocol::Http));
    assert!(!registry.supports_protocol(&NetworkProtocol::Tcp));
    
    // Create and test handler
    let handler = registry.create_handler(NetworkProtocol::Http).unwrap();
    assert!(handler.status().await.is_ok());
}

#[tokio::test]
async fn test_protocol_lifecycle() {
    let mut registry = ProtocolRegistry::new();
    registry.register(NetworkProtocol::Http, Box::new(MockHttpFactory));
    
    // Create handler and test lifecycle
    let mut handler = registry.create_handler(NetworkProtocol::Http).unwrap();
    
    // Test open
    assert!(handler.open("http://test.com").await.is_ok());
    
    // Test operations
    let mut buf = vec![0; 10];
    assert!(handler.read(&mut buf).await.is_ok());
    assert!(handler.write(&[1, 2, 3]).await.is_ok());
    
    // Test close
    assert!(handler.close().await.is_ok());
    
    // Verify final status
    assert!(matches!(
        handler.status().await.unwrap(),
        ConnectionStatus::Connected
    ));
} 