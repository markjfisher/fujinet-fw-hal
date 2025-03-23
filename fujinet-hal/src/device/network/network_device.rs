use async_trait::async_trait;
use crate::device::{Device, DeviceResult, DeviceError, DeviceStatus};
use std::any::Any;
use super::protocols::{ProtocolHandler, ConnectionStatus};
use super::url::NetworkUrl;

#[async_trait]
pub trait NetworkDevice: Device + Send + Sync {
    /// Connects to a network endpoint
    async fn connect(&mut self, endpoint: &str) -> DeviceResult<()>;

    /// Disconnects from the current endpoint
    async fn disconnect(&mut self) -> DeviceResult<()>;

    /// Opens a network connection using the specified URL
    /// The URL determines which protocol handler to use
    async fn open_url(&mut self, url: &NetworkUrl) -> DeviceResult<()>;

    /// Gets the protocol handler for this device
    fn protocol_handler(&mut self) -> &mut dyn ProtocolHandler;
}

pub struct NetworkDeviceImpl {
    endpoint: String,
    protocol: Box<dyn ProtocolHandler>,
}

impl NetworkDeviceImpl {
    pub fn new(endpoint: String, protocol: Box<dyn ProtocolHandler>) -> Self {
        Self {
            endpoint,
            protocol,
        }
    }

    pub fn protocol(&self) -> &dyn ProtocolHandler {
        &*self.protocol
    }

    pub fn protocol_mut(&mut self) -> &mut dyn ProtocolHandler {
        &mut *self.protocol
    }
}

#[async_trait]
impl NetworkDevice for NetworkDeviceImpl {
    async fn connect(&mut self, endpoint: &str) -> DeviceResult<()> {
        self.endpoint = endpoint.to_string();
        self.protocol.open(endpoint).await
    }

    async fn disconnect(&mut self) -> DeviceResult<()> {
        self.protocol.close().await
    }

    async fn open_url(&mut self, url: &NetworkUrl) -> DeviceResult<()> {
        self.connect(&url.url).await
    }

    fn protocol_handler(&mut self) -> &mut dyn ProtocolHandler {
        &mut *self.protocol
    }
}

#[async_trait]
impl Device for NetworkDeviceImpl {
    fn name(&self) -> &str {
        "network"
    }

    async fn open(&mut self) -> DeviceResult<()> {
        self.protocol.open(&self.endpoint).await
    }

    async fn close(&mut self) -> DeviceResult<()> {
        self.protocol.close().await
    }

    async fn read_bytes(&mut self, buf: &mut [u8]) -> DeviceResult<usize> {
        self.protocol.read(buf).await
    }

    async fn write_bytes(&mut self, buf: &[u8]) -> DeviceResult<usize> {
        self.protocol.write(buf).await
    }

    async fn read_block(&mut self, _block: u32, _buf: &mut [u8]) -> DeviceResult<usize> {
        Err(DeviceError::InvalidOperation)
    }

    async fn write_block(&mut self, _block: u32, _buf: &[u8]) -> DeviceResult<usize> {
        Err(DeviceError::InvalidOperation)
    }

    async fn get_status(&self) -> DeviceResult<DeviceStatus> {
        match self.protocol.status().await? {
            ConnectionStatus::Connected => Ok(DeviceStatus::Ready),
            ConnectionStatus::Connecting => Ok(DeviceStatus::Disconnected), // Still establishing connection
            ConnectionStatus::Disconnected => Ok(DeviceStatus::Disconnected),
            ConnectionStatus::Error(_) => Ok(DeviceStatus::Error),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::network::protocols::{ProtocolHandler, ConnectionStatus};
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct TestProtocol {
        status: Arc<Mutex<ConnectionStatus>>,
        write_data: Arc<Mutex<Vec<u8>>>,
        read_data: Arc<Mutex<Vec<u8>>>,
    }

    #[async_trait]
    impl ProtocolHandler for TestProtocol {
        fn as_any(&self) -> &dyn std::any::Any { self }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

        async fn open(&mut self, _endpoint: &str) -> DeviceResult<()> {
            *self.status.lock().unwrap() = ConnectionStatus::Connected;
            Ok(())
        }

        async fn close(&mut self) -> DeviceResult<()> {
            *self.status.lock().unwrap() = ConnectionStatus::Disconnected;
            Ok(())
        }

        async fn read(&mut self, buf: &mut [u8]) -> DeviceResult<usize> {
            let read_data = self.read_data.lock().unwrap();
            let len = std::cmp::min(buf.len(), read_data.len());
            buf[..len].copy_from_slice(&read_data[..len]);
            Ok(len)
        }

        async fn write(&mut self, buf: &[u8]) -> DeviceResult<usize> {
            self.write_data.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        async fn status(&self) -> DeviceResult<ConnectionStatus> {
            Ok(self.status.lock().unwrap().clone())
        }

        async fn available(&self) -> DeviceResult<usize> {
            Ok(self.read_data.lock().unwrap().len())
        }
    }

    #[tokio::test]
    async fn test_device_lifecycle() -> DeviceResult<()> {
        let device = NetworkDeviceImpl::new(
            "test://example.com".to_string(),
            TestProtocol::default()
        );

        // Test initial state
        assert_eq!(device.name(), "network");
        assert_eq!(device.get_status().await?, DeviceStatus::Disconnected);

        // Test open
        device.open().await?;
        assert_eq!(device.get_status().await?, DeviceStatus::Ready);

        // Test close
        device.close().await?;
        assert_eq!(device.get_status().await?, DeviceStatus::Disconnected);
        Ok(())
    }

    #[tokio::test]
    async fn test_device_io() -> DeviceResult<()> {
        let device = NetworkDeviceImpl::new(
            "test://example.com".to_string(),
            TestProtocol::default()
        );

        device.open().await?;

        // Test write
        let test_data = b"Hello, World!".to_vec();
        assert_eq!(device.write_bytes(&test_data).await?, test_data.len());

        // Test read
        let mut buf = vec![0u8; 100];
        let read_len = device.read_bytes(&mut buf).await?;
        assert_eq!(read_len, 0); // No data available by default

        device.close().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_block_operations_not_supported() -> DeviceResult<()> {
        let device = NetworkDeviceImpl::new(
            "test://example.com".to_string(),
            TestProtocol::default()
        );

        let mut buf = vec![0u8; 100];
        let test_data = vec![1u8; 100];

        // Block operations should return errors
        assert!(device.read_block(0, &mut buf).await.is_err());
        assert!(device.write_block(0, &test_data).await.is_err());
        Ok(())
    }
} 