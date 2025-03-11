use async_trait::async_trait;
use crate::device::{Device, DeviceResult};

#[async_trait]
pub trait Platform: Send + Sync {
    /// Initializes the platform
    async fn initialize(&mut self) -> DeviceResult<()>;

    /// Shuts down the platform
    async fn shutdown(&mut self) -> DeviceResult<()>;

    /// Creates a new device instance
    async fn create_device(&self, device_type: &str) -> DeviceResult<Box<dyn Device>>;
} 