use async_trait::async_trait;
use crate::error::DeviceResult;
use std::borrow::Cow;

#[async_trait]
pub trait HostTranslator: Send + Sync {
    /// Initializes the host translator
    async fn initialize(&mut self) -> DeviceResult<()>;

    /// Processes incoming data from the host
    /// Returns either a reference to the processed data or a new vector if transformation is needed
    async fn process_host_data<'a>(&'a mut self, data: &'a [u8]) -> DeviceResult<Cow<'a, [u8]>>;

    /// Processes outgoing data to the host
    /// Returns either a reference to the processed data or a new vector if transformation is needed
    async fn process_device_data<'a>(&'a mut self, data: &'a [u8]) -> DeviceResult<Cow<'a, [u8]>>;
} 