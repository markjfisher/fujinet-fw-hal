pub mod protocols;

use fujinet_core::error::DeviceResult;
use fujinet_core::device::Device;
use async_trait::async_trait;
use std::any::Any;

pub use protocols::{ProtocolHandler, ConnectionStatus, AnyProtocolHandler};
pub use protocols::http::{HttpProtocol, HttpProtocolHandler};

pub struct NetworkDevice<P: ProtocolHandler + 'static> {
    endpoint: String,
    protocol: P,
}

impl<P: ProtocolHandler> NetworkDevice<P> {
    pub fn new(endpoint: String, protocol: P) -> Self {
        Self {
            endpoint,
            protocol,
        }
    }
}

#[async_trait]
impl<P: ProtocolHandler> Device for NetworkDevice<P> {
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
        Err(fujinet_core::error::DeviceError::NotSupported)
    }

    async fn write_block(&mut self, _block: u32, _buf: &[u8]) -> DeviceResult<usize> {
        Err(fujinet_core::error::DeviceError::NotSupported)
    }

    async fn get_status(&self) -> DeviceResult<fujinet_core::device::DeviceStatus> {
        match self.protocol.status().await? {
            ConnectionStatus::Connected => Ok(fujinet_core::device::DeviceStatus::Ready),
            ConnectionStatus::Disconnected => Ok(fujinet_core::device::DeviceStatus::Disconnected),
            ConnectionStatus::Connecting => Ok(fujinet_core::device::DeviceStatus::Disconnected),
            ConnectionStatus::Error(_) => Ok(fujinet_core::device::DeviceStatus::Error),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// Factory function to create the right device type based on URL
pub fn new_network_device(endpoint: String) -> DeviceResult<Box<dyn Device>> {
    let scheme = endpoint.split("://").next().ok_or(fujinet_core::error::DeviceError::InvalidProtocol)?;
    
    match scheme {
        "http" | "https" => {
            let protocol = HttpProtocol::default();
            let device = NetworkDevice::new(endpoint, protocol);
            Ok(Box::new(device))
        },
        _ => Err(fujinet_core::error::DeviceError::InvalidProtocol),
    }
}

// Helper function to create an HTTP device
pub fn new_http_device(url: String) -> DeviceResult<Box<dyn HttpProtocolHandler>> {
    let device = new_network_device(url.clone())?;
    if let Some(http) = device.as_any().downcast_ref::<HttpProtocol>() {
        let mut http_protocol = http.clone();
        // Set up the HTTP client
        http_protocol.set_http_client(Box::new(fujinet_core::platform::network::create_http_client()?));
        Ok(Box::new(http_protocol))
    } else {
        Err(fujinet_core::error::DeviceError::InvalidProtocol)
    }
}