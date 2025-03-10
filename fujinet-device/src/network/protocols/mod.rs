pub mod http;

use fujinet_core::error::{DeviceError, DeviceResult};
use async_trait::async_trait;
use std::any::Any;

#[async_trait]
pub trait ProtocolHandler: Send + Sync + Default {
    /// Open a connection to the endpoint
    async fn open(&mut self, endpoint: &str) -> DeviceResult<()>;
    
    /// Close the connection
    async fn close(&mut self) -> DeviceResult<()>;
    
    /// Read data from the connection
    /// Returns the number of bytes read
    async fn read(&mut self, buf: &mut [u8]) -> DeviceResult<usize>;
    
    /// Write data to the connection
    /// Returns the number of bytes written
    async fn write(&mut self, buf: &[u8]) -> DeviceResult<usize>;
    
    /// Get the current status of the connection
    async fn status(&self) -> DeviceResult<ConnectionStatus>;
    
    /// Get the number of bytes available to read
    async fn available(&self) -> DeviceResult<usize>;
}

pub trait AnyProtocolHandler: ProtocolHandler + Any {}

#[derive(Debug, PartialEq, Clone)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(DeviceError),
}
