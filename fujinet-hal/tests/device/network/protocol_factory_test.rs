use fujinet_hal::device::network::protocols::{
    ProtocolFactory, NetworkProtocol, ProtocolRegistry, ProtocolHandlerFactory, ProtocolHandler,
    HttpProtocol
};
use fujinet_hal::device::network::url::NetworkUrl;
use fujinet_hal::device::DeviceResult;
use std::sync::Arc;
use crate::common::mocks::MockHttpClientProvider;

struct HttpProtocolFactory {
    provider: Arc<MockHttpClientProvider>,
}

impl ProtocolHandlerFactory for HttpProtocolFactory {
    fn create_handler(&self) -> Box<dyn ProtocolHandler> {
        // Create HTTP protocol with our mock client
        let protocol = HttpProtocol::new(&*self.provider);
        Box::new(protocol)
    }
}

fn setup_test_registry() -> ProtocolRegistry {
    let provider = Arc::new(MockHttpClientProvider::default());
    let mut registry = ProtocolRegistry::new();
    registry.register(NetworkProtocol::Http, Box::new(HttpProtocolFactory { provider }));
    registry
}

#[tokio::test]
async fn test_protocol_factory_integration() -> DeviceResult<()> {
    let registry = setup_test_registry();
    let mut factory = ProtocolFactory::new(registry);
    
    // Test HTTP protocol creation
    let url = NetworkUrl::parse("N:http://example.com")?;
    let device_id = factory.get_or_create_device(0, NetworkProtocol::Http, &url).await?;
    let device = factory.get_device(device_id).unwrap();
    
    // Test device lifecycle
    device.open().await?;
    assert!(device.get_status().await.is_ok());
    device.close().await?;

    // Test device reuse
    let same_device_id = factory.get_or_create_device(0, NetworkProtocol::Http, &url).await?;
    assert_eq!(device_id, same_device_id);

    Ok(())
}

#[tokio::test]
async fn test_protocol_factory_invalid_urls() -> DeviceResult<()> {
    let registry = setup_test_registry();
    let mut factory = ProtocolFactory::new(registry);

    // Test invalid protocol
    let url = NetworkUrl::parse("N:invalid://example.com")?;
    assert!(factory.get_or_create_device(0, NetworkProtocol::Http, &url).await.is_err());

    // Test malformed URL
    assert!(NetworkUrl::parse("malformed_url").is_err());

    Ok(())
} 