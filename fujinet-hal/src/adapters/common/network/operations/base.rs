use crate::device::DeviceError;
use crate::adapters::common::error::AdapterError;
use tokio::runtime::Runtime;
use super::{context::OperationsContext, types::DeviceOpenRequest};
use crate::device::network::manager::NetworkManager;

impl<M: NetworkManager> OperationsContext<M> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::common::network::test_mocks::TestNetworkManager;

    #[test]
    fn test_open_device_success() {
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com")
            .with_open_result(true);

        let context = OperationsContext::new(manager);
        let request = DeviceOpenRequest {
            device_spec: "N1:http://test.com".to_string(),
            mode: 0,
            translation: 0,
        };

        let result = context.open_device(request);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_open_device_invalid_spec() {
        let manager = TestNetworkManager::new();  // No parse result set = invalid spec

        let context = OperationsContext::new(manager);
        let request = DeviceOpenRequest {
            device_spec: "invalid".to_string(),
            mode: 0,
            translation: 0,
        };

        let result = context.open_device(request);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AdapterError::InvalidDeviceSpec));
    }

    #[test]
    fn test_open_device_open_error() {
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com")
            .with_open_result(false);  // Will cause open to fail

        let context = OperationsContext::new(manager);
        let request = DeviceOpenRequest {
            device_spec: "N1:http://test.com".to_string(),
            mode: 0,
            translation: 0,
        };

        let result = context.open_device(request);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AdapterError::DeviceError(DeviceError::InvalidUrl)));
    }

    #[test]
    fn test_close_device_success() {
        let manager = TestNetworkManager::new()
            .with_close_result(true);

        let context = OperationsContext::new(manager);
        let result = context.close_device(1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_close_device_failure() {
        let manager = TestNetworkManager::new()
            .with_close_result(false);

        let context = OperationsContext::new(manager);
        let result = context.close_device(1);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AdapterError::DeviceError(DeviceError::InvalidUrl)));
    }
} 