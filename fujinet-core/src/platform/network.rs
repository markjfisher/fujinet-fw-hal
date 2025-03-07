use async_trait::async_trait;
use crate::error::DeviceResult;
use std::any::Any;

#[async_trait]
pub trait NetworkDriver: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    async fn connect(&mut self, endpoint: &str) -> DeviceResult<()>;
    async fn disconnect(&mut self) -> DeviceResult<()>;
    async fn write(&mut self, data: &[u8]) -> DeviceResult<usize>;
    async fn read(&mut self, buffer: &mut [u8]) -> DeviceResult<usize>;
} 