use async_trait::async_trait;
use std::collections::HashMap;
use reqwest;

use crate::device::DeviceResult;
use crate::device::network::protocols::{HttpClient, BaseHttpClient, HttpClientProvider};
use reqwest::Client as ReqwestClient;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

/// X86 platform-specific HTTP client implementation using reqwest
pub struct X86HttpClient {
    base: BaseHttpClient,
    client: ReqwestClient,
}

impl Default for X86HttpClient {
    fn default() -> Self {
        Self {
            base: BaseHttpClient::default(),
            client: ReqwestClient::new(),
        }
    }
}

#[async_trait]
impl HttpClient for X86HttpClient {
    async fn connect(&mut self, url: &str) -> DeviceResult<()> {
        let real_url = self.base.parse_network_url(url)?;
        let state = self.base.get_connection_state();
        state.url = real_url;
        Ok(())
    }

    async fn disconnect(&mut self) -> DeviceResult<()> {
        self.base.remove_current_connection();
        Ok(())
    }

    async fn get(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
        let real_url = self.base.parse_network_url(url)?;
        self.base.update_url_if_changed(real_url.clone());
        let state = self.base.get_connection_state();
        
        let mut request = self.client.get(&real_url);
        
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
        let real_url = self.base.parse_network_url(url)?;
        self.base.update_url_if_changed(real_url.clone());
        let state = self.base.get_connection_state();
        
        let mut request = self.client.post(&real_url).body(body.to_vec());
        
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
        let real_url = self.base.parse_network_url(url)?;
        self.base.update_url_if_changed(real_url.clone());
        let state = self.base.get_connection_state();
        
        let mut request = self.client.put(&real_url).body(body.to_vec());
        
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
        let real_url = self.base.parse_network_url(url)?;
        self.base.update_url_if_changed(real_url.clone());
        let state = self.base.get_connection_state();
        
        let mut request = self.client.delete(&real_url);
        
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

    async fn head(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
        let real_url = self.base.parse_network_url(url)?;
        self.base.update_url_if_changed(real_url.clone());
        let state = self.base.get_connection_state();
        
        let mut request = self.client.head(&real_url);
        
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

        Ok(Vec::new())
    }

    async fn patch(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        let real_url = self.base.parse_network_url(url)?;
        self.base.update_url_if_changed(real_url.clone());
        let state = self.base.get_connection_state();
        
        let mut request = self.client.patch(&real_url).body(body.to_vec());
        
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
        let state = self.base.get_connection_state();
        state.headers.insert(key.to_string(), value.to_string());
    }

    fn get_status_code(&self) -> u16 {
        self.base.get_current_state()
            .map(|state| state.status_code)
            .unwrap_or(200)
    }

    fn get_headers(&self) -> HashMap<String, String> {
        self.base.get_current_state()
            .map(|state| state.headers.clone())
            .unwrap_or_default()
    }

    fn get_network_unit(&self) -> u8 {
        self.base.get_network_unit()
    }
}

#[derive(Default)]
pub struct DefaultHttpClientProvider;

impl HttpClientProvider for DefaultHttpClientProvider {
    fn create_http_client(&self) -> Box<dyn HttpClient> {
        Box::new(X86HttpClient::default())
    }
} 