use crate::device::DeviceError;
use crate::adapters::common::error::AdapterError;
use super::{context::OperationsContext, types::DeviceOpenRequest};
use crate::device::network::manager::NetworkManager;

impl<M: NetworkManager> OperationsContext<M> {
    /// Open a network device
    pub fn open_device(&self, request: DeviceOpenRequest) -> Result<usize, AdapterError> {
        println!("OperationsContext::open_device() called with spec: {}", request.device_spec);
        let mut manager = self.manager.lock().unwrap();
        
        // Parse and validate the device specification
        let parse_result = manager.parse_device_spec(&request.device_spec);
        println!("parse_device_spec result: {:?}", parse_result);
        
        let (device_id, _url) = parse_result.map_err(|_| AdapterError::InvalidDeviceSpec)?;

        // Execute open_device using stored runtime
        let open_result = self.runtime.block_on(manager.open_device(&request.device_spec, request.mode, request.translation));
        println!("open_device result: {:?}", open_result);
        
        open_result.map_err(AdapterError::from)?;
        Ok(device_id)
    }

    /// Close a network device
    pub fn close_device(&self, device_id: usize) -> Result<(), AdapterError> {
        let mut manager = self.manager.lock().unwrap();
        
        // Execute close_device using stored runtime
        let closed = self.runtime.block_on(manager.close_device(device_id))
            .map_err(AdapterError::from)?;

        if !closed {
            return Err(AdapterError::DeviceError(DeviceError::IoError("Failed to close device".into())));
        }

        Ok(())
    }

    /// Validate that a device spec matches what was used in open_device
    pub fn validate_device_spec(&self, spec: &str) -> Result<usize, AdapterError> {
        let mut manager = self.manager.lock().unwrap();
        
        // Parse device spec to get ID and URL
        let (id, url) = manager.parse_device_spec(spec)
            .map_err(|_| AdapterError::InvalidDeviceSpec)?;

        // Get device state
        let device_state = manager.get_device(id)
            .ok_or_else(|| AdapterError::DeviceError(DeviceError::InvalidUrl))?;

        // Get stored URL
        let stored_url = device_state.url.as_ref()
            .ok_or_else(|| AdapterError::DeviceError(DeviceError::NotReady))?;

        // Validate URLs match exactly
        if url.url != stored_url.url {
            return Err(AdapterError::DeviceError(DeviceError::InvalidUrl));
        }

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::common::network::test_mocks::TestNetworkManager;
    use crate::device::network::url::NetworkUrl;

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
        if let AdapterError::DeviceError(DeviceError::IoError(msg)) = result.unwrap_err() {
            assert_eq!(msg, "Failed to close device");
        } else {
            panic!("Expected IoError");
        }
    }

    #[test]
    fn test_validate_device_spec_success() {
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com")
            .with_device_state(1, NetworkUrl::parse("N1:http://test.com").unwrap());

        let context = OperationsContext::new(manager);
        let result = context.validate_device_spec("N1:http://test.com");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_validate_device_spec_invalid_spec() {
        let manager = TestNetworkManager::new();  // No parse result set = invalid spec
        let context = OperationsContext::new(manager);
        let result = context.validate_device_spec("invalid");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AdapterError::InvalidDeviceSpec));
    }

    #[test]
    fn test_validate_device_spec_device_not_found() {
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com");
            // No device state set = device not found

        let context = OperationsContext::new(manager);
        let result = context.validate_device_spec("N1:http://test.com");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AdapterError::DeviceError(DeviceError::InvalidUrl)));
    }

    #[test]
    fn test_validate_device_spec_url_mismatch() {
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com")
            .with_device_state(1, NetworkUrl::parse("N1:http://different.com").unwrap());

        let context = OperationsContext::new(manager);
        let result = context.validate_device_spec("N1:http://test.com");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AdapterError::DeviceError(DeviceError::InvalidUrl)));
    }

    #[test]
    fn test_validate_device_spec_no_url() {
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com")
            .with_device_state_no_url(1);  // Device exists but has no URL set

        let context = OperationsContext::new(manager);
        let result = context.validate_device_spec("N1:http://test.com");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AdapterError::DeviceError(DeviceError::NotReady)));
    }
} 