pub mod protocols;

use fujinet_core::error::DeviceResult;
use fujinet_core::device::Device;
use async_trait::async_trait;

pub use protocols::{Protocol, ProtocolHandler, ConnectionStatus, AnyProtocolHandler};
pub use protocols::http::{HttpProtocol, HttpProtocolHandler};

pub struct NetworkDevice<P: AnyProtocolHandler> {
    endpoint: String,
    protocol: P,
}

impl<P: AnyProtocolHandler> NetworkDevice<P> {
    pub fn new(endpoint: String, protocol: P) -> Self {
        Self {
            endpoint,
            protocol,
        }
    }
}

#[async_trait]
impl<P: AnyProtocolHandler> Device for NetworkDevice<P> {
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
        // For network devices, we don't support block-based reading yet
        Err(fujinet_core::error::DeviceError::NotSupported)
    }

    async fn write_block(&mut self, _block: u32, _buf: &[u8]) -> DeviceResult<usize> {
        // For network devices, we don't support block-based writing yet
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
}

impl NetworkDevice<HttpProtocol> {
    // Protocol-specific methods
    pub async fn send_request(&mut self, method: String, url: String) -> DeviceResult<()> {
        self.protocol.send_request(method, url).await
    }

    pub async fn add_header(&mut self, key: String, value: String) -> DeviceResult<()> {
        self.protocol.add_header(key, value).await
    }

    pub async fn get_status_code(&self) -> DeviceResult<Option<u16>> {
        self.protocol.get_status_code().await
    }

    pub async fn get_headers(&self) -> DeviceResult<std::collections::HashMap<String, String>> {
        self.protocol.get_headers().await
    }
}