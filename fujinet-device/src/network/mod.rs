mod protocols;

use fujinet_core::device::Device;
use fujinet_core::device::DeviceStatus;
use fujinet_core::error::DeviceResult;
use async_trait::async_trait;

pub use protocols::{Protocol, ProtocolHandler, ConnectionStatus};

pub struct NetworkDevice {
    endpoint: String,
    protocol: Box<dyn ProtocolHandler>,
}

impl NetworkDevice {
    pub fn new(endpoint: String, protocol: Box<dyn ProtocolHandler>) -> Self {
        Self {
            endpoint,
            protocol,
        }
    }

    // Network-specific methods
    pub async fn send_request(&mut self, method: String, url: String) -> DeviceResult<()> {
        self.protocol.send_request(method, url).await
    }
}

#[async_trait]
impl Device for NetworkDevice {
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

    async fn read_block(&mut self, block: u32, buf: &mut [u8]) -> DeviceResult<usize> {
        // For network devices, we'll treat blocks as direct reads
        self.protocol.read(buf).await
    }

    async fn write_block(&mut self, block: u32, buf: &[u8]) -> DeviceResult<usize> {
        // For network devices, we'll treat blocks as direct writes
        self.protocol.write(buf).await
    }

    async fn get_status(&self) -> DeviceResult<DeviceStatus> {
        match self.protocol.status().await? {
            ConnectionStatus::Connected => Ok(DeviceStatus::Ready),
            ConnectionStatus::Connecting => Ok(DeviceStatus::Busy),
            ConnectionStatus::Disconnected => Ok(DeviceStatus::Disconnected),
            ConnectionStatus::Error(e) => Err(e),
        }
    }
} 