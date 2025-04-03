use async_trait::async_trait;
use crate::device::DeviceResult;
use std::collections::HashMap;

/// HTTP connection state
#[derive(Clone)]
pub struct HttpState {
    pub headers: HashMap<String, String>,
    pub status_code: u16,
}

impl Default for HttpState {
    fn default() -> Self {
        Self {
            headers: HashMap::new(),
            status_code: 200, // Set default status code to 200 (OK)
        }
    }
}

/// Base HTTP client implementation that handles common functionality
pub struct BaseHttpClient {
    state: HttpState,
}

impl Default for BaseHttpClient {
    fn default() -> Self {
        Self {
            state: HttpState::default(),
        }
    }
}

impl BaseHttpClient {
    /// Get current state
    pub fn state(&self) -> &HttpState {
        &self.state
    }

    /// Get mutable state
    pub fn state_mut(&mut self) -> &mut HttpState {
        &mut self.state
    }

    /// Set a header for subsequent requests
    pub fn set_header(&mut self, key: String, value: String) {
        self.state.headers.insert(key, value);
    }

    /// Get all current headers
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.state.headers
    }

    /// Get current status code
    pub fn status_code(&self) -> u16 {
        self.state.status_code
    }

    /// Update status code (typically after a request)
    pub fn set_status_code(&mut self, code: u16) {
        self.state.status_code = code;
    }
}

/// Platform-agnostic HTTP client interface
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// Connect to an endpoint
    async fn connect(&mut self, endpoint: &str) -> DeviceResult<()>;
    
    /// Disconnect from current endpoint
    async fn disconnect(&mut self) -> DeviceResult<()>;
    
    /// Perform HTTP GET request
    async fn get(&mut self, url: &str) -> DeviceResult<Vec<u8>>;
    
    /// Perform HTTP POST request
    async fn post(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>>;
    
    /// Perform HTTP PUT request
    async fn put(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>>;
    
    /// Perform HTTP DELETE request
    async fn delete(&mut self, url: &str) -> DeviceResult<Vec<u8>>;
    
    /// Perform HTTP HEAD request
    async fn head(&mut self, url: &str) -> DeviceResult<Vec<u8>>;
    
    /// Perform HTTP PATCH request
    async fn patch(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>>;
    
    /// Set a header for subsequent requests
    fn set_header(&mut self, key: &str, value: &str);
    
    /// Get current status code from last request
    fn status_code(&self) -> u16;
    
    /// Get all current headers
    fn headers(&self) -> HashMap<String, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let client = BaseHttpClient::default();
        assert_eq!(client.status_code(), 200);
        assert!(client.headers().is_empty());
    }

    #[test]
    fn test_header_management() {
        let mut client = BaseHttpClient::default();
        
        // Set and verify headers
        client.set_header("Content-Type".to_string(), "application/json".to_string());
        client.set_header("Authorization".to_string(), "Bearer token".to_string());
        
        assert_eq!(client.headers().len(), 2);
        assert_eq!(
            client.headers().get("Content-Type").unwrap(),
            "application/json"
        );
        assert_eq!(
            client.headers().get("Authorization").unwrap(),
            "Bearer token"
        );
    }

    #[test]
    fn test_status_code() {
        let mut client = BaseHttpClient::default();
        assert_eq!(client.status_code(), 200); // Default
        
        client.set_status_code(404);
        assert_eq!(client.status_code(), 404);
        
        client.set_status_code(500);
        assert_eq!(client.status_code(), 500);
    }
} 