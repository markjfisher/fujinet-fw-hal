use async_trait::async_trait;
use crate::device::DeviceResult;
use std::collections::HashMap;
use crate::device::network::NetworkUrl;

/// Base connection state that can be shared across implementations
#[derive(Clone, Default)]
pub struct HttpConnectionState {
    pub url: String,
    pub headers: HashMap<String, String>,
    pub status_code: u16,
}

/// Base HTTP client implementation that handles common functionality
pub struct BaseHttpClient {
    connections: HashMap<u8, HttpConnectionState>,
    current_unit: u8,
}

impl Default for BaseHttpClient {
    fn default() -> Self {
        Self {
            connections: HashMap::new(),
            current_unit: 1, // Default to N1
        }
    }
}

impl BaseHttpClient {
    /// Parse network unit ID from URL and return cleaned URL
    pub fn parse_network_url(&mut self, url: &str) -> DeviceResult<String> {
        let parsed = NetworkUrl::parse(url)?;
        self.current_unit = parsed.unit;
        Ok(parsed.url)
    }

    /// Get or create connection state for current network unit
    pub fn get_connection_state(&mut self) -> &mut HttpConnectionState {
        self.connections.entry(self.current_unit).or_insert_with(|| HttpConnectionState {
            url: String::new(),
            headers: HashMap::new(),
            status_code: 200,
        })
    }

    /// Get connection state for current network unit without creating if missing
    pub fn get_current_state(&self) -> Option<&HttpConnectionState> {
        self.connections.get(&self.current_unit)
    }

    /// Remove connection state for current network unit
    pub fn remove_current_connection(&mut self) {
        self.connections.remove(&self.current_unit);
    }

    /// Get current network unit
    pub fn get_network_unit(&self) -> u8 {
        self.current_unit
    }

    /// Update URL if changed
    pub fn update_url_if_changed(&mut self, url: String) {
        let state = self.get_connection_state();
        state.url = url;
    }
}

/// Platform-agnostic HTTP client interface
#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn connect(&mut self, endpoint: &str) -> DeviceResult<()>;
    async fn disconnect(&mut self) -> DeviceResult<()>;
    async fn get(&mut self, url: &str) -> DeviceResult<Vec<u8>>;
    async fn post(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>>;
    async fn put(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>>;
    async fn delete(&mut self, url: &str) -> DeviceResult<Vec<u8>>;
    async fn head(&mut self, url: &str) -> DeviceResult<Vec<u8>>;
    async fn patch(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>>;
    fn set_header(&mut self, key: &str, value: &str);
    fn get_status_code(&self) -> u16;
    fn get_headers(&self) -> HashMap<String, String>;
    fn get_network_unit(&self) -> u8;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let client = BaseHttpClient::default();
        assert_eq!(client.get_network_unit(), 1);
        assert!(client.get_current_state().is_none());
    }

    #[test]
    fn test_parse_network_url_default() {
        let mut client = BaseHttpClient::default();
        let result = client.parse_network_url("N:http://ficticious_example.madeup").unwrap();
        assert_eq!(result, "http://ficticious_example.madeup");
        assert_eq!(client.get_network_unit(), 1);
    }

    #[test]
    fn test_parse_network_url_with_unit() {
        let mut client = BaseHttpClient::default();
        let result = client.parse_network_url("N3:http://ficticious_example.madeup").unwrap();
        assert_eq!(result, "http://ficticious_example.madeup");
        assert_eq!(client.get_network_unit(), 3);
    }

    #[test]
    fn test_connection_state_management() {
        let mut client = BaseHttpClient::default();
        
        // Initially no state exists
        assert!(client.get_current_state().is_none());
        
        // Get or create creates new state
        let state = client.get_connection_state();
        assert_eq!(state.url, "");
        assert_eq!(state.status_code, 200);
        assert!(state.headers.is_empty());
        
        // State now exists
        assert!(client.get_current_state().is_some());
        
        // Remove state
        client.remove_current_connection();
        assert!(client.get_current_state().is_none());
    }

    #[test]
    fn test_multiple_units() {
        let mut client = BaseHttpClient::default();
        
        // Set up unit 1
        client.parse_network_url("N1:http://example1.com").unwrap();
        client.get_connection_state().url = "http://example1.com".to_string();
        
        // Set up unit 2
        client.parse_network_url("N2:http://example2.com").unwrap();
        client.get_connection_state().url = "http://example2.com".to_string();
        
        // Verify unit 1 state
        client.parse_network_url("N1:anything").unwrap();
        assert_eq!(client.get_connection_state().url, "http://example1.com");
        
        // Verify unit 2 state
        client.parse_network_url("N2:anything").unwrap();
        assert_eq!(client.get_connection_state().url, "http://example2.com");
    }
} 