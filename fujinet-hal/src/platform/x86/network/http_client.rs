use std::collections::HashMap;
use async_trait::async_trait;
use crate::device::DeviceResult;
use crate::device::network::protocols::HttpClient;
use reqwest::Client as ReqwestClient;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

/// X86 platform-specific HTTP client implementation using reqwest
pub struct X86HttpClient {
    client: ReqwestClient,
    headers: HashMap<String, String>,
    status_code: u16,
}

impl Default for X86HttpClient {
    fn default() -> Self {
        Self {
            client: ReqwestClient::new(),
            headers: HashMap::new(),
            status_code: 200,
        }
    }
}

#[async_trait]
impl HttpClient for X86HttpClient {
    async fn connect(&mut self, _url: &str) -> DeviceResult<()> {
        // No-op for reqwest as it manages connections automatically
        Ok(())
    }

    async fn disconnect(&mut self) -> DeviceResult<()> {
        // No-op for reqwest as it manages connections automatically
        Ok(())
    }

    async fn get(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
        let mut request = self.client.get(url);
        
        // Add headers
        let mut header_map = HeaderMap::new();
        for (key, value) in &self.headers {
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
        self.status_code = response.status().as_u16();

        // Get response body
        let body = response.bytes().await.map_err(|e| {
            crate::device::DeviceError::NetworkError(format!("Failed to read response body: {}", e))
        })?;

        Ok(body.to_vec())
    }

    async fn post(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        let mut request = self.client.post(url).body(body.to_vec());
        
        // Add headers
        let mut header_map = HeaderMap::new();
        for (key, value) in &self.headers {
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
        self.status_code = response.status().as_u16();

        // Get response body
        let body = response.bytes().await.map_err(|e| {
            crate::device::DeviceError::NetworkError(format!("Failed to read response body: {}", e))
        })?;

        Ok(body.to_vec())
    }

    async fn put(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        let mut request = self.client.put(url).body(body.to_vec());
        
        // Add headers
        let mut header_map = HeaderMap::new();
        for (key, value) in &self.headers {
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
        self.status_code = response.status().as_u16();

        // Get response body
        let body = response.bytes().await.map_err(|e| {
            crate::device::DeviceError::NetworkError(format!("Failed to read response body: {}", e))
        })?;

        Ok(body.to_vec())
    }

    async fn delete(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
        let mut request = self.client.delete(url);
        
        // Add headers
        let mut header_map = HeaderMap::new();
        for (key, value) in &self.headers {
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
        self.status_code = response.status().as_u16();

        // Get response body
        let body = response.bytes().await.map_err(|e| {
            crate::device::DeviceError::NetworkError(format!("Failed to read response body: {}", e))
        })?;

        Ok(body.to_vec())
    }

    async fn head(&mut self, url: &str) -> DeviceResult<()> {
        let mut request = self.client.head(url);
        
        // Add headers
        let mut header_map = HeaderMap::new();
        for (key, value) in &self.headers {
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
        self.status_code = response.status().as_u16();

        Ok(())
    }

    async fn patch(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        let mut request = self.client.patch(url).body(body.to_vec());
        
        // Add headers
        let mut header_map = HeaderMap::new();
        for (key, value) in &self.headers {
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
        self.status_code = response.status().as_u16();

        // Get response body
        let body = response.bytes().await.map_err(|e| {
            crate::device::DeviceError::NetworkError(format!("Failed to read response body: {}", e))
        })?;

        Ok(body.to_vec())
    }

    fn set_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    fn get_status_code(&self) -> u16 {
        self.status_code
    }

    fn get_headers(&self) -> HashMap<String, String> {
        self.headers.clone()
    }
} 