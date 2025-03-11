use crate::device::network::protocols::HttpClient;
use crate::device::{DeviceError, DeviceResult};

/// Creates a platform-specific HTTP client
pub fn create_http_client() -> DeviceResult<Box<dyn HttpClient>> {
    // For now, return NotSupported
    // TODO: Implement platform-specific HTTP client
    Err(DeviceError::NotSupported)
} 