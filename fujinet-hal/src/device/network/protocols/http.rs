use std::collections::HashMap;
use crate::device::{DeviceError, DeviceResult};
use super::{ProtocolHandler, ConnectionStatus, HttpClient, client_provider::HttpClientProvider};
use async_trait::async_trait;

#[async_trait]
pub trait HttpProtocolHandler: ProtocolHandler + std::any::Any {
    /// Send an HTTP request
    async fn send_request(&mut self, method: &str, url: &str, body: &[u8]) -> DeviceResult<()>;
    
    /// Add a header to the current request
    async fn add_header(&mut self, key: &str, value: &str) -> DeviceResult<()>;
    
    /// Get the status code of the last response
    async fn get_status_code(&self) -> DeviceResult<u16>;
    
    /// Get the headers of the last response
    async fn get_headers(&self) -> DeviceResult<HashMap<String, String>>;
    
    /// Convenience methods
    async fn get(&mut self, url: &str) -> DeviceResult<()> {
        self.send_request("GET", url, &[]).await
    }
    
    async fn post(&mut self, url: &str, body: &[u8]) -> DeviceResult<()> {
        self.send_request("POST", url, body).await
    }
    
    async fn put(&mut self, url: &str, body: &[u8]) -> DeviceResult<()> {
        self.send_request("PUT", url, body).await
    }
    
    async fn delete(&mut self, url: &str) -> DeviceResult<()> {
        self.send_request("DELETE", url, &[]).await
    }
    
    async fn head(&mut self, url: &str) -> DeviceResult<()> {
        self.send_request("HEAD", url, &[]).await
    }
    
    async fn patch(&mut self, url: &str, body: &[u8]) -> DeviceResult<()> {
        self.send_request("PATCH", url, body).await
    }
}

#[derive(Debug)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

pub struct HttpProtocol {
    endpoint: String,
    status: ConnectionStatus,
    http_client: Option<Box<dyn HttpClient>>,
    response_buffer: Vec<u8>,
    response_pos: usize,
}

impl HttpProtocol {
    pub fn new(client_provider: &dyn HttpClientProvider) -> Self {
        Self {
            endpoint: String::new(),
            status: ConnectionStatus::Disconnected,
            http_client: Some(client_provider.create_http_client()),
            response_buffer: Vec::new(),
            response_pos: 0,
        }
    }

    pub fn new_without_client() -> Self {
        Self {
            endpoint: String::new(),
            status: ConnectionStatus::Disconnected,
            http_client: None,
            response_buffer: Vec::new(),
            response_pos: 0,
        }
    }

    pub fn set_http_client(&mut self, client: Box<dyn HttpClient>) {
        self.http_client = Some(client);
    }
}

impl Clone for HttpProtocol {
    fn clone(&self) -> Self {
        Self {
            endpoint: self.endpoint.clone(),
            status: self.status.clone(),
            http_client: None, // Don't clone the client
            response_buffer: Vec::new(),
            response_pos: 0,
        }
    }
}

#[async_trait]
impl HttpProtocolHandler for HttpProtocol {
    async fn send_request(&mut self, method: &str, url: &str, body: &[u8]) -> DeviceResult<()> {
        if let Some(client) = &mut self.http_client {
            match method.to_uppercase().as_str() {
                "GET" => { client.get(url).await?; }
                "POST" => { client.post(url, body).await?; }
                "PUT" => { client.put(url, body).await?; }
                "DELETE" => { client.delete(url).await?; }
                "HEAD" => { client.head(url).await?; }
                "PATCH" => { client.patch(url, body).await?; }
                _ => return Err(DeviceError::InvalidOperation),
            };
            Ok(())
        } else {
            Err(DeviceError::NotReady)
        }
    }

    async fn add_header(&mut self, key: &str, value: &str) -> DeviceResult<()> {
        if let Some(client) = &mut self.http_client {
            client.set_header(key, value);
            Ok(())
        } else {
            Err(DeviceError::NotReady)
        }
    }

    async fn get_status_code(&self) -> DeviceResult<u16> {
        if let Some(client) = &self.http_client {
            Ok(client.get_status_code())
        } else {
            Err(DeviceError::NotReady)
        }
    }

    async fn get_headers(&self) -> DeviceResult<HashMap<String, String>> {
        if let Some(client) = &self.http_client {
            Ok(client.get_headers())
        } else {
            Err(DeviceError::NotReady)
        }
    }
}

#[async_trait]
impl ProtocolHandler for HttpProtocol {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    async fn open(&mut self, endpoint: &str) -> DeviceResult<()> {
        self.endpoint = endpoint.to_string();
        self.status = ConnectionStatus::Connecting;
        
        if let Some(client) = &mut self.http_client {
            client.connect(endpoint).await?;
            self.status = ConnectionStatus::Connected;
            Ok(())
        } else {
            Err(DeviceError::NotReady)
        }
    }

    async fn close(&mut self) -> DeviceResult<()> {
        if self.status == ConnectionStatus::Disconnected {
            return Err(DeviceError::NotReady);
        }
        
        if let Some(client) = &mut self.http_client {
            client.disconnect().await?;
            self.status = ConnectionStatus::Disconnected;
            Ok(())
        } else {
            Err(DeviceError::NotReady)
        }
    }

    async fn write(&mut self, buf: &[u8]) -> DeviceResult<usize> {
        if self.status != ConnectionStatus::Connected {
            return Err(DeviceError::NotReady);
        }

        if let Some(client) = &mut self.http_client {
            // For protocol-agnostic usage, treat writes as POST requests
            let response = client.post(&self.endpoint, buf).await?;
            self.response_buffer = response;
            self.response_pos = 0;
            Ok(buf.len())
        } else {
            Err(DeviceError::NotReady)
        }
    }

    async fn read(&mut self, buf: &mut [u8]) -> DeviceResult<usize> {
        if self.status != ConnectionStatus::Connected {
            return Err(DeviceError::NotReady);
        }

        if let Some(client) = &mut self.http_client {
            // If no response data available, do a GET request
            if self.response_buffer.is_empty() {
                let response = client.get(&self.endpoint).await?;
                self.response_buffer = response;
                self.response_pos = 0;
            }

            // Read from response buffer
            if self.response_pos >= self.response_buffer.len() {
                return Ok(0); // EOF
            }

            let remaining = self.response_buffer.len() - self.response_pos;
            let to_read = std::cmp::min(remaining, buf.len());
            
            buf[..to_read].copy_from_slice(&self.response_buffer[self.response_pos..self.response_pos + to_read]);
            self.response_pos += to_read;
            
            Ok(to_read)
        } else {
            Err(DeviceError::NotReady)
        }
    }

    async fn status(&self) -> DeviceResult<ConnectionStatus> {
        Ok(self.status.clone())
    }

    async fn available(&self) -> DeviceResult<usize> {
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::DeviceError;
    use crate::device::network::protocols::HttpClient;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Mock HTTP client for testing
    #[derive(Clone)]
    struct TestHttpClient {
        state: Arc<Mutex<TestHttpClientState>>,
    }

    #[derive(Default)]
    struct TestHttpClientState {
        last_method: String,
        last_url: String,
        last_body: Vec<u8>,
        headers: HashMap<String, String>,
        response_data: Vec<u8>,
        is_connected: bool,
    }

    #[async_trait]
    impl HttpClient for TestHttpClient {
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
            Ok(state.response_data.clone())
        }

        async fn head(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
            let mut state = self.state.lock().unwrap();
            state.last_method = "HEAD".to_string();
            state.last_url = url.to_string();
            Ok(Vec::new())  // HEAD returns empty body
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

        fn get_headers(&self) -> HashMap<String, String> {
            self.state.lock().unwrap().headers.clone()
        }

        fn get_status_code(&self) -> u16 { 200 }
        fn get_network_unit(&self) -> u8 { 1 }
    }

    impl Default for TestHttpClient {
        fn default() -> Self {
            Self {
                state: Arc::new(Mutex::new(TestHttpClientState::default())),
            }
        }
    }

    // Helper methods for test verification
    impl TestHttpClient {
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

        fn is_connected(&self) -> bool {
            self.state.lock().unwrap().is_connected
        }
    }

    #[tokio::test]
    async fn test_protocol_lifecycle() {
        let mut protocol = HttpProtocol::new_without_client();
        assert_eq!(protocol.status().await.unwrap(), ConnectionStatus::Disconnected);
        
        // Test operations without client should fail
        assert!(matches!(protocol.open("test").await, Err(DeviceError::NotReady)));
        assert!(matches!(protocol.close().await, Err(DeviceError::NotReady)));
        assert!(matches!(protocol.read(&mut [0; 10]).await, Err(DeviceError::NotReady)));
        assert!(matches!(protocol.write(&[0; 10]).await, Err(DeviceError::NotReady)));

        // Set client and test connection lifecycle
        let client = TestHttpClient::default();
        protocol.set_http_client(Box::new(client.clone()));
        
        protocol.open("http://test.com").await.unwrap();
        assert!(client.is_connected());
        
        protocol.close().await.unwrap();
        assert!(!client.is_connected());
    }

    #[tokio::test]
    async fn test_protocol_io() -> DeviceResult<()> {
        let mut protocol = HttpProtocol::new_without_client();
        let client = TestHttpClient::default();
        protocol.set_http_client(Box::new(client.clone()));

        // Setup test data
        let test_url = "http://test.com";
        client.set_response_data(b"test response");
        protocol.open(test_url).await?;

        // Test write (POST)
        let test_data = b"test request";
        protocol.write(test_data).await?;
        let (method, url, body) = client.get_last_request().unwrap();
        assert_eq!(method, "POST");
        assert_eq!(url, test_url);
        assert_eq!(body, test_data);

        // Test read (should trigger GET)
        let mut buf = vec![0; 100];
        let read = protocol.read(&mut buf).await?;
        assert_eq!(&buf[..read], b"test response");

        Ok(())
    }

    #[tokio::test]
    async fn test_protocol_headers() -> DeviceResult<()> {
        let mut protocol = HttpProtocol::new_without_client();
        let client = TestHttpClient::default();
        protocol.set_http_client(Box::new(client.clone()));
        protocol.open("http://test.com").await?;

        // Add headers
        protocol.add_header("Content-Type", "application/json").await?;
        protocol.add_header("Authorization", "Bearer token").await?;

        // Verify headers were set
        let headers = protocol.get_headers().await?;
        assert_eq!(headers.get("Content-Type").unwrap(), "application/json");
        assert_eq!(headers.get("Authorization").unwrap(), "Bearer token");

        // Verify headers are sent with request
        protocol.get("http://test.com").await?;
        let headers = client.get_headers();
        assert_eq!(headers.get("Content-Type").unwrap(), "application/json");
        assert_eq!(headers.get("Authorization").unwrap(), "Bearer token");

        Ok(())
    }
} 