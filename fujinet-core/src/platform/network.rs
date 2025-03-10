use async_trait::async_trait;
use crate::error::DeviceResult;
use std::any::Any;
use std::collections::HashMap;

#[async_trait]
pub trait NetworkDriver: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    async fn connect(&mut self, endpoint: &str) -> DeviceResult<()>;
    async fn disconnect(&mut self) -> DeviceResult<()>;
    async fn write(&mut self, data: &[u8]) -> DeviceResult<usize>;
    async fn read(&mut self, buffer: &mut [u8]) -> DeviceResult<usize>;
}

#[async_trait]
pub trait HttpClient: Send + Sync + Any {
    async fn get(&mut self, url: &str) -> DeviceResult<Vec<u8>>;
    async fn post(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>>;
    async fn put(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>>;
    async fn delete(&mut self, url: &str) -> DeviceResult<Vec<u8>>;
    async fn head(&mut self, url: &str) -> DeviceResult<Vec<u8>>;
    async fn patch(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>>;
    
    async fn set_header(&mut self, key: &str, value: &str) -> DeviceResult<()>;
    async fn get_status_code(&self) -> DeviceResult<u16>;
    async fn get_headers(&self) -> DeviceResult<HashMap<String, String>>;
} 