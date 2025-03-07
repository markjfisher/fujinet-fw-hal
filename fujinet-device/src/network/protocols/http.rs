use std::collections::HashMap;
use fujinet_core::error::{DeviceError, DeviceResult};
use super::{ProtocolHandler, ConnectionStatus};
use async_trait::async_trait;
use fujinet_core::platform::network::NetworkDriver;

#[async_trait]
pub trait HttpProtocolHandler: ProtocolHandler {
    /// Send an HTTP request
    async fn send_request(&mut self, method: String, url: String) -> DeviceResult<()>;
    
    /// Add a header to the current request
    async fn add_header(&mut self, key: String, value: String) -> DeviceResult<()>;
    
    /// Get the status code of the last response
    async fn get_status_code(&self) -> DeviceResult<Option<u16>>;
    
    /// Get the headers of the last response
    async fn get_headers(&self) -> DeviceResult<HashMap<String, String>>;
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
    network_driver: Option<Box<dyn NetworkDriver>>,
    request: Option<HttpRequest>,
    response: Option<HttpResponse>,
}

impl Default for HttpProtocol {
    fn default() -> Self {
        Self {
            endpoint: String::new(),
            status: ConnectionStatus::Disconnected,
            network_driver: None,
            request: None,
            response: None,
        }
    }
}

impl HttpProtocol {
    pub fn set_network_driver(&mut self, driver: Box<dyn NetworkDriver>) {
        self.network_driver = Some(driver);
    }
}

#[async_trait]
impl HttpProtocolHandler for HttpProtocol {
    async fn send_request(&mut self, method: String, url: String) -> DeviceResult<()> {
        self.request = Some(HttpRequest {
            method,
            url,
            headers: HashMap::new(),
            body: Vec::new(),
        });
        // This will be implemented by the platform layer
        // For now, we'll simulate a successful request
        self.response = Some(HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: Vec::new(),
        });
        Ok(())
    }

    async fn add_header(&mut self, key: String, value: String) -> DeviceResult<()> {
        if let Some(request) = &mut self.request {
            request.headers.insert(key, value);
            Ok(())
        } else {
            Err(DeviceError::NotReady)
        }
    }

    async fn get_status_code(&self) -> DeviceResult<Option<u16>> {
        Ok(self.response.as_ref().map(|r| r.status_code))
    }

    async fn get_headers(&self) -> DeviceResult<HashMap<String, String>> {
        Ok(self.response.as_ref().map_or(HashMap::new(), |r| r.headers.clone()))
    }
}

#[async_trait]
impl ProtocolHandler for HttpProtocol {
    fn set_network_driver(&mut self, driver: Box<dyn NetworkDriver>) {
        self.network_driver = Some(driver);
    }

    fn get_network_driver(&mut self) -> Option<&mut dyn NetworkDriver> {
        self.network_driver.as_mut().map(|d| d.as_mut())
    }

    async fn open(&mut self, endpoint: &str) -> DeviceResult<()> {
        self.endpoint = endpoint.to_string();
        self.status = ConnectionStatus::Connecting;
        
        if self.network_driver.is_none() {
            return Err(DeviceError::NotReady);
        }
        
        if let Some(driver) = &mut self.network_driver {
            driver.connect(endpoint).await?;
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
        
        if let Some(conn) = &mut self.network_driver {
            conn.disconnect().await?;
            self.status = ConnectionStatus::Disconnected;
            Ok(())
        } else {
            Err(DeviceError::NotReady)
        }
    }

    async fn write(&mut self, buf: &[u8]) -> DeviceResult<usize> {
        if let Some(conn) = &mut self.network_driver {
            conn.write(buf).await
        } else {
            Err(DeviceError::NotReady)
        }
    }

    async fn read(&mut self, buf: &mut [u8]) -> DeviceResult<usize> {
        if let Some(conn) = &mut self.network_driver {
            conn.read(buf).await
        } else {
            Err(DeviceError::NotReady)
        }
    }

    async fn status(&self) -> DeviceResult<ConnectionStatus> {
        Ok(self.status.clone())
    }

    async fn available(&self) -> DeviceResult<usize> {
        // This would need platform support to check available bytes
        Ok(0)
    }

}