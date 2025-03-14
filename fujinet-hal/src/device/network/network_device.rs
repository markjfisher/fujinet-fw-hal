use async_trait::async_trait;
use crate::device::{Device, DeviceResult, DeviceError, DeviceStatus};
use std::any::Any;
use super::protocols::{ProtocolHandler, ConnectionStatus};
use super::protocols::http::HttpProtocol;
use super::url::NetworkUrl;

#[async_trait]
pub trait NetworkDevice: Device {
    /// Connects to a network endpoint
    async fn connect(&mut self, endpoint: &str) -> DeviceResult<()>;

    /// Disconnects from the current endpoint
    async fn disconnect(&mut self) -> DeviceResult<()>;
}

pub struct NetworkDeviceImpl<P: ProtocolHandler + 'static> {
    endpoint: String,
    protocol: P,
}

impl<P: ProtocolHandler> NetworkDeviceImpl<P> {
    pub fn new(endpoint: String, protocol: P) -> Self {
        Self {
            endpoint,
            protocol,
        }
    }

    pub fn protocol(&self) -> &P {
        &self.protocol
    }

    pub fn protocol_mut(&mut self) -> &mut P {
        &mut self.protocol
    }
}

#[async_trait]
impl<P: ProtocolHandler> Device for NetworkDeviceImpl<P> {
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
        Err(DeviceError::NotSupported)
    }

    async fn write_block(&mut self, _block: u32, _buf: &[u8]) -> DeviceResult<usize> {
        Err(DeviceError::NotSupported)
    }

    async fn get_status(&self) -> DeviceResult<DeviceStatus> {
        match self.protocol.status().await? {
            ConnectionStatus::Connected => Ok(DeviceStatus::Ready),
            ConnectionStatus::Disconnected => Ok(DeviceStatus::Disconnected),
            ConnectionStatus::Connecting => Ok(DeviceStatus::Disconnected),
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

// Factory function to create the right device type based on URL
pub fn new_network_device(endpoint: String) -> DeviceResult<Box<dyn Device>> {
    let parsed = NetworkUrl::parse(&endpoint)?;
    let scheme = parsed.scheme()?;
    
    match scheme {
        "http" | "https" => {
            let protocol = HttpProtocol::default();
            // The protocol will handle the network unit internally
            let device = NetworkDeviceImpl::new(endpoint, protocol);
            Ok(Box::new(device))
        },
        _ => Err(DeviceError::InvalidProtocol),
    }
} 