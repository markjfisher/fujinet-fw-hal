use std::collections::HashMap;
use async_trait::async_trait;
use crate::device::DeviceResult;
use crate::device::network::protocols::HttpClient;
use reqwest::Client as ReqwestClient;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use crate::device::DeviceError;

#[derive(Clone)]
struct ConnectionState {
    url: String,
    client: ReqwestClient,
    headers: HashMap<String, String>,
    status_code: u16,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self {
            url: String::new(),
            client: ReqwestClient::new(),
            headers: HashMap::new(),
            status_code: 200,
        }
    }
}

/// X86 platform-specific HTTP client implementation using reqwest
pub struct X86HttpClient {
    // Map of network unit -> connection state
    connections: HashMap<u8, ConnectionState>,
    current_unit: u8,
}

impl Default for X86HttpClient {
    fn default() -> Self {
        Self {
            connections: HashMap::new(),
            current_unit: 1, // Default to N1
        }
    }
}

impl X86HttpClient {
    /// Parse network unit ID from URL and return cleaned URL
    fn parse_network_url(&mut self, url: &str) -> DeviceResult<String> {
        // Check if URL starts with N: or Nx: where x is 1-8
        if let Some(rest) = url.strip_prefix("N:") {
            self.current_unit = 1;
            Ok(rest.to_string())
        } else if url.starts_with('N') && url.len() >= 3 && url.chars().nth(1).unwrap().is_ascii_digit() && url.chars().nth(2) == Some(':') {
            let unit = url.chars().nth(1).unwrap().to_digit(10).unwrap() as u8;
            if unit == 0 || unit > 8 {
                return Err(DeviceError::InvalidProtocol);
            }
            self.current_unit = unit;
            Ok(url[3..].to_string())
        } else {
            Err(DeviceError::InvalidProtocol)
        }
    }

    /// Get or create connection state for current network unit
    fn get_connection_state(&mut self) -> &mut ConnectionState {
        self.connections.entry(self.current_unit).or_default()
    }
}

#[async_trait]
impl HttpClient for X86HttpClient {
    async fn connect(&mut self, url: &str) -> DeviceResult<()> {
        let real_url = self.parse_network_url(url)?;
        let state = self.get_connection_state();
        state.url = real_url;
        Ok(())
    }

    async fn disconnect(&mut self) -> DeviceResult<()> {
        // Remove the connection state for current unit
        self.connections.remove(&self.current_unit);
        Ok(())
    }

    async fn get(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
        let real_url = self.parse_network_url(url)?;
        let state = self.get_connection_state();
        
        // If URL changed, update it
        if !state.url.is_empty() && state.url != real_url {
            state.url = real_url.clone();
        }
        
        let mut request = state.client.get(&real_url);
        
        // Add headers
        let mut header_map = HeaderMap::new();
        for (key, value) in &state.headers {
            if let (Ok(name), Ok(val)) = (HeaderName::from_bytes(key.as_bytes()), 
                                        HeaderValue::from_str(value)) {
                header_map.insert(name, val);
            }
        }
        request = request.headers(header_map);

        // Send request
        let response = request.send().await.map_err(|e| {
            crate::device::DeviceError::NetworkError(format!("HTTP GET failed: {}", e))
        })?;

        // Store status code
        state.status_code = response.status().as_u16();

        // Get response body
        let body = response.bytes().await.map_err(|e| {
            crate::device::DeviceError::NetworkError(format!("Failed to read response body: {}", e))
        })?;

        Ok(body.to_vec())
    }

    async fn post(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        let real_url = self.parse_network_url(url)?;
        let state = self.get_connection_state();
        
        // If URL changed, update it
        if !state.url.is_empty() && state.url != real_url {
            state.url = real_url.clone();
        }
        
        let mut request = state.client.post(&real_url).body(body.to_vec());
        
        // Add headers
        let mut header_map = HeaderMap::new();
        for (key, value) in &state.headers {
            if let (Ok(name), Ok(val)) = (HeaderName::from_bytes(key.as_bytes()), 
                                        HeaderValue::from_str(value)) {
                header_map.insert(name, val);
            }
        }
        request = request.headers(header_map);

        // Send request
        let response = request.send().await.map_err(|e| {
            crate::device::DeviceError::NetworkError(format!("HTTP POST failed: {}", e))
        })?;

        // Store status code
        state.status_code = response.status().as_u16();

        // Get response body
        let body = response.bytes().await.map_err(|e| {
            crate::device::DeviceError::NetworkError(format!("Failed to read response body: {}", e))
        })?;

        Ok(body.to_vec())
    }

    async fn put(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        let real_url = self.parse_network_url(url)?;
        let state = self.get_connection_state();
        
        // If URL changed, update it
        if !state.url.is_empty() && state.url != real_url {
            state.url = real_url.clone();
        }
        
        let mut request = state.client.put(&real_url).body(body.to_vec());
        
        // Add headers
        let mut header_map = HeaderMap::new();
        for (key, value) in &state.headers {
            if let (Ok(name), Ok(val)) = (HeaderName::from_bytes(key.as_bytes()), 
                                        HeaderValue::from_str(value)) {
                header_map.insert(name, val);
            }
        }
        request = request.headers(header_map);

        // Send request
        let response = request.send().await.map_err(|e| {
            crate::device::DeviceError::NetworkError(format!("HTTP PUT failed: {}", e))
        })?;

        // Store status code
        state.status_code = response.status().as_u16();

        // Get response body
        let body = response.bytes().await.map_err(|e| {
            crate::device::DeviceError::NetworkError(format!("Failed to read response body: {}", e))
        })?;

        Ok(body.to_vec())
    }

    async fn delete(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
        let real_url = self.parse_network_url(url)?;
        let state = self.get_connection_state();
        
        // If URL changed, update it
        if !state.url.is_empty() && state.url != real_url {
            state.url = real_url.clone();
        }
        
        let mut request = state.client.delete(&real_url);
        
        // Add headers
        let mut header_map = HeaderMap::new();
        for (key, value) in &state.headers {
            if let (Ok(name), Ok(val)) = (HeaderName::from_bytes(key.as_bytes()), 
                                        HeaderValue::from_str(value)) {
                header_map.insert(name, val);
            }
        }
        request = request.headers(header_map);

        // Send request
        let response = request.send().await.map_err(|e| {
            crate::device::DeviceError::NetworkError(format!("HTTP DELETE failed: {}", e))
        })?;

        // Store status code
        state.status_code = response.status().as_u16();

        // Get response body
        let body = response.bytes().await.map_err(|e| {
            crate::device::DeviceError::NetworkError(format!("Failed to read response body: {}", e))
        })?;

        Ok(body.to_vec())
    }

    async fn head(&mut self, url: &str) -> DeviceResult<()> {
        let real_url = self.parse_network_url(url)?;
        let state = self.get_connection_state();
        
        // If URL changed, update it
        if !state.url.is_empty() && state.url != real_url {
            state.url = real_url.clone();
        }
        
        let mut request = state.client.head(&real_url);
        
        // Add headers
        let mut header_map = HeaderMap::new();
        for (key, value) in &state.headers {
            if let (Ok(name), Ok(val)) = (HeaderName::from_bytes(key.as_bytes()), 
                                        HeaderValue::from_str(value)) {
                header_map.insert(name, val);
            }
        }
        request = request.headers(header_map);

        // Send request
        let response = request.send().await.map_err(|e| {
            crate::device::DeviceError::NetworkError(format!("HTTP HEAD failed: {}", e))
        })?;

        // Store status code
        state.status_code = response.status().as_u16();

        Ok(())
    }

    async fn patch(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        let real_url = self.parse_network_url(url)?;
        let state = self.get_connection_state();
        
        // If URL changed, update it
        if !state.url.is_empty() && state.url != real_url {
            state.url = real_url.clone();
        }
        
        let mut request = state.client.patch(&real_url).body(body.to_vec());
        
        // Add headers
        let mut header_map = HeaderMap::new();
        for (key, value) in &state.headers {
            if let (Ok(name), Ok(val)) = (HeaderName::from_bytes(key.as_bytes()), 
                                        HeaderValue::from_str(value)) {
                header_map.insert(name, val);
            }
        }
        request = request.headers(header_map);

        // Send request
        let response = request.send().await.map_err(|e| {
            crate::device::DeviceError::NetworkError(format!("HTTP PATCH failed: {}", e))
        })?;

        // Store status code
        state.status_code = response.status().as_u16();

        // Get response body
        let body = response.bytes().await.map_err(|e| {
            crate::device::DeviceError::NetworkError(format!("Failed to read response body: {}", e))
        })?;

        Ok(body.to_vec())
    }

    fn set_header(&mut self, key: &str, value: &str) {
        let state = self.get_connection_state();
        state.headers.insert(key.to_string(), value.to_string());
    }

    fn get_status_code(&self) -> u16 {
        self.connections.get(&self.current_unit)
            .map(|state| state.status_code)
            .unwrap_or(200)
    }

    fn get_headers(&self) -> HashMap<String, String> {
        self.connections.get(&self.current_unit)
            .map(|state| state.headers.clone())
            .unwrap_or_default()
    }

    fn get_network_unit(&self) -> u8 {
        self.current_unit
    }
} 