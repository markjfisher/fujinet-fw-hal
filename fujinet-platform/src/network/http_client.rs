/// Creates a platform-specific HTTP client
pub fn create_http_client() -> fujinet_device::device::DeviceResult<Box<dyn fujinet_device::network::protocols::HttpClient>> {
    // For now, return a mock client
    #[cfg(test)]
    {
        use crate::tests::mocks::MockHttpClient;
        Ok(Box::new(MockHttpClient::new()))
    }
    #[cfg(not(test))]
    {
        Err(fujinet_device::device::DeviceError::NotSupported)
    }
} 