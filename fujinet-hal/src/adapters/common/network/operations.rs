use crate::device::network::manager::{NetworkManager, NetworkManagerImpl};
use crate::device::DeviceError;
use crate::adapters::common::error::AdapterError;
use tokio::runtime::Runtime;
use crate::device::network::protocols::{http::HttpProtocol, HttpProtocolHandler};
use std::sync::{Arc, Mutex};

/// Common request structure for opening a network device
#[derive(Debug)]
pub struct DeviceOpenRequest {
    /// The device specification string (e.g. "N1:http://ficticious_example.madeup")
    pub device_spec: String,
    /// The mode for opening the device
    pub mode: u8,
    /// The translation setting
    pub translation: u8,
}

/// Common request structure for HTTP POST operations
#[derive(Debug)]
pub struct HttpPostRequest {
    /// The device specification string (e.g. "N1:http://ficticious_example.madeup")
    pub device_spec: String,
    /// The data to POST
    pub data: Vec<u8>,
}

/// Context for network operations that manages dependencies
pub struct OperationsContext<M: NetworkManager> {
    manager: Arc<Mutex<M>>,
}

impl<M: NetworkManager> OperationsContext<M> {
    /// Create a new context with the given network manager
    pub fn new(manager: M) -> Self {
        Self {
            manager: Arc::new(Mutex::new(manager))
        }
    }

    /// Open a network device
    pub fn open_device(&self, request: DeviceOpenRequest) -> Result<usize, AdapterError> {
        let mut manager = self.manager.lock().unwrap();
        // Parse and validate the device specification
        let (device_id, _url) = manager.parse_device_spec(&request.device_spec)
            .map_err(|_| AdapterError::InvalidDeviceSpec)?;

        // Create runtime and execute open_device
        let rt = Runtime::new().unwrap();
        rt.block_on(manager.open_device(&request.device_spec, request.mode, request.translation))
            .map_err(AdapterError::from)?;

        Ok(device_id)
    }

    /// Close a network device
    pub fn close_device(&self, device_id: usize) -> Result<(), AdapterError> {
        let mut manager = self.manager.lock().unwrap();
        // Create runtime and execute close_device
        let rt = Runtime::new().unwrap();
        let closed = rt.block_on(manager.close_device(device_id))
            .map_err(AdapterError::from)?;

        if !closed {
            return Err(AdapterError::DeviceError(DeviceError::InvalidUrl));
        }

        Ok(())
    }

    /// Perform an HTTP POST operation
    pub fn http_post(&self, request: HttpPostRequest) -> Result<(), AdapterError> {
        let mut manager = self.manager.lock().unwrap();
        let rt = Runtime::new().unwrap();
        
        // Parse device spec to get device ID and URL
        let (device_id, url) = manager.parse_device_spec(&request.device_spec)
            .map_err(|_| AdapterError::InvalidDeviceSpec)?;

        // Get the device from protocol factory
        let device = manager.get_network_device(device_id)
            .ok_or(AdapterError::DeviceError(DeviceError::InvalidUrl))?;

        // Get the protocol handler and downcast it to HTTP
        let protocol = device.protocol_handler();
        let http_protocol = protocol.as_any_mut()
            .downcast_mut::<HttpProtocol>()
            .ok_or(AdapterError::DeviceError(DeviceError::InvalidUrl))?;

        // Execute POST request with raw URL
        rt.block_on(http_protocol.post(&url.url, &request.data))
            .map(|_| ())  // Discard the response data
            .map_err(AdapterError::from)?;

        Ok(())
    }
}

impl OperationsContext<NetworkManagerImpl> {
    /// Create a default context using production implementations
    pub fn default() -> Self {
        let manager = NetworkManagerImpl::new();
        Self::new(manager)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::DeviceResult;
    use crate::device::network::NetworkUrl;
    use crate::device::manager::DeviceState;
    use crate::device::network::NetworkDevice;
    use async_trait::async_trait;

    // Create a test-specific network manager that avoids reference issues
    struct TestNetworkManager {
        parse_result: Option<(usize, NetworkUrl)>,
        open_result: bool,
        close_result: bool,
    }

    #[async_trait]
    impl NetworkManager for TestNetworkManager {
        fn parse_device_spec(&self, spec: &str) -> DeviceResult<(usize, NetworkUrl)> {
            self.parse_result.clone().ok_or(DeviceError::InvalidUrl)
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

        fn get_device(&mut self, _device_id: usize) -> Option<&mut DeviceState> {
            None
        }

        async fn close_device(&mut self, _device_id: usize) -> DeviceResult<bool> {
            Ok(self.close_result)
        }

        fn get_network_device(&mut self, _device_id: usize) -> Option<&mut Box<dyn NetworkDevice>> {
            None
        }
    }

    impl TestNetworkManager {
        fn new() -> Self {
            Self {
                parse_result: None,
                open_result: false,
                close_result: false,
            }
        }

        fn with_parse_result(mut self, device_id: usize, url: &str) -> Self {
            self.parse_result = Some((device_id, NetworkUrl::parse(url).unwrap()));
            self
        }

        fn with_open_result(mut self, result: bool) -> Self {
            self.open_result = result;
            self
        }

        fn with_close_result(mut self, result: bool) -> Self {
            self.close_result = result;
            self
        }
    }

    #[test]
    fn test_open_device() {
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com")
            .with_open_result(true);

        let context = OperationsContext::new(manager);
        let request = DeviceOpenRequest {
            device_spec: "N1:http://test.com".to_string(),
            mode: 4,
            translation: 0,
        };

        let result = context.open_device(request);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_close_device() {
        let manager = TestNetworkManager::new()
            .with_close_result(true);

        let context = OperationsContext::new(manager);
        let result = context.close_device(1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_http_post() {
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com");

        let context = OperationsContext::new(manager);
        let request = HttpPostRequest {
            device_spec: "N1:http://test.com".to_string(),
            data: vec![1, 2, 3],
        };

        let result = context.http_post(request);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AdapterError::DeviceError(DeviceError::InvalidUrl)));
    }
} 