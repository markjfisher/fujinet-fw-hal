use async_trait::async_trait;
use crate::error::DeviceResult;

#[async_trait]
pub trait HostTranslator: Send + Sync {
    /// Initializes the host translator
    async fn initialize(&mut self) -> DeviceResult<()>;

    /// Processes incoming data from the host
    async fn process_host_data(&mut self, data: &[u8]) -> DeviceResult<Vec<u8>>;

    /// Processes outgoing data to the host
    async fn process_device_data(&mut self, data: &[u8]) -> DeviceResult<Vec<u8>>;
} 