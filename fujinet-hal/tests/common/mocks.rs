use fujinet_hal::device::network::protocols::{HttpClient, HttpClientProvider};
use fujinet_hal::device::DeviceResult;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;

/// Mock HTTP client provider for testing
/// This mock provides a full implementation that tracks request/response data
/// and connection state for verification in tests.
#[derive(Default, Clone)]
pub struct MockHttpClientProvider;

impl HttpClientProvider for MockHttpClientProvider {
    fn create_http_client(&self) -> Box<dyn HttpClient> {
        Box::new(MockHttpClient::default())
    }
}

#[derive(Debug, Clone)]
pub struct MockHttpClient {
    state: Arc<Mutex<MockHttpClientState>>,
}

#[derive(Debug, Default)]
struct MockHttpClientState {
    last_method: String,
    last_url: String,
    last_body: Vec<u8>,
    headers: HashMap<String, String>,
    status_code: u16,
    response_data: Vec<u8>,
    is_connected: bool,
    network_unit: u8,
}

impl Default for MockHttpClient {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(MockHttpClientState::default())),
        }
    }
}

/// Helper methods for test verification
pub trait MockHttpClientHelpers {
    fn get_last_request(&self) -> Option<(String, String, Vec<u8>)>;
    fn set_response_data(&self, data: &[u8]);
    fn set_status_code(&self, code: u16);
    fn is_connected(&self) -> bool;
}

impl MockHttpClientHelpers for MockHttpClient {
    fn get_last_request(&self) -> Option<(String, String, Vec<u8>)> {
        let state = self.state.lock().unwrap();
        if state.last_method.is_empty() {
            None
        } else {
            Some((
                state.last_method.clone(),
                state.last_url.clone(),
                state.last_body.clone(),
            ))
        }
    }

    fn set_response_data(&self, data: &[u8]) {
        let mut state = self.state.lock().unwrap();
        state.response_data = data.to_vec();
    }

    fn set_status_code(&self, code: u16) {
        let mut state = self.state.lock().unwrap();
        state.status_code = code;
    }

    fn is_connected(&self) -> bool {
        self.state.lock().unwrap().is_connected
    }
}

#[async_trait]
impl HttpClient for MockHttpClient {
    async fn connect(&mut self, url: &str) -> DeviceResult<()> {
        let mut state = self.state.lock().unwrap();
        state.is_connected = true;
        state.last_url = url.to_string();
        Ok(())
    }

    async fn disconnect(&mut self) -> DeviceResult<()> {
        let mut state = self.state.lock().unwrap();
        state.is_connected = false;
        Ok(())
    }

    async fn get(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
        let mut state = self.state.lock().unwrap();
        state.last_method = "GET".to_string();
        state.last_url = url.to_string();
        state.last_body.clear();
        Ok(state.response_data.clone())
    }

    async fn post(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        let mut state = self.state.lock().unwrap();
        state.last_method = "POST".to_string();
        state.last_url = url.to_string();
        state.last_body = body.to_vec();
        Ok(state.response_data.clone())
    }

    async fn put(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        let mut state = self.state.lock().unwrap();
        state.last_method = "PUT".to_string();
        state.last_url = url.to_string();
        state.last_body = body.to_vec();
        Ok(state.response_data.clone())
    }

    async fn delete(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
        let mut state = self.state.lock().unwrap();
        state.last_method = "DELETE".to_string();
        state.last_url = url.to_string();
        state.last_body.clear();
        Ok(state.response_data.clone())
    }

    async fn head(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
        let mut state = self.state.lock().unwrap();
        state.last_method = "HEAD".to_string();
        state.last_url = url.to_string();
        Ok(Vec::new())  // HEAD requests return empty body
    }

    async fn patch(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        let mut state = self.state.lock().unwrap();
        state.last_method = "PATCH".to_string();
        state.last_url = url.to_string();
        state.last_body = body.to_vec();
        Ok(state.response_data.clone())
    }

    fn set_header(&mut self, key: &str, value: &str) {
        let mut state = self.state.lock().unwrap();
        state.headers.insert(key.to_string(), value.to_string());
    }

    fn get_status_code(&self) -> u16 {
        let state = self.state.lock().unwrap();
        state.status_code
    }

    fn get_headers(&self) -> HashMap<String, String> {
        let state = self.state.lock().unwrap();
        state.headers.clone()
    }

    fn get_network_unit(&self) -> u8 {
        let state = self.state.lock().unwrap();
        state.network_unit
    }
} 