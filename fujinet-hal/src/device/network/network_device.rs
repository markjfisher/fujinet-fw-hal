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