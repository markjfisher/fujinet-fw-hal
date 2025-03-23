use super::NetworkProtocol;
use super::registry::ProtocolRegistry;
use crate::device::network::NetworkDevice;
use crate::device::DeviceResult;
use crate::device::network::NetworkUrl;
use crate::device::network::network_device::NetworkDeviceImpl;

/// Factory for creating and managing network devices
/// Uses ProtocolRegistry to create appropriate protocol handlers
pub struct ProtocolFactory {
    // Each device can have one active protocol handler
    active_devices: [Option<Box<dyn NetworkDevice>>; 8],
    registry: ProtocolRegistry,
}

impl ProtocolFactory {
    pub fn new(registry: ProtocolRegistry) -> Self {
        Self {
            active_devices: [None, None, None, None, None, None, None, None],
            registry,
        }
    }

    // Returns device_id if successful
    pub async fn get_or_create_device(
        &mut self, 
        device_id: usize,
        protocol: NetworkProtocol,
        url: &NetworkUrl
    ) -> DeviceResult<usize> {
        debug_assert!(device_id < self.active_devices.len(), "device_id out of bounds");
        
        // If we already have an active device
        if self.active_devices[device_id].is_some() {
            return Ok(device_id);
        }

        // Create new device with protocol handler from registry
        let handler = self.registry.create_handler(protocol)?;
        let device = NetworkDeviceImpl::new(url.url.clone(), handler);
        
        self.active_devices[device_id] = Some(Box::new(device));
        Ok(device_id)
    }

    // Get device by ID
    pub fn get_device(&mut self, device_id: usize) -> Option<&mut Box<dyn NetworkDevice>> {
        self.active_devices[device_id].as_mut()
    }

    pub async fn close_device(&mut self, device_id: usize) -> DeviceResult<()> {
        if let Some(device) = &mut self.active_devices[device_id] {
            device.disconnect().await?;
            self.active_devices[device_id] = None;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::ConnectionStatus;
    use async_trait::async_trait;
    use std::any::Any;

    struct MockProtocol;

    #[async_trait]
    impl ProtocolHandler for MockProtocol {
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }

        async fn open(&mut self, _: &str) -> DeviceResult<()> { Ok(()) }
        async fn close(&mut self) -> DeviceResult<()> { Ok(()) }
        async fn read(&mut self, _: &mut [u8]) -> DeviceResult<usize> { Ok(0) }
        async fn write(&mut self, buf: &[u8]) -> DeviceResult<usize> { Ok(buf.len()) }
        async fn status(&self) -> DeviceResult<ConnectionStatus> { Ok(ConnectionStatus::Connected) }
        async fn available(&self) -> DeviceResult<usize> { Ok(0) }
    }

    struct MockFactory;

    impl super::super::registry::ProtocolHandlerFactory for MockFactory {
        fn create_handler(&self) -> Box<dyn ProtocolHandler> {
            Box::new(MockProtocol)
        }
    }

    #[tokio::test]
    async fn test_protocol_factory() -> DeviceResult<()> {
        let mut registry = ProtocolRegistry::new();
        registry.register(NetworkProtocol::Http, Box::new(MockFactory));
        
        let mut factory = ProtocolFactory::new(registry);
        
        // Test device creation
        let url = NetworkUrl::parse("N:http://test.com")?;
        let device_id = factory.get_or_create_device(0, NetworkProtocol::Http, &url).await?;
        assert_eq!(device_id, 0);
        
        // Test getting existing device
        assert!(factory.get_device(0).is_some());
        
        // Test closing device
        factory.close_device(0).await?;
        assert!(factory.get_device(0).is_none());
        
        Ok(())
    }
}
