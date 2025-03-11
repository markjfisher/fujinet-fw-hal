use async_trait::async_trait;
use fujinet_core::device::Device;
use fujinet_core::error::{DeviceError, DeviceResult};
use fujinet_core::platform::network::HttpClient;
use fujinet_device::network::{new_network_device, HttpProtocol, NetworkDevice};
use std::collections::HashMap;
use std::any::Any;

// Modify create_network_device to return both
pub fn create_network_device(endpoint: String) -> DeviceResult<Box<dyn Device>> {
    let device = new_network_device(endpoint)?;
    if let Some(network_device) = device.as_any().downcast_ref::<NetworkDevice<HttpProtocol>>() {
        let mut http_protocol = network_device.protocol.clone();
        http_protocol.set_http_client(Box::new(MockHttpClient::new()));
        Ok(Box::new(NetworkDevice::new(endpoint, http_protocol)))
    } else {
        Ok(device)
    }
}

pub struct MockHttpClient {
    connected: bool,
    headers: HashMap<String, String>,
    status_code: u16,
    response_data: Vec<u8>,
}

impl MockHttpClient {
    pub fn new() -> Self {
        Self {
            connected: false,
            headers: HashMap::new(),
            status_code: 200,
            response_data: Vec::new(),
        }
    }

    // Test helper methods
    pub fn set_response(&mut self, status: u16, data: Vec<u8>) {
        self.status_code = status;
        self.response_data = data;
    }
}

#[async_trait]
impl HttpClient for MockHttpClient {
    async fn connect(&mut self, _endpoint: &str) -> DeviceResult<()> {
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> DeviceResult<()> {
        self.connected = false;
        Ok(())
    }

    async fn get(&mut self, _url: &str) -> DeviceResult<Vec<u8>> {
        if !self.connected {
            return Err(DeviceError::NotReady);
        }
        Ok(self.response_data.clone())
    }

    async fn post(&mut self, _url: &str, _body: &[u8]) -> DeviceResult<Vec<u8>> {
        if !self.connected {
            return Err(DeviceError::NotReady);
        }
        Ok(self.response_data.clone())
    }

    async fn put(&mut self, _url: &str, _body: &[u8]) -> DeviceResult<Vec<u8>> {
        if !self.connected {
            return Err(DeviceError::NotReady);
        }
        Ok(self.response_data.clone())
    }

    async fn delete(&mut self, _url: &str) -> DeviceResult<Vec<u8>> {
        if !self.connected {
            return Err(DeviceError::NotReady);
        }
        Ok(self.response_data.clone())
    }

    async fn head(&mut self, _url: &str) -> DeviceResult<Vec<u8>> {
        if !self.connected {
            return Err(DeviceError::NotReady);
        }
        Ok(self.response_data.clone())
    }

    async fn patch(&mut self, _url: &str, _body: &[u8]) -> DeviceResult<Vec<u8>> {
        if !self.connected {
            return Err(DeviceError::NotReady);
        }
        Ok(self.response_data.clone())
    }

    async fn set_header(&mut self, key: &str, value: &str) -> DeviceResult<()> {
        self.headers.insert(key.to_string(), value.to_string());
        Ok(())
    }

    async fn get_status_code(&self) -> DeviceResult<u16> {
        Ok(self.status_code)
    }

    async fn get_headers(&self) -> DeviceResult<HashMap<String, String>> {
        Ok(self.headers.clone())
    }
}
