use async_trait::async_trait;
use std::collections::HashMap;
use reqwest;

use crate::device::{DeviceResult, DeviceError};
use crate::device::network::protocols::{HttpClient, BaseHttpClient, HttpClientProvider};

/// Platform-specific HTTP client implementation for x86
pub struct X86HttpClient {
    base: BaseHttpClient,
    client: reqwest::Client,
}

impl Default for X86HttpClient {
    fn default() -> Self {
        Self {
            base: BaseHttpClient::default(),
            client: reqwest::Client::new(),
        }
    }
}

impl From<reqwest::Error> for DeviceError {
    fn from(err: reqwest::Error) -> Self {
        DeviceError::NetworkError(err.to_string())
    }
}

#[async_trait]
impl HttpClient for X86HttpClient {
    async fn connect(&mut self, _endpoint: &str) -> DeviceResult<()> {
        // reqwest client doesn't need explicit connection
        Ok(())
    }

    async fn disconnect(&mut self) -> DeviceResult<()> {
        // reqwest client doesn't need explicit disconnection
        Ok(())
    }

    async fn get(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
        let mut request = self.client.get(url);
        
        // Add headers
        for (key, value) in self.base.headers() {
            request = request.header(key, value);
        }

        let response = request.send().await?;
        self.base.set_status_code(response.status().as_u16());
        Ok(response.bytes().await?.to_vec())
    }

    async fn post(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        let mut request = self.client.post(url).body(body.to_vec());
        
        // Add headers
        for (key, value) in self.base.headers() {
            request = request.header(key, value);
        }

        let response = request.send().await?;
        self.base.set_status_code(response.status().as_u16());
        Ok(response.bytes().await?.to_vec())
    }

    async fn put(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        let mut request = self.client.put(url).body(body.to_vec());
        
        // Add headers
        for (key, value) in self.base.headers() {
            request = request.header(key, value);
        }

        let response = request.send().await?;
        self.base.set_status_code(response.status().as_u16());
        Ok(response.bytes().await?.to_vec())
    }

    async fn delete(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
        let mut request = self.client.delete(url);
        
        // Add headers
        for (key, value) in self.base.headers() {
            request = request.header(key, value);
        }

        let response = request.send().await?;
        self.base.set_status_code(response.status().as_u16());
        Ok(response.bytes().await?.to_vec())
    }

    async fn head(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
        let mut request = self.client.head(url);
        
        // Add headers
        for (key, value) in self.base.headers() {
            request = request.header(key, value);
        }

        let response = request.send().await?;
        self.base.set_status_code(response.status().as_u16());
        Ok(response.bytes().await?.to_vec())
    }

    async fn patch(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        let mut request = self.client.patch(url).body(body.to_vec());
        
        // Add headers
        for (key, value) in self.base.headers() {
            request = request.header(key, value);
        }

        let response = request.send().await?;
        self.base.set_status_code(response.status().as_u16());
        Ok(response.bytes().await?.to_vec())
    }

    fn set_header(&mut self, key: &str, value: &str) {
        self.base.set_header(key.to_string(), value.to_string());
    }

    fn status_code(&self) -> u16 {
        self.base.status_code()
    }

    fn headers(&self) -> HashMap<String, String> {
        self.base.headers().clone()
    }
}

/// Default HTTP client provider for x86 platform
pub struct DefaultHttpClientProvider;

impl HttpClientProvider for DefaultHttpClientProvider {
    fn create_http_client(&self) -> Box<dyn HttpClient> {
        Box::new(X86HttpClient::default())
    }
} 