use std::collections::HashMap;
use fujinet_core::error::{DeviceError, DeviceResult};
use super::{ProtocolHandler, ConnectionStatus};
use async_trait::async_trait;
use fujinet_core::platform::network::{NetworkDriver, HttpClient};

#[async_trait]
pub trait HttpProtocolHandler: ProtocolHandler {
    /// Send an HTTP request
    async fn send_request(&mut self, method: &str, url: &str, body: &[u8]) -> DeviceResult<()>;
    
    /// Add a header to the current request
    async fn add_header(&mut self, key: &str, value: &str) -> DeviceResult<()>;
    
    /// Get the status code of the last response
    async fn get_status_code(&self) -> DeviceResult<u16>;
    
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
    http_client: Option<Box<dyn HttpClient>>,
}

impl Default for HttpProtocol {
    fn default() -> Self {
        Self {
            endpoint: String::new(),
            status: ConnectionStatus::Disconnected,
            network_driver: None,
            http_client: None,
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
    fn set_network_driver(&mut self, driver: Box<dyn NetworkDriver>) {
        // TODO: Determine how platforms will provide their HTTP clients
        self.network_driver = Some(driver);
    }

    fn get_network_driver(&mut self) -> Option<&mut dyn NetworkDriver> {
        self.network_driver.as_mut().map(|d| d.as_mut())
    }

    async fn open(&mut self, endpoint: &str) -> DeviceResult<()> {
        self.endpoint = endpoint.to_string();
        self.status = ConnectionStatus::Connecting;
        
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
        
        if let Some(driver) = &mut self.network_driver {
            driver.disconnect().await?;
            self.status = ConnectionStatus::Disconnected;
            Ok(())
        } else {
            Err(DeviceError::NotReady)
        }
    }

    async fn write(&mut self, buf: &[u8]) -> DeviceResult<usize> {
        if let Some(driver) = &mut self.network_driver {
            driver.write(buf).await
        } else {
            Err(DeviceError::NotReady)
        }
    }

    async fn read(&mut self, buf: &mut [u8]) -> DeviceResult<usize> {
        if let Some(driver) = &mut self.network_driver {
            driver.read(buf).await
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