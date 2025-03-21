use fujinet_hal::device::network::NetworkUrl;
use fujinet_hal::device::network::protocols::HttpClient;
use fujinet_hal::device::DeviceResult;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;

/// Mock HTTP client for testing
#[derive(Debug, Clone)]
pub struct MockHttpClient {
    state: Arc<Mutex<MockHttpClientState>>,
}

#[derive(Debug)]
struct MockHttpClientState {
    last_url: String,
    last_post_data: Vec<u8>,
    headers: HashMap<String, String>,
    status_code: u16,
    network_unit: u8,
    is_connected: bool,
}

impl Default for MockHttpClient {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(MockHttpClientState {
                last_url: String::new(),
                last_post_data: Vec::new(),
                headers: HashMap::new(),
                status_code: 200,
                network_unit: 1,
                is_connected: false,
            })),
        }
    }
}

impl MockHttpClient {
    pub fn _new() -> Self {
        Self::default()
    }

    pub fn _get_last_post(&self) -> Option<(String, Vec<u8>)> {
        let state = self.state.lock().unwrap();
        if state.last_url.is_empty() {
            None
        } else {
            Some((state.last_url.clone(), state.last_post_data.clone()))
        }
    }
}

#[async_trait]
impl HttpClient for MockHttpClient {
    async fn connect(&mut self, url: &str) -> DeviceResult<()> {
        let mut state = self.state.lock().unwrap();
        state.is_connected = true;
        // Parse the URL to strip N: prefix if present
        if let Ok(network_url) = NetworkUrl::parse(url) {
            state.last_url = network_url.url;
        } else {
            state.last_url = url.to_string();
        }
        Ok(())
    }

    async fn disconnect(&mut self) -> DeviceResult<()> {
        let mut state = self.state.lock().unwrap();
        state.is_connected = false;
        Ok(())
    }

    async fn get(&mut self, _url: &str) -> DeviceResult<Vec<u8>> {
        Ok(vec![])
    }

    async fn post(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        let mut state = self.state.lock().unwrap();
        // Parse the URL to strip N: prefix if present
        if let Ok(network_url) = NetworkUrl::parse(url) {
            state.last_url = network_url.url;
        } else {
            state.last_url = url.to_string();
        }
        state.last_post_data = body.to_vec();
        Ok(vec![])
    }

    async fn put(&mut self, _url: &str, _body: &[u8]) -> DeviceResult<Vec<u8>> {
        Ok(vec![])
    }

    async fn delete(&mut self, _url: &str) -> DeviceResult<Vec<u8>> {
        Ok(vec![])
    }

    async fn head(&mut self, _url: &str) -> DeviceResult<()> {
        Ok(())
    }

    async fn patch(&mut self, _url: &str, _body: &[u8]) -> DeviceResult<Vec<u8>> {
        Ok(vec![])
    }

    fn set_header(&mut self, key: &str, value: &str) {
        let mut state = self.state.lock().unwrap();
        state.headers.insert(key.to_string(), value.to_string());
    }

    fn get_status_code(&self) -> u16 {
        self.state.lock().unwrap().status_code
    }

    fn get_headers(&self) -> HashMap<String, String> {
        self.state.lock().unwrap().headers.clone()
    }

    fn get_network_unit(&self) -> u8 {
        self.state.lock().unwrap().network_unit
    }
} 