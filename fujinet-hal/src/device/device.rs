use async_trait::async_trait;
use std::any::Any;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceStatus {
    Ready,
    Error,
    Disconnected,
}

/// Result type for device operations
pub type DeviceResult<T> = Result<T, DeviceError>;

/// Error type for device operations
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceError {
    NotSupported,
    NotReady,
    InvalidProtocol,
    InvalidOperation,
    IoError(String),
    NetworkError(String),
}

impl From<std::io::Error> for DeviceError {
    fn from(err: std::io::Error) -> Self {
        DeviceError::IoError(err.to_string())
    }
}

#[async_trait]
pub trait Device: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Returns the name of the device
    fn name(&self) -> &str;

    /// Opens the device
    async fn open(&mut self) -> DeviceResult<()>;

    /// Closes the device
    async fn close(&mut self) -> DeviceResult<()>;

    /// Reads bytes from the device
    async fn read_bytes(&mut self, buf: &mut [u8]) -> DeviceResult<usize>;

    /// Writes bytes to the device
    async fn write_bytes(&mut self, buf: &[u8]) -> DeviceResult<usize>;

    /// Reads a block of data from the device
    async fn read_block(&mut self, block: u32, buf: &mut [u8]) -> DeviceResult<usize>;

    /// Writes a block of data to the device
    async fn write_block(&mut self, block: u32, buf: &[u8]) -> DeviceResult<usize>;

    /// Gets the current status of the device
    async fn get_status(&self) -> DeviceResult<DeviceStatus>;
} 