use async_trait::async_trait;
use crate::device::DeviceResult;
use std::any::Any;
use std::collections::HashMap;

#[async_trait]
pub trait HttpClient: Any + Send + Sync {
    async fn connect(&mut self, url: &str) -> DeviceResult<()>;
    async fn disconnect(&mut self) -> DeviceResult<()>;
    
    async fn get(&mut self, url: &str) -> DeviceResult<Vec<u8>>;
    async fn post(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>>;
    async fn put(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>>;
    async fn delete(&mut self, url: &str) -> DeviceResult<Vec<u8>>;
    async fn head(&mut self, url: &str) -> DeviceResult<()>;
    async fn patch(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>>;
    
    fn set_header(&mut self, key: &str, value: &str);
    fn get_status_code(&self) -> u16;
    fn get_headers(&self) -> HashMap<String, String>;

} 