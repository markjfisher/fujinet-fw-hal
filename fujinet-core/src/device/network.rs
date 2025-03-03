use async_trait::async_trait;
use crate::error::DeviceResult;
use super::Device;

#[async_trait]
pub trait NetworkDevice: Device {
    /// Connects to a network endpoint
    async fn connect(&mut self, endpoint: &str) -> DeviceResult<()>;

    /// Disconnects from the current endpoint
    async fn disconnect(&mut self) -> DeviceResult<()>;
} 