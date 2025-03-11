use async_trait::async_trait;
use fujinet_hal::device::{Device, DeviceResult, DeviceStatus};
use fujinet_hal::device::network::protocols::{ProtocolHandler, ConnectionStatus};
use fujinet_hal::device::network::{NetworkDeviceImpl, new_network_device};
use fujinet_hal::device::network::protocols::http::HttpProtocol;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct TestProtocol {
    endpoint: Arc<Mutex<Option<String>>>,
    status: Arc<Mutex<ConnectionStatus>>,
    write_calls: Arc<Mutex<Vec<Vec<u8>>>>,
    read_calls: Arc<Mutex<Vec<usize>>>,  // stores buffer sizes requested
}

impl TestProtocol {
    fn get_endpoint(&self) -> Option<String> {
        self.endpoint.lock().unwrap().clone()
    }

    fn get_write_calls(&self) -> Vec<Vec<u8>> {
        self.write_calls.lock().unwrap().clone()
    }

    fn get_read_calls(&self) -> Vec<usize> {
        self.read_calls.lock().unwrap().clone()
    }
}

// A mock implementation of the ProtocolHandler trait that we can use to test the NetworkDeviceImpl
#[async_trait]
impl ProtocolHandler for TestProtocol {
    async fn open(&mut self, endpoint: &str) -> DeviceResult<()> {
        *self.endpoint.lock().unwrap() = Some(endpoint.to_string());
        *self.status.lock().unwrap() = ConnectionStatus::Connected;
        Ok(())
    }

    async fn close(&mut self) -> DeviceResult<()> {
        *self.status.lock().unwrap() = ConnectionStatus::Disconnected;
        Ok(())
    }

    async fn read(&mut self, buf: &mut [u8]) -> DeviceResult<usize> {
        self.read_calls.lock().unwrap().push(buf.len());
        Ok(0)  // Return 0 to simulate no data available
    }

    async fn write(&mut self, buf: &[u8]) -> DeviceResult<usize> {
        self.write_calls.lock().unwrap().push(buf.to_vec());
        Ok(buf.len())
    }

    async fn status(&self) -> DeviceResult<ConnectionStatus> {
        Ok(self.status.lock().unwrap().clone())
    }

    async fn available(&self) -> DeviceResult<usize> {
        Ok(0)
    }
}

#[tokio::test]
async fn test_network_device_basic_operations() -> DeviceResult<()> {
    let protocol = TestProtocol::default();
    let endpoint = "test://example.com".to_string();
    let mut device = NetworkDeviceImpl::new(endpoint.clone(), protocol);

    // Test initial state
    assert_eq!(device.name(), "network");
    assert_eq!(device.get_status().await?, DeviceStatus::Disconnected);
    
    // Perform all device operations
    device.open().await?;
    assert_eq!(device.get_status().await?, DeviceStatus::Ready);

    let test_data = b"test data".to_vec();
    assert_eq!(device.write_bytes(&test_data).await?, test_data.len());

    let mut buf = vec![0u8; 10];
    assert_eq!(device.read_bytes(&mut buf).await?, 0);

    device.close().await?;
    assert_eq!(device.get_status().await?, DeviceStatus::Disconnected);

    // Verify block operations are not supported
    assert!(device.read_block(0, &mut buf).await.is_err());
    assert!(device.write_block(0, &test_data).await.is_err());

    // Now verify all protocol interactions
    let protocol = device.protocol();
    assert_eq!(protocol.get_endpoint(), Some(endpoint));
    
    let write_calls = protocol.get_write_calls();
    assert_eq!(write_calls.len(), 1);
    assert_eq!(write_calls[0], test_data);

    let read_calls = protocol.get_read_calls();
    assert_eq!(read_calls.len(), 1);
    assert_eq!(read_calls[0], 10);

    Ok(())
}

#[tokio::test]
async fn test_network_device_factory() -> DeviceResult<()> {
    // Test valid HTTP URL
    let device = new_network_device("http://example.com".to_string())?;
    assert_eq!(device.name(), "network");
    assert_eq!(device.get_status().await?, DeviceStatus::Disconnected);
    
    // Verify it's using HttpProtocol
    let _device = device.as_any().downcast_ref::<NetworkDeviceImpl<HttpProtocol>>()
        .expect("Device should be NetworkDeviceImpl with HttpProtocol");

    // Test valid HTTPS URL
    let device = new_network_device("https://example.com".to_string())?;
    assert_eq!(device.name(), "network");
    assert_eq!(device.get_status().await?, DeviceStatus::Disconnected);
    
    // Verify it's using HttpProtocol for HTTPS too
    let _device = device.as_any().downcast_ref::<NetworkDeviceImpl<HttpProtocol>>()
        .expect("Device should be NetworkDeviceImpl with HttpProtocol");

    // Test invalid protocol
    let result = new_network_device("invalid://example.com".to_string());
    assert!(result.is_err());

    // Test malformed URL
    let result = new_network_device("malformed_url".to_string());
    assert!(result.is_err());

    Ok(())
}
