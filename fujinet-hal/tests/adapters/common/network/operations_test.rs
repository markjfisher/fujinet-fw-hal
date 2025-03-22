// use fujinet_hal::device::network::manager::NetworkManager;
// use fujinet_hal::device::network::NetworkUrl;
// use fujinet_hal::device::manager::DeviceState;
// use fujinet_hal::device::DeviceResult;
// use fujinet_hal::device::DeviceError;
// use fujinet_hal::adapters::common::network::operations::{DeviceOpenRequest, HttpPostRequest};
// use fujinet_hal::adapters::common::error::AdapterError;
// use fujinet_hal::device::network::protocols::HttpClient;
// use std::collections::HashMap;
// use std::sync::{Arc, Mutex};
// use async_trait::async_trait;

// /// Mock HTTP client for testing
// #[derive(Debug, Clone)]
// struct MockHttpClient {
//     state: Arc<Mutex<MockHttpClientState>>,
// }

// #[derive(Debug)]
// struct MockHttpClientState {
//     last_url: String,
//     last_post_data: Vec<u8>,
//     headers: HashMap<String, String>,
//     status_code: u16,
//     network_unit: u8,
//     is_connected: bool,
// }

// impl Default for MockHttpClient {
//     fn default() -> Self {
//         Self {
//             state: Arc::new(Mutex::new(MockHttpClientState {
//                 last_url: String::new(),
//                 last_post_data: Vec::new(),
//                 headers: HashMap::new(),
//                 status_code: 200,
//                 network_unit: 1,
//                 is_connected: false,
//             })),
//         }
//     }
// }

// impl MockHttpClient {
//     fn new() -> Self {
//         Self::default()
//     }

//     fn get_last_post(&self) -> Option<(String, Vec<u8>)> {
//         let state = self.state.lock().unwrap();
//         if state.last_url.is_empty() {
//             None
//         } else {
//             Some((state.last_url.clone(), state.last_post_data.clone()))
//         }
//     }
// }

// #[async_trait]
// impl HttpClient for MockHttpClient {
//     async fn connect(&mut self, url: &str) -> DeviceResult<()> {
//         let mut state = self.state.lock().unwrap();
//         state.is_connected = true;
//         // Parse the URL to strip N: prefix if present
//         if let Ok(network_url) = NetworkUrl::parse(url) {
//             state.last_url = network_url.url;
//         } else {
//             state.last_url = url.to_string();
//         }
//         Ok(())
//     }

//     async fn disconnect(&mut self) -> DeviceResult<()> {
//         let mut state = self.state.lock().unwrap();
//         state.is_connected = false;
//         Ok(())
//     }

//     async fn get(&mut self, _url: &str) -> DeviceResult<Vec<u8>> {
//         Ok(vec![])
//     }

//     async fn post(&mut self, url: &str, body: &[u8]) -> DeviceResult<Vec<u8>> {
//         let mut state = self.state.lock().unwrap();
//         // Parse the URL to strip N: prefix if present
//         if let Ok(network_url) = NetworkUrl::parse(url) {
//             state.last_url = network_url.url;
//         } else {
//             state.last_url = url.to_string();
//         }
//         state.last_post_data = body.to_vec();
//         Ok(vec![])
//     }

//     async fn put(&mut self, _url: &str, _body: &[u8]) -> DeviceResult<Vec<u8>> {
//         Ok(vec![])
//     }

//     async fn delete(&mut self, _url: &str) -> DeviceResult<Vec<u8>> {
//         Ok(vec![])
//     }

//     async fn head(&mut self, _url: &str) -> DeviceResult<()> {
//         Ok(())
//     }

//     async fn patch(&mut self, _url: &str, _body: &[u8]) -> DeviceResult<Vec<u8>> {
//         Ok(vec![])
//     }

//     fn set_header(&mut self, key: &str, value: &str) {
//         let mut state = self.state.lock().unwrap();
//         state.headers.insert(key.to_string(), value.to_string());
//     }

//     fn get_status_code(&self) -> u16 {
//         self.state.lock().unwrap().status_code
//     }

//     fn get_headers(&self) -> HashMap<String, String> {
//         self.state.lock().unwrap().headers.clone()
//     }

//     fn get_network_unit(&self) -> u8 {
//         self.state.lock().unwrap().network_unit
//     }
// }

// /// Mock implementation of NetworkManager for testing
// #[derive(Default)]
// struct MockNetworkManager {
//     // Track what devices are open and their states
//     devices: Vec<DeviceState>,
//     // Control behavior of operations
//     should_fail_parse: bool,
//     should_fail_open: bool,
//     should_fail_find: bool,
// }

// impl MockNetworkManager {
//     fn new() -> Self {
//         let mut devices = Vec::with_capacity(8);
//         for _ in 0..8 {
//             devices.push(DeviceState::default());
//         }

//         Self {
//             devices,
//             should_fail_parse: false,
//             should_fail_open: false,
//             should_fail_find: false,
//         }
//     }

//     fn with_parse_error(mut self) -> Self {
//         self.should_fail_parse = true;
//         self
//     }

//     fn with_open_error(mut self) -> Self {
//         self.should_fail_open = true;
//         self
//     }

//     fn with_find_error(mut self) -> Self {
//         self.should_fail_find = true;
//         self
//     }
// }

// impl NetworkManager for MockNetworkManager {
//     fn parse_device_spec(&self, spec: &str) -> DeviceResult<(usize, NetworkUrl)> {
//         if self.should_fail_parse {
//             return Err(DeviceError::InvalidUrl);
//         }

//         // Parse N1:, N2:, etc to get device ID
//         let url = NetworkUrl::parse(spec).map_err(|_| DeviceError::InvalidUrl)?;
//         let device_id = (url.unit - 1) as usize;
        
//         Ok((device_id, url))
//     }

//     fn open_device(&mut self, spec: &str, mode: u8, trans: u8) -> DeviceResult<()> {
//         if self.should_fail_open {
//             return Err(DeviceError::InvalidDeviceId);
//         }

//         let (device_id, url) = self.parse_device_spec(spec)?;
        
//         if device_id >= self.devices.len() {
//             return Err(DeviceError::InvalidDeviceId);
//         }

//         let device = &mut self.devices[device_id];
        
//         // Check if device is already in use
//         if device.url.is_some() {
//             return Err(DeviceError::InvalidDeviceId);
//         }

//         device.mode = mode;
//         device.trans = trans;
//         device.url = Some(url);
        
//         Ok(())
//     }

//     fn find_device(&mut self, spec: &str) -> DeviceResult<Option<(usize, &mut DeviceState)>> {
//         if self.should_fail_find {
//             return Err(DeviceError::InvalidUrl);
//         }

//         let (device_id, _) = self.parse_device_spec(spec)?;
        
//         if device_id >= self.devices.len() {
//             return Ok(None);
//         }

//         // Only return Some if the device has a URL (is opened)
//         if self.devices[device_id].url.is_some() {
//             Ok(Some((device_id, &mut self.devices[device_id])))
//         } else {
//             Ok(None)
//         }
//     }

//     fn get_device(&mut self, device_id: usize) -> Option<&mut DeviceState> {
//         self.devices.get_mut(device_id)
//     }

//     fn close_device(&mut self, device_id: usize) -> bool {
//         if let Some(device) = self.get_device(device_id) {
//             device.url = None;
//             device.client = None;
//             device.mode = 0;
//             device.trans = 0;
//             true
//         } else {
//             false
//         }
//     }
// }

// #[test]
// fn test_open_device_success() {
//     let mut mock = MockNetworkManager::new();
//     let request = DeviceOpenRequest {
//         device_spec: "N1:http://ficticious_example.madeup".to_string(),
//         mode: 4,
//         translation: 0,
//     };

//     let result = fujinet_hal::adapters::common::network::operations::open_device(
//         &mut mock,
//         request
//     );

//     assert!(result.is_ok());
//     let device_id = result.unwrap();
//     assert_eq!(device_id, 0);
    
//     // Verify device state
//     let device = mock.get_device(0).unwrap();
//     assert!(device.url.is_some());
//     assert_eq!(device.mode, 4);
//     assert_eq!(device.trans, 0);
// }

// #[test]
// fn test_open_device_invalid_spec() {
//     let mut mock = MockNetworkManager::new().with_parse_error();
//     let request = DeviceOpenRequest {
//         device_spec: "invalid".to_string(),
//         mode: 4,
//         translation: 0,
//     };

//     let result = fujinet_hal::adapters::common::network::operations::open_device(
//         &mut mock,
//         request
//     );

//     assert!(result.is_err());
//     assert!(matches!(result.unwrap_err(), AdapterError::InvalidDeviceSpec));
// }

// #[test]
// fn test_close_device_success() {
//     let mut mock = MockNetworkManager::new();
    
//     // First open a device
//     let _ = mock.open_device("N1:http://ficticious_example.madeup", 4, 0);
    
//     let result = fujinet_hal::adapters::common::network::operations::close_device(
//         &mut mock,
//         0
//     );

//     assert!(result.is_ok());
    
//     // Verify device is closed
//     let device = mock.get_device(0).unwrap();
//     assert!(device.url.is_none());
//     assert_eq!(device.mode, 0);
//     assert_eq!(device.trans, 0);
// }

// #[test]
// fn test_open_device_fails() {
//     let mut mock = MockNetworkManager::new().with_open_error();
//     let request = DeviceOpenRequest {
//         device_spec: "N1:http://ficticious_example.madeup".to_string(),
//         mode: 4,
//         translation: 0,
//     };

//     let result = fujinet_hal::adapters::common::network::operations::open_device(
//         &mut mock,
//         request
//     );

//     assert!(result.is_err());
//     assert!(matches!(result.unwrap_err(), AdapterError::DeviceError(DeviceError::InvalidDeviceId)));
// }

// #[test]
// fn test_http_post_find_error() {
//     let mut mock = MockNetworkManager::new().with_find_error();
//     let request = HttpPostRequest {
//         device_spec: "N1:http://ficticious_example.madeup".to_string(),
//         data: vec![1, 2, 3],
//     };

//     let result = fujinet_hal::adapters::common::network::operations::http_post(
//         &mut mock,
//         request
//     );

//     assert!(result.is_err());
//     assert!(matches!(result.unwrap_err(), AdapterError::InvalidDeviceSpec));
// }

// #[test]
// fn test_close_device_not_found() {
//     let mut mock = MockNetworkManager::new();
    
//     // Try to close a device that doesn't exist (index out of bounds)
//     let result = fujinet_hal::adapters::common::network::operations::close_device(
//         &mut mock,
//         999 // Invalid device ID
//     );

//     assert!(result.is_err());
//     assert!(matches!(result.unwrap_err(), AdapterError::DeviceError(DeviceError::InvalidUrl)));
// }

// #[test]
// fn test_http_post_device_not_found() {
//     let mut mock = MockNetworkManager::new();
//     let request = HttpPostRequest {
//         device_spec: "N1:http://ficticious_example.madeup".to_string(),
//         data: vec![1, 2, 3],
//     };

//     let result = fujinet_hal::adapters::common::network::operations::http_post(
//         &mut mock,
//         request
//     );

//     assert!(result.is_err());
//     assert!(matches!(result.unwrap_err(), AdapterError::DeviceError(DeviceError::InvalidUrl)));
// }

// #[test]
// fn test_http_post_success() {
//     let mut mock = MockNetworkManager::new();
    
//     // First open a device
//     let _ = mock.open_device("N1:http://ficticious_example.madeup", 4, 0);
    
//     // Add HTTP client to the device
//     let device = mock.get_device(0).unwrap();
//     let http_client = MockHttpClient::new();
//     device.client = Some(Box::new(http_client.clone()));
    
//     let test_data = vec![1, 2, 3, 4, 5];
//     let request = HttpPostRequest {
//         device_spec: "N1:http://ficticious_example.madeup".to_string(),
//         data: test_data.clone(),
//     };

//     let result = fujinet_hal::adapters::common::network::operations::http_post(
//         &mut mock,
//         request
//     );

//     assert!(result.is_ok());
    
//     // Verify the POST data was passed to the client
//     if let Some((url, data)) = http_client.get_last_post() {
//         assert_eq!(data, test_data);
//         assert_eq!(url, "http://ficticious_example.madeup");
//     } else {
//         panic!("No POST request was recorded");
//     }
// }

// #[test]
// fn test_open_device_already_in_use() {
//     let mut mock = MockNetworkManager::new();
    
//     // First open device N1
//     let request1 = DeviceOpenRequest {
//         device_spec: "N1:http://ficticious_example.madeup".to_string(),
//         mode: 4,
//         translation: 0,
//     };
//     let _ = fujinet_hal::adapters::common::network::operations::open_device(&mut mock, request1);
    
//     // Try to open same device with different URL
//     let request2 = DeviceOpenRequest {
//         device_spec: "N1:http://different.com".to_string(),
//         mode: 4,
//         translation: 0,
//     };
//     let result = fujinet_hal::adapters::common::network::operations::open_device(&mut mock, request2);
    
//     assert!(result.is_err());
//     assert!(matches!(result.unwrap_err(), AdapterError::DeviceError(DeviceError::InvalidDeviceId)));
// }

// #[test]
// fn test_close_device_edge_cases() {
//     let mut mock = MockNetworkManager::new();
    
//     // Test closing device at max valid index
//     let result = fujinet_hal::adapters::common::network::operations::close_device(
//         &mut mock,
//         7  // Last valid index (0-7)
//     );
//     assert!(result.is_ok());
    
//     // Test closing device at boundary index
//     let result = fujinet_hal::adapters::common::network::operations::close_device(
//         &mut mock,
//         8  // Just beyond valid range
//     );
//     assert!(result.is_err());
//     assert!(matches!(result.unwrap_err(), AdapterError::DeviceError(DeviceError::InvalidUrl)));
// }

// #[test]
// fn test_device_id_boundaries() {
//     let mut mock = MockNetworkManager::new();
    
//     // Test opening device with max valid ID (N8)
//     let request = DeviceOpenRequest {
//         device_spec: "N8:http://ficticious_example.madeup".to_string(),
//         mode: 4,
//         translation: 0,
//     };
//     let result = fujinet_hal::adapters::common::network::operations::open_device(
//         &mut mock,
//         request
//     );
//     assert!(result.is_ok());
//     assert_eq!(result.unwrap(), 7);  // N8 maps to index 7
    
//     // Test opening device with invalid ID (N9)
//     let request = DeviceOpenRequest {
//         device_spec: "N9:http://ficticious_example.madeup".to_string(),
//         mode: 4,
//         translation: 0,
//     };
//     let result = fujinet_hal::adapters::common::network::operations::open_device(
//         &mut mock,
//         request
//     );
//     assert!(result.is_err());
//     assert!(matches!(result.unwrap_err(), AdapterError::InvalidDeviceSpec));
// }

// #[test]
// fn test_http_post_device_not_opened() {
//     let mut mock = MockNetworkManager::new();
    
//     // Try to POST without opening the device first
//     let request = HttpPostRequest {
//         device_spec: "N1:http://ficticious_example.madeup".to_string(),
//         data: vec![1, 2, 3],
//     };

//     let result = fujinet_hal::adapters::common::network::operations::http_post(
//         &mut mock,
//         request
//     );

//     assert!(result.is_err());
//     assert!(matches!(result.unwrap_err(), AdapterError::DeviceError(DeviceError::InvalidUrl)));
// }

// #[test]
// fn test_http_post_no_client() {
//     let mut mock = MockNetworkManager::new();
    
//     // Open device but don't attach HTTP client
//     let _ = mock.open_device("N1:http://ficticious_example.madeup", 4, 0);
    
//     let request = HttpPostRequest {
//         device_spec: "N1:http://ficticious_example.madeup".to_string(),
//         data: vec![1, 2, 3],
//     };

//     let result = fujinet_hal::adapters::common::network::operations::http_post(
//         &mut mock,
//         request
//     );

//     assert!(result.is_err());
//     assert!(matches!(result.unwrap_err(), AdapterError::DeviceError(DeviceError::InvalidUrl)));
// }

// #[test]
// fn test_http_post_empty_data() {
//     let mut mock = MockNetworkManager::new();
    
//     // First open a device
//     let _ = mock.open_device("N1:http://ficticious_example.madeup", 4, 0);
    
//     // Add HTTP client to the device
//     let device = mock.get_device(0).unwrap();
//     let http_client = MockHttpClient::new();
//     device.client = Some(Box::new(http_client.clone()));
    
//     let request = HttpPostRequest {
//         device_spec: "N1:http://ficticious_example.madeup".to_string(),
//         data: vec![],  // Empty data
//     };

//     let result = fujinet_hal::adapters::common::network::operations::http_post(
//         &mut mock,
//         request
//     );

//     assert!(result.is_ok());
    
//     // Verify empty POST was made
//     if let Some((url, data)) = http_client.get_last_post() {
//         assert!(data.is_empty());
//         assert_eq!(url, "http://ficticious_example.madeup");
//     } else {
//         panic!("No POST request was recorded");
//     }
// }

// #[test]
// fn test_http_post_large_data() {
//     let mut mock = MockNetworkManager::new();
    
//     // First open a device
//     let _ = mock.open_device("N1:http://ficticious_example.madeup", 4, 0);
    
//     // Add HTTP client to the device
//     let device = mock.get_device(0).unwrap();
//     let http_client = MockHttpClient::new();
//     device.client = Some(Box::new(http_client.clone()));
    
//     // Create large test data (64KB)
//     let test_data = vec![0xAA; 65536];
//     let request = HttpPostRequest {
//         device_spec: "N1:http://ficticious_example.madeup".to_string(),
//         data: test_data.clone(),
//     };

//     let result = fujinet_hal::adapters::common::network::operations::http_post(
//         &mut mock,
//         request
//     );

//     assert!(result.is_ok());
    
//     // Verify large POST was made correctly
//     if let Some((url, data)) = http_client.get_last_post() {
//         assert_eq!(data.len(), 65536);
//         assert_eq!(data, test_data);
//         assert_eq!(url, "http://ficticious_example.madeup");
//     } else {
//         panic!("No POST request was recorded");
//     }
// } 