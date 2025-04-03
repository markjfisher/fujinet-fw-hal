use std::collections::HashMap;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use fujinet_hal::device::DeviceResult;
use fujinet_hal::device::network::protocols::{
    ProtocolHandler,
    ProtocolHandlerFactory,
    NetworkProtocol,
    ConnectionStatus,
    ProtocolRegistry,
};
use fujinet_hal::device::network::manager::{NetworkManager, NetworkManagerImpl};

// Mock response data for assertions
#[derive(Default, Clone)]
struct MockResponseData {
    http_requests: Vec<(String, Vec<u8>)>,  // (url, body)
    tcp_data: HashMap<String, Vec<u8>>,     // endpoint -> data
}

// Mock HTTP Protocol
struct MockHttpProtocol {
    response_data: Arc<Mutex<MockResponseData>>,
    endpoint: String,
    status: ConnectionStatus,
}

#[async_trait]
impl ProtocolHandler for MockHttpProtocol {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

    async fn open(&mut self, endpoint: &str) -> DeviceResult<()> {
        self.endpoint = endpoint.to_string();
        self.status = ConnectionStatus::Connected;
        Ok(())
    }

    async fn close(&mut self) -> DeviceResult<()> {
        self.status = ConnectionStatus::Disconnected;
        Ok(())
    }

    async fn read(&mut self, _buf: &mut [u8]) -> DeviceResult<usize> {
        Ok(0)
    }

    async fn write(&mut self, data: &[u8]) -> DeviceResult<usize> {
        let mut response_data = self.response_data.lock().unwrap();
        response_data.http_requests.push((self.endpoint.clone(), data.to_vec()));
        Ok(data.len())
    }

    async fn status(&self) -> DeviceResult<ConnectionStatus> {
        Ok(self.status.clone())
    }

    async fn available(&self) -> DeviceResult<usize> {
        Ok(0)
    }
}

// Mock TCP Protocol
struct MockTcpProtocol {
    response_data: Arc<Mutex<MockResponseData>>,
    endpoint: String,
    status: ConnectionStatus,
}

#[async_trait]
impl ProtocolHandler for MockTcpProtocol {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

    async fn open(&mut self, endpoint: &str) -> DeviceResult<()> {
        self.endpoint = endpoint.to_string();
        self.status = ConnectionStatus::Connected;
        Ok(())
    }

    async fn close(&mut self) -> DeviceResult<()> {
        self.status = ConnectionStatus::Disconnected;
        Ok(())
    }

    async fn read(&mut self, buf: &mut [u8]) -> DeviceResult<usize> {
        let response_data = self.response_data.lock().unwrap();
        if let Some(data) = response_data.tcp_data.get(&self.endpoint) {
            let len = std::cmp::min(buf.len(), data.len());
            buf[..len].copy_from_slice(&data[..len]);
            Ok(len)
        } else {
            Ok(0)
        }
    }

    async fn write(&mut self, _buf: &[u8]) -> DeviceResult<usize> {
        Ok(0)
    }

    async fn status(&self) -> DeviceResult<ConnectionStatus> {
        Ok(self.status.clone())
    }

    async fn available(&self) -> DeviceResult<usize> {
        let response_data = self.response_data.lock().unwrap();
        Ok(response_data.tcp_data.get(&self.endpoint).map_or(0, |data| data.len()))
    }
}

// Factories for our mock protocols
struct MockHttpFactory {
    response_data: Arc<Mutex<MockResponseData>>,
}

impl ProtocolHandlerFactory for MockHttpFactory {
    fn create_handler(&self) -> Box<dyn ProtocolHandler> {
        Box::new(MockHttpProtocol {
            response_data: self.response_data.clone(),
            endpoint: String::new(),
            status: ConnectionStatus::Disconnected,
        })
    }
}

struct MockTcpFactory {
    response_data: Arc<Mutex<MockResponseData>>,
}

impl ProtocolHandlerFactory for MockTcpFactory {
    fn create_handler(&self) -> Box<dyn ProtocolHandler> {
        Box::new(MockTcpProtocol {
            response_data: self.response_data.clone(),
            endpoint: String::new(),
            status: ConnectionStatus::Disconnected,
        })
    }
}

#[tokio::test]
async fn test_protocol_registry_with_mocks() -> DeviceResult<()> {
    // Create shared test data
    let response_data = Arc::new(Mutex::new(MockResponseData::default()));
    
    // Prepare some test TCP data
    {
        let mut data = response_data.lock().unwrap();
        data.tcp_data.insert("tcp://test-server:8080".to_string(), b"Hello TCP!".to_vec());
    }

    // Create registry with mock factories
    let mut registry = ProtocolRegistry::new();
    registry.register(
        NetworkProtocol::Http,
        Box::new(MockHttpFactory { response_data: response_data.clone() })
    );
    registry.register(
        NetworkProtocol::Tcp,
        Box::new(MockTcpFactory { response_data: response_data.clone() })
    );

    // Create network manager with mock registry
    let mut manager = NetworkManagerImpl::with_registry(registry);

    // Test HTTP POST - using device 0
    manager.open_device("N:http://api.example.com", 0, 0).await?;
    
    // Verify device 0 is connected and is HTTP protocol
    let device = manager.get_network_device(0)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Device 0 not found"))?;
    
    // Get the protocol handler from the device
    let protocol = device.protocol_handler();
    
    let http = protocol.as_any_mut()
        .downcast_mut::<MockHttpProtocol>()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Failed to downcast to HTTP protocol"))?;
    
    let status = http.status().await?;
    assert_eq!(status, ConnectionStatus::Connected, "HTTP device should be connected");
    
    http.write(b"POST data").await?;

    // Test TCP read - using device 1
    manager.open_device("N2:tcp://test-server:8080", 1, 1).await?;
    
    // Verify device 1 is connected and is TCP protocol
    let device = manager.get_network_device(1)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Device 1 not found"))?;
    
    // Get the protocol handler from the device
    let protocol = device.protocol_handler();
    
    let tcp = protocol.as_any_mut()
        .downcast_mut::<MockTcpProtocol>()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Failed to downcast to TCP protocol"))?;
    
    let status = tcp.status().await?;
    assert_eq!(status, ConnectionStatus::Connected, "TCP device should be connected");
    
    let mut buf = [0u8; 32];
    let n = tcp.read(&mut buf).await?;
    assert_eq!(&buf[..n], b"Hello TCP!");

    // Verify HTTP requests
    let data = response_data.lock().unwrap();
    assert_eq!(data.http_requests.len(), 1, "Should have exactly one HTTP request");
    assert_eq!(data.http_requests[0].0, "http://api.example.com");
    assert_eq!(data.http_requests[0].1, b"POST data");

    Ok(())
} 