use std::collections::HashMap;
use crate::device::{DeviceError, DeviceResult};
use super::{ProtocolHandler, ConnectionStatus, HttpClient, client_provider::HttpClientProvider};
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

/// HTTP protocol handler implementation
pub struct HttpProtocol {
    client: Box<dyn HttpClient>,
    url: Option<String>,
}

impl HttpProtocol {
    pub fn new(client_provider: Arc<dyn HttpClientProvider>) -> Self {
        Self {
            client: client_provider.create_http_client(),
            url: None,
        }
    }

    /// Send an HTTP request
    pub async fn send_request(&mut self, method: &str, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        match method.to_uppercase().as_str() {
            "GET" => self.client.get(url).await,
            "POST" => self.client.post(url, body).await,
            "PUT" => self.client.put(url, body).await,
            "DELETE" => self.client.delete(url).await,
            "HEAD" => self.client.head(url).await,
            "PATCH" => self.client.patch(url, body).await,
            _ => Err(DeviceError::InvalidOperation),
        }
    }

    /// Perform HTTP GET request
    pub async fn get(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
        self.send_request("GET", url, &[]).await
    }
    
    /// Perform HTTP POST request
    pub async fn post(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        self.send_request("POST", url, body).await
    }
    
    /// Perform HTTP PUT request
    pub async fn put(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        self.send_request("PUT", url, body).await
    }
    
    /// Perform HTTP DELETE request
    pub async fn delete(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
        self.send_request("DELETE", url, &[]).await
    }
    
    /// Perform HTTP HEAD request
    pub async fn head(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
        self.send_request("HEAD", url, &[]).await
    }
    
    /// Perform HTTP PATCH request
    pub async fn patch(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
        self.send_request("PATCH", url, body).await
    }

    /// Set a header for subsequent requests
    pub fn set_header(&mut self, key: &str, value: &str) {
        self.client.set_header(key, value);
    }

    /// Get current status code from last request
    pub fn status_code(&self) -> u16 {
        self.client.status_code()
    }

    /// Get all current headers
    pub fn headers(&self) -> HashMap<String, String> {
        self.client.headers()
    }

    /// Get the base URL for this connection
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }
}

#[async_trait]
impl ProtocolHandler for HttpProtocol {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    async fn open(&mut self, url: &str) -> DeviceResult<()> {
        self.url = Some(url.to_string());
        self.client.connect(url).await
    }

    async fn close(&mut self) -> DeviceResult<()> {
        let result = self.client.disconnect().await;
        if result.is_ok() {
            self.url = None;
        }
        result
    }

    async fn read(&mut self, _buf: &mut [u8]) -> DeviceResult<usize> {
        Err(DeviceError::NotSupported)
    }

    async fn write(&mut self, _buf: &[u8]) -> DeviceResult<usize> {
        Err(DeviceError::NotSupported)
    }

    async fn status(&self) -> DeviceResult<ConnectionStatus> {
        Ok(if self.url.is_some() {
            ConnectionStatus::Connected
        } else {
            ConnectionStatus::Disconnected
        })
    }

    async fn available(&self) -> DeviceResult<usize> {
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default, Debug, PartialEq)]
    struct RequestRecord {
        method: String,
        url: String,
        body: Vec<u8>,
    }

    struct TestHttpClient {
        headers: HashMap<String, String>,
        recorded_requests: Arc<Mutex<Vec<RequestRecord>>>,
        is_connected: bool,
        endpoint: String,
    }

    impl Default for TestHttpClient {
        fn default() -> Self {
            Self {
                headers: HashMap::new(),
                recorded_requests: Arc::new(Mutex::new(Vec::new())),
                is_connected: false,
                endpoint: String::new(),
            }
        }
    }

    impl Clone for TestHttpClient {
        fn clone(&self) -> Self {
            Self {
                headers: self.headers.clone(),
                recorded_requests: self.recorded_requests.clone(),
                is_connected: self.is_connected,
                endpoint: self.endpoint.clone(),
            }
        }
    }

    #[async_trait]
    impl HttpClient for TestHttpClient {
        async fn connect(&mut self, url: &str) -> DeviceResult<()> {
            self.endpoint = url.to_string();
            self.is_connected = true;
            Ok(())
        }

        async fn disconnect(&mut self) -> DeviceResult<()> {
            self.is_connected = false;
            self.endpoint.clear();
            Ok(())
        }

        async fn get(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
            if !self.is_connected {
                return Err(DeviceError::NotReady);
            }
            self.recorded_requests.lock().unwrap().push(RequestRecord {
                method: "GET".to_string(),
                url: url.to_string(),
                body: Vec::new(),
            });
            Ok(b"test response".to_vec())
        }

        async fn post(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
            if !self.is_connected {
                return Err(DeviceError::NotReady);
            }
            self.recorded_requests.lock().unwrap().push(RequestRecord {
                method: "POST".to_string(),
                url: url.to_string(),
                body: body.to_vec(),
            });
            Ok(b"test response".to_vec())
        }

        async fn put(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
            if !self.is_connected {
                return Err(DeviceError::NotReady);
            }
            self.recorded_requests.lock().unwrap().push(RequestRecord {
                method: "PUT".to_string(),
                url: url.to_string(),
                body: body.to_vec(),
            });
            Ok(b"test response".to_vec())
        }

        async fn delete(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
            if !self.is_connected {
                return Err(DeviceError::NotReady);
            }
            self.recorded_requests.lock().unwrap().push(RequestRecord {
                method: "DELETE".to_string(),
                url: url.to_string(),
                body: Vec::new(),
            });
            Ok(Vec::new())
        }

        async fn head(&mut self, url: &str) -> DeviceResult<Vec<u8>> {
            if !self.is_connected {
                return Err(DeviceError::NotReady);
            }
            self.recorded_requests.lock().unwrap().push(RequestRecord {
                method: "HEAD".to_string(),
                url: url.to_string(),
                body: Vec::new(),
            });
            Ok(Vec::new())
        }

        async fn patch(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
            if !self.is_connected {
                return Err(DeviceError::NotReady);
            }
            self.recorded_requests.lock().unwrap().push(RequestRecord {
                method: "PATCH".to_string(),
                url: url.to_string(),
                body: body.to_vec(),
            });
            Ok(b"test response".to_vec())
        }

        fn set_header(&mut self, key: &str, value: &str) {
            self.headers.insert(key.to_string(), value.to_string());
        }

        fn status_code(&self) -> u16 {
            200
        }

        fn headers(&self) -> HashMap<String, String> {
            self.headers.clone()
        }
    }

    #[derive(Default)]
    struct TestHttpClientProvider {
        client: TestHttpClient,
    }

    impl HttpClientProvider for TestHttpClientProvider {
        fn create_http_client(&self) -> Box<dyn HttpClient> {
            Box::new(self.client.clone())
        }
    }

    #[tokio::test]
    async fn test_protocol_lifecycle() {
        let provider = Arc::new(TestHttpClientProvider::default());
        let mut protocol = HttpProtocol::new(provider);

        // Test initial state
        assert_eq!(protocol.status().await.unwrap(), ConnectionStatus::Disconnected);

        // Test connection
        protocol.open("http://test.com").await.unwrap();
        assert_eq!(protocol.status().await.unwrap(), ConnectionStatus::Connected);

        // Test disconnection
        protocol.close().await.unwrap();
        assert_eq!(protocol.status().await.unwrap(), ConnectionStatus::Disconnected);
    }

    #[tokio::test]
    async fn test_http_operations() {
        let provider = Arc::new(TestHttpClientProvider::default());
        let mut protocol = HttpProtocol::new(provider.clone());

        // Connect first
        protocol.open("http://test.com").await.unwrap();

        // Test GET request
        let response = protocol.get("http://test.com/api").await.unwrap();
        assert_eq!(response, b"test response");

        // Test POST request with body
        let post_data = b"test data";
        let response = protocol.post("http://test.com/api", post_data).await.unwrap();
        assert_eq!(response, b"test response");

        // Test PUT request with different body
        let put_data = b"updated data";
        let response = protocol.put("http://test.com/api", put_data).await.unwrap();
        assert_eq!(response, b"test response");

        // Test DELETE request
        let response = protocol.delete("http://test.com/api").await.unwrap();
        assert_eq!(response, Vec::<u8>::new());

        // Test HEAD request
        let response = protocol.head("http://test.com/api").await.unwrap();
        assert_eq!(response, Vec::<u8>::new());

        // Test PATCH request
        let patch_data = b"patch data";
        let response = protocol.patch("http://test.com/api", patch_data).await.unwrap();
        assert_eq!(response, b"test response");

        // Test headers
        protocol.set_header("Content-Type", "application/json");
        assert_eq!(protocol.headers().get("Content-Type").unwrap(), "application/json");

        // Verify recorded requests
        let client = provider.client.clone();
        let recorded_requests = client.recorded_requests.lock().unwrap();
        
        let expected_requests = vec![
            RequestRecord {
                method: "GET".to_string(),
                url: "http://test.com/api".to_string(),
                body: Vec::new(),
            },
            RequestRecord {
                method: "POST".to_string(),
                url: "http://test.com/api".to_string(),
                body: post_data.to_vec(),
            },
            RequestRecord {
                method: "PUT".to_string(),
                url: "http://test.com/api".to_string(),
                body: put_data.to_vec(),
            },
            RequestRecord {
                method: "DELETE".to_string(),
                url: "http://test.com/api".to_string(),
                body: Vec::new(),
            },
            RequestRecord {
                method: "HEAD".to_string(),
                url: "http://test.com/api".to_string(),
                body: Vec::new(),
            },
            RequestRecord {
                method: "PATCH".to_string(),
                url: "http://test.com/api".to_string(),
                body: patch_data.to_vec(),
            },
        ];

        assert_eq!(*recorded_requests, expected_requests, "Recorded requests don't match expected requests");
    }

    #[tokio::test]
    async fn test_error_handling() {
        let provider = Arc::new(TestHttpClientProvider::default());
        let mut protocol = HttpProtocol::new(provider);

        // Test operations without connecting first
        assert!(matches!(
            protocol.get("http://test.com/api").await,
            Err(DeviceError::NotReady)
        ));

        assert!(matches!(
            protocol.post("http://test.com/api", b"test").await,
            Err(DeviceError::NotReady)
        ));
    }
} 


