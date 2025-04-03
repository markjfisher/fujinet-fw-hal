use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use fujinet_hal::device::DeviceResult;
use fujinet_hal::device::network::protocols::{HttpClient, BaseHttpClient, HttpClientProvider};
use async_trait::async_trait;

/// Mock HTTP client for testing
pub struct MockHttpClient {
    base: BaseHttpClient,
    recorded_requests: Arc<Mutex<Vec<(String, Vec<u8>)>>>,
}

impl Default for MockHttpClient {
    fn default() -> Self {
        Self {
            base: BaseHttpClient::default(),
            recorded_requests: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Clone for MockHttpClient {
    fn clone(&self) -> Self {
        Self {
            base: BaseHttpClient::default(),
            recorded_requests: self.recorded_requests.clone(),
        }
    }
}

impl MockHttpClient {
    pub fn get_recorded_requests(&self) -> Arc<Mutex<Vec<(String, Vec<u8>)>>> {
        self.recorded_requests.clone()
    }
}

#[async_trait]
impl HttpClient for MockHttpClient {
    async fn connect(&mut self, _url: &str) -> DeviceResult<()> {
        Ok(())
    }

    async fn disconnect(&mut self) -> DeviceResult<()> {
        Ok(())
    }

    async fn get(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
        self.recorded_requests.lock().unwrap().push((url.to_string(), Vec::new()));
        Ok(b"test response".to_vec())
    }

    async fn post(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        self.recorded_requests.lock().unwrap().push((url.to_string(), body.to_vec()));
        Ok(b"test response".to_vec())
    }

    async fn put(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        self.recorded_requests.lock().unwrap().push((url.to_string(), body.to_vec()));
        Ok(b"test response".to_vec())
    }

    async fn delete(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
        self.recorded_requests.lock().unwrap().push((url.to_string(), Vec::new()));
        Ok(Vec::new())
    }

    async fn head(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
        self.recorded_requests.lock().unwrap().push((url.to_string(), Vec::new()));
        Ok(Vec::new())
    }

    async fn patch(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        self.recorded_requests.lock().unwrap().push((url.to_string(), body.to_vec()));
        Ok(b"test response".to_vec())
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

/// Mock HTTP client provider for testing
#[derive(Default)]
pub struct MockHttpClientProvider {
    mock_client: MockHttpClient,
}

impl HttpClientProvider for MockHttpClientProvider {
    fn create_http_client(&self) -> Box<dyn HttpClient> {
        Box::new(self.mock_client.clone())
    }
} 