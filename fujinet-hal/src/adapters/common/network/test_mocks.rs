use crate::device::DeviceResult;
use crate::device::DeviceError;
use crate::device::network::NetworkUrl;
use crate::device::manager::DeviceState;
use crate::device::network::NetworkDevice;
use crate::device::network::protocols::{ProtocolHandler, HttpClient, HttpProtocol, HttpClientProvider};
use crate::device::network::manager::NetworkManager;
use crate::device::{Device, DeviceStatus};
use async_trait::async_trait;
use std::collections::HashMap;
use std::any::Any;
use std::sync::Arc;

// Mock HTTP client for testing
#[derive(Clone)]
pub struct MockHttpClient {
    pub post_result: Result<(), DeviceError>,
    pub get_result: Result<Vec<u8>, DeviceError>,
    headers: HashMap<String, String>,
}

impl Default for MockHttpClient {
    fn default() -> Self {
        Self {
            post_result: Ok(()),
            get_result: Ok(vec![]),
            headers: HashMap::new(),
        }
    }
}

#[async_trait]
impl HttpClient for MockHttpClient {
    async fn connect(&mut self, _url: &str) -> DeviceResult<()> {
        Ok(())
    }

    async fn disconnect(&mut self) -> DeviceResult<()> {
        Ok(())
    }

    fn set_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    async fn get(&mut self, _url: &str) -> DeviceResult<Vec<u8>> {
        self.get_result.clone()
    }

    async fn post(&mut self, _url: &str, _body: &[u8]) -> DeviceResult<Vec<u8>> {
        self.post_result.clone().map(|_| vec![])
    }

    async fn put(&mut self, _url: &str, _body: &[u8]) -> DeviceResult<Vec<u8>> {
        Ok(vec![])
    }

    async fn delete(&mut self, _url: &str) -> DeviceResult<Vec<u8>> {
        Ok(vec![])
    }

    async fn head(&mut self, _url: &str) -> DeviceResult<Vec<u8>> {
        Ok(vec![])
    }

    async fn patch(&mut self, _url: &str, _body: &[u8]) -> DeviceResult<Vec<u8>> {
        Ok(vec![])
    }

    fn status_code(&self) -> u16 {
        200
    }

    fn headers(&self) -> HashMap<String, String> {
        self.headers.clone()
    }
}

pub struct TestNetworkManager {
    parse_result: Option<(usize, NetworkUrl)>,
    open_result: bool,
    close_result: bool,
    device: Option<Box<dyn NetworkDevice>>,
    device_states: HashMap<usize, DeviceState>,
}

#[async_trait]
impl NetworkManager for TestNetworkManager {
    fn parse_device_spec(&self, spec: &str) -> DeviceResult<(usize, NetworkUrl)> {
        // If we have a parse_result, check if the spec matches
        if let Some((device_id, url)) = &self.parse_result {
            // Parse the incoming spec to compare
            if let Ok(parsed_url) = NetworkUrl::parse(spec) {
                if parsed_url.url == url.url {
                    return Ok((*device_id, url.clone()));
                }
            }
        }
        // Return InvalidUrl for any non-matching spec
        Err(DeviceError::InvalidUrl)
    }

    async fn open_device(&mut self, _spec: &str, _mode: u8, _trans: u8) -> DeviceResult<()> {
        if self.open_result {
            Ok(())
        } else {
            Err(DeviceError::InvalidUrl)
        }
    }

    async fn find_device(&mut self, _spec: &str) -> DeviceResult<Option<(usize, &mut DeviceState)>> {
        Ok(None)
    }

    fn get_device(&mut self, device_id: usize) -> Option<&mut DeviceState> {
        self.device_states.get_mut(&device_id)
    }

    async fn close_device(&mut self, _device_id: usize) -> DeviceResult<bool> {
        Ok(self.close_result)
    }

    fn get_network_device(&mut self, _device_id: usize) -> Option<&mut Box<dyn NetworkDevice>> {
        self.device.as_mut()
    }
}

impl TestNetworkManager {
    pub fn new() -> Self {
        Self {
            parse_result: None,
            open_result: false,
            close_result: false,
            device: None,
            device_states: HashMap::new(),
        }
    }

    pub fn with_parse_result(mut self, device_id: usize, url: &str) -> Self {
        self.parse_result = Some((device_id, NetworkUrl::parse(url).unwrap()));
        self
    }

    pub fn with_device_state(mut self, device_id: usize, url: NetworkUrl) -> Self {
        let mut state = DeviceState::default();
        state.url = Some(url);
        self.device_states.insert(device_id, state);
        self
    }

    pub fn with_device_state_no_url(mut self, device_id: usize) -> Self {
        let state = DeviceState::default();
        self.device_states.insert(device_id, state);
        self
    }

    pub fn with_http_device(mut self, post_result: Result<(), DeviceError>) -> Self {
        let client = MockHttpClient {
            post_result,
            ..Default::default()
        };
        let provider = Arc::new(MockHttpClientProvider::new(client));
        let protocol = HttpProtocol::new(provider);
        let device = Box::new(MockNetworkDevice {
            protocol: Box::new(protocol),
        });
        self.device = Some(device);
        self
    }

    pub fn with_http_device_get(mut self, get_result: Result<Vec<u8>, DeviceError>) -> Self {
        let client = MockHttpClient {
            get_result,
            ..Default::default()
        };
        let provider = Arc::new(MockHttpClientProvider::new(client));
        let protocol = HttpProtocol::new(provider);
        let device = Box::new(MockNetworkDevice {
            protocol: Box::new(protocol),
        });
        self.device = Some(device);
        self
    }

    pub fn with_open_result(mut self, open_result: bool) -> Self {
        self.open_result = open_result;
        self
    }

    pub fn with_close_result(mut self, close_result: bool) -> Self {
        self.close_result = close_result;
        self
    }
}

// Mock network device
pub struct MockNetworkDevice {
    protocol: Box<dyn ProtocolHandler>,
}

#[async_trait]
impl Device for MockNetworkDevice {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn name(&self) -> &str {
        "mock_network_device"
    }

    async fn open(&mut self) -> DeviceResult<()> {
        Ok(())
    }

    async fn close(&mut self) -> DeviceResult<()> {
        Ok(())
    }

    async fn read_bytes(&mut self, _buf: &mut [u8]) -> DeviceResult<usize> {
        Ok(0)
    }

    async fn write_bytes(&mut self, _buf: &[u8]) -> DeviceResult<usize> {
        Ok(0)
    }

    async fn read_block(&mut self, _block: u32, _buf: &mut [u8]) -> DeviceResult<usize> {
        Ok(0)
    }

    async fn write_block(&mut self, _block: u32, _buf: &[u8]) -> DeviceResult<usize> {
        Ok(0)
    }

    async fn get_status(&self) -> DeviceResult<DeviceStatus> {
        Ok(DeviceStatus::Ready)
    }
}

#[async_trait]
impl NetworkDevice for MockNetworkDevice {
    async fn connect(&mut self, endpoint: &str) -> DeviceResult<()> {
        self.protocol.open(endpoint).await
    }

    async fn disconnect(&mut self) -> DeviceResult<()> {
        self.protocol.close().await
    }

    async fn open_url(&mut self, url: &NetworkUrl) -> DeviceResult<()> {
        self.protocol.open(&url.url).await
    }

    fn protocol_handler(&mut self) -> &mut dyn ProtocolHandler {
        self.protocol.as_mut()
    }
}

// Mock HTTP client provider for testing
pub struct MockHttpClientProvider {
    client: MockHttpClient,
}

impl MockHttpClientProvider {
    pub fn new(client: MockHttpClient) -> Self {
        Self { client }
    }
}

impl HttpClientProvider for MockHttpClientProvider {
    fn create_http_client(&self) -> Box<dyn HttpClient> {
        Box::new(self.client.clone())
    }
} 