#![cfg(test)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use fujinet_hal::device::DeviceResult;
use fujinet_hal::device::network::protocols::HttpClient;

#[derive(Debug, Clone)]
pub struct TestHttpClient {
    state: Arc<Mutex<TestHttpClientState>>,
}

#[derive(Debug)]
struct TestHttpClientState {
    last_method: String,
    last_url: String,
    last_body: Vec<u8>,
    headers: HashMap<String, String>,
    status_code: u16,
}

impl Default for TestHttpClient {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(TestHttpClientState {
                last_method: String::new(),
                last_url: String::new(),
                last_body: Vec::new(),
                headers: HashMap::new(),
                status_code: 200,
            })),
        }
    }
}

pub trait TestHttpClientHelpers {
    fn get_last_request(&self) -> Option<(String, String, Vec<u8>)>;
    fn get_test_headers(&self) -> Option<HashMap<String, String>>;
}

impl TestHttpClientHelpers for TestHttpClient {
    fn get_last_request(&self) -> Option<(String, String, Vec<u8>)> {
        let state = self.state.lock().unwrap();
        Some((state.last_method.clone(), state.last_url.clone(), state.last_body.clone()))
    }

    fn get_test_headers(&self) -> Option<HashMap<String, String>> {
        let state = self.state.lock().unwrap();
        Some(state.headers.clone())
    }
}

#[async_trait]
impl HttpClient for TestHttpClient {
    async fn connect(&mut self, url: &str) -> DeviceResult<()> {
        let mut state = self.state.lock().unwrap();
        state.last_method = "CONNECT".to_string();
        state.last_url = url.to_string();
        Ok(())
    }

    async fn disconnect(&mut self) -> DeviceResult<()> {
        let mut state = self.state.lock().unwrap();
        state.last_method = "DISCONNECT".to_string();
        Ok(())
    }

    async fn get(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
        let mut state = self.state.lock().unwrap();
        state.last_method = "GET".to_string();
        state.last_url = url.to_string();
        Ok(vec![])
    }

    async fn post(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        let mut state = self.state.lock().unwrap();
        state.last_method = "POST".to_string();
        state.last_url = url.to_string();
        state.last_body = body.to_vec();
        Ok(vec![])
    }

    async fn put(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        let mut state = self.state.lock().unwrap();
        state.last_method = "PUT".to_string();
        state.last_url = url.to_string();
        state.last_body = body.to_vec();
        Ok(vec![])
    }

    async fn delete(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
        let mut state = self.state.lock().unwrap();
        state.last_method = "DELETE".to_string();
        state.last_url = url.to_string();
        state.last_body.clear();
        Ok(vec![])
    }

    async fn head(&mut self, url: &str) -> DeviceResult<()> {
        let mut state = self.state.lock().unwrap();
        state.last_method = "HEAD".to_string();
        state.last_url = url.to_string();
        Ok(())
    }

    async fn patch(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        let mut state = self.state.lock().unwrap();
        state.last_method = "PATCH".to_string();
        state.last_url = url.to_string();
        state.last_body = body.to_vec();
        Ok(vec![])
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
}

