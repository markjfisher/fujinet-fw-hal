use std::collections::HashMap;
use fujinet_core::error::{DeviceError, DeviceResult};
use super::{ProtocolHandler, ConnectionStatus};
use async_trait::async_trait;
use fujinet_core::platform::network::HttpClient;

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

impl Default for HttpProtocol {
    fn default() -> Self {
        Self {
            endpoint: String::new(),
            status: ConnectionStatus::Disconnected,
            http_client: None,
            response_buffer: Vec::new(),
            response_pos: 0,
        }
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

impl HttpProtocol {
    pub fn set_http_client(&mut self, client: Box<dyn HttpClient>) {
        self.http_client = Some(client);
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
            client.set_header(key, value).await
        } else {
            Err(DeviceError::NotReady)
        }
    }

    async fn get_status_code(&self) -> DeviceResult<u16> {
        if let Some(client) = &self.http_client {
            client.get_status_code().await
        } else {
            Err(DeviceError::NotReady)
        }
    }

    async fn get_headers(&self) -> DeviceResult<HashMap<String, String>> {
        if let Some(client) = &self.http_client {
            client.get_headers().await
        } else {
            Err(DeviceError::NotReady)
        }
    }
}

#[async_trait]
impl ProtocolHandler for HttpProtocol {
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