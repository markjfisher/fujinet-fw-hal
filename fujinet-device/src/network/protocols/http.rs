use std::collections::HashMap;
use fujinet_core::error::{DeviceError, DeviceResult};
use super::{ProtocolHandler, ConnectionStatus};
use async_trait::async_trait;

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
    request: Option<HttpRequest>,
    response: Option<HttpResponse>,
    read_pos: usize,
    status: ConnectionStatus,
}

impl HttpProtocol {
    pub fn new() -> Self {
        Self {
            endpoint: String::new(),
            request: None,
            response: None,
            read_pos: 0,
            status: ConnectionStatus::Disconnected,
        }
    }

    pub fn set_request(&mut self, method: String, url: String) {
        self.request = Some(HttpRequest {
            method,
            url,
            headers: HashMap::new(),
            body: Vec::new(),
        });
    }

    pub fn add_header(&mut self, key: String, value: String) {
        if let Some(req) = &mut self.request {
            req.headers.insert(key, value);
        }
    }

    pub fn set_body(&mut self, body: Vec<u8>) {
        if let Some(req) = &mut self.request {
            req.body = body;
        }
    }

    pub fn get_status_code(&self) -> Option<u16> {
        self.response.as_ref().map(|r| r.status_code)
    }

    pub fn get_headers(&self) -> HashMap<String, String> {
        self.response.as_ref().map_or(HashMap::new(), |r| r.headers.clone())
    }
}

#[async_trait]
impl ProtocolHandler for HttpProtocol {
    async fn open(&mut self, endpoint: &str) -> DeviceResult<()> {
        self.endpoint = endpoint.to_string();
        self.status = ConnectionStatus::Connecting;
        
        // This will be implemented by the platform layer
        // For now, we'll simulate a successful connection
        self.status = ConnectionStatus::Connected;
        Ok(())
    }

    async fn close(&mut self) -> DeviceResult<()> {
        if self.status != ConnectionStatus::Connected {
            return Err(DeviceError::NotReady);
        }
        self.status = ConnectionStatus::Disconnected;
        self.request = None;
        self.response = None;
        self.read_pos = 0;
        Ok(())
    }

    async fn read(&mut self, buf: &mut [u8]) -> DeviceResult<usize> {
        if self.status != ConnectionStatus::Connected {
            return Err(DeviceError::NotReady);
        }

        if let Some(response) = &self.response {
            let available = response.body.len() - self.read_pos;
            let to_read = available.min(buf.len());
            
            if to_read > 0 {
                buf[..to_read].copy_from_slice(&response.body[self.read_pos..self.read_pos + to_read]);
                self.read_pos += to_read;
                Ok(to_read)
            } else {
                Ok(0)
            }
        } else {
            Ok(0)
        }
    }

    async fn write(&mut self, buf: &[u8]) -> DeviceResult<usize> {
        println!("Protocol status: {:?}", self.status);
        if self.status != ConnectionStatus::Connected {
            return Err(DeviceError::NotReady);
        }

        if let Some(request) = &mut self.request {
            request.body.extend_from_slice(buf);
            Ok(buf.len())
        } else {
            Err(DeviceError::NotReady)
        }
    }

    async fn status(&self) -> DeviceResult<ConnectionStatus> {
        Ok(self.status.clone())
    }

    async fn available(&self) -> DeviceResult<usize> {
        if let Some(response) = &self.response {
            Ok(response.body.len() - self.read_pos)
        } else {
            Ok(0)
        }
    }

    async fn send_request(&mut self, method: String, url: String) -> DeviceResult<()> {
        self.set_request(method, url);
        // This will be implemented by the platform layer
        // For now, we'll simulate a successful request
        self.response = Some(HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: Vec::new(),
        });
        Ok(())
    }
} 