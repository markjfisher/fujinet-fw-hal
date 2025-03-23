use crate::device::network::manager::{NetworkManager, NetworkManagerImpl};
use crate::device::DeviceError;
use crate::adapters::common::error::AdapterError;
use tokio::runtime::Runtime;
use crate::device::network::protocols::{http::HttpProtocol, HttpProtocolHandler};
use std::sync::{Arc, Mutex};
use crate::device::network::NetworkUrl;
use crate::device::manager::DeviceState;
use crate::device::network::NetworkDevice;

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
    use mockall::predicate::eq;
    use mockall::mock;

    mock! {
        NetworkManagerImpl {
            fn parse_device_spec(&self, spec: &str) -> Result<(usize, NetworkUrl), DeviceError>;
            async fn open_device(&mut self, spec: &str, mode: u8, trans: u8) -> Result<(), DeviceError>;
            async fn find_device<'a>(&'a mut self, spec: &str) -> Result<Option<(usize, &'a mut DeviceState)>, DeviceError>;
            fn get_device<'a>(&'a mut self, device_id: usize) -> Option<&'a mut DeviceState>;
            async fn close_device(&mut self, device_id: usize) -> Result<bool, DeviceError>;
            fn get_network_device<'a>(&'a mut self, device_id: usize) -> Option<&'a mut Box<dyn NetworkDevice>>;
        }

        impl NetworkManager for NetworkManagerImpl {
        }
    }

    #[test]
    fn test_open_device() {
        let mut mock = MockNetworkManagerImpl::new();
        mock.expect_parse_device_spec()
            .with(eq("N1:http://test.com"))
            .returning(|_| Ok((1, NetworkUrl::parse("N1:http://test.com").unwrap())));
        mock.expect_open_device()
            .with(eq("N1:http://test.com"), eq(4), eq(0))
            .returning(|_, _, _| Ok(()));

        let context = OperationsContext::new(mock);
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
        let mut mock = MockNetworkManagerImpl::new();
        mock.expect_close_device()
            .with(eq(1))
            .returning(|_| Ok(true));

        let context = OperationsContext::new(mock);
        let result = context.close_device(1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_http_post() {
        let mut mock = MockNetworkManagerImpl::new();
        mock.expect_parse_device_spec()
            .with(eq("N1:http://test.com"))
            .returning(|_| Ok((1, NetworkUrl::parse("N1:http://test.com").unwrap())));
        
        // Note: We can't easily mock the full HTTP protocol chain in this test
        // Instead we verify the request is properly parsed and passed to the manager
        mock.expect_get_network_device()
            .with(eq(1))
            .returning(|_| None);

        let context = OperationsContext::new(mock);
        let request = HttpPostRequest {
            device_spec: "N1:http://test.com".to_string(),
            data: vec![1, 2, 3],
        };

        let result = context.http_post(request);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AdapterError::DeviceError(DeviceError::InvalidUrl)));
    }
} 