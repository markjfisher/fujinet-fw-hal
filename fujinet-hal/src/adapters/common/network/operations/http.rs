use crate::device::DeviceError;
use crate::adapters::common::error::AdapterError;
use super::{context::OperationsContext, types::{HttpPostRequest, HttpGetRequest}};
use crate::device::network::manager::NetworkManager;
use crate::device::network::protocols::http::HttpProtocol;

impl<M: NetworkManager> OperationsContext<M> {
    /// Perform an HTTP POST operation
    pub fn http_post(&self, request: HttpPostRequest) -> Result<(), AdapterError> {
        let mut manager = self.manager.lock().unwrap();
        
        // Parse and validate the device specification
        let parse_result = manager.parse_device_spec(&request.device_spec);
        let (device_id, url) = parse_result.map_err(|_| AdapterError::InvalidDeviceSpec)?;

        // Execute HTTP POST using stored runtime
        self.runtime.block_on(async {
            if let Some(device) = manager.get_network_device(device_id) {
                let protocol = device.protocol_handler();
                
                // Try to downcast to HttpProtocol
                let http_protocol = protocol.as_any_mut()
                    .downcast_mut::<HttpProtocol>()
                    .ok_or(AdapterError::DeviceError(DeviceError::UnsupportedProtocol))?;

                // Execute POST request
                http_protocol.post(&url.url, &request.data)
                    .await
                    .map(|_| ())  // Discard the response data
                    .map_err(AdapterError::from)
            } else {
                // Return InvalidUrl error when device is not found
                Err(AdapterError::DeviceError(DeviceError::InvalidUrl))
            }
        })
    }

    /// Perform an HTTP GET operation
    pub fn http_get(&self, request: &mut HttpGetRequest) -> Result<usize, AdapterError> {
        let mut manager = self.manager.lock().unwrap();
        
        // Parse and validate the device specification
        let parse_result = manager.parse_device_spec(&request.device_spec);
        let (device_id, url) = parse_result.map_err(|_| AdapterError::InvalidDeviceSpec)?;

        // Execute HTTP GET using stored runtime
        self.runtime.block_on(async {
            if let Some(device) = manager.get_network_device(device_id) {
                let protocol = device.protocol_handler();
                
                // Try to downcast to HttpProtocol
                let http_protocol = protocol.as_any_mut()
                    .downcast_mut::<HttpProtocol>()
                    .ok_or(AdapterError::DeviceError(DeviceError::UnsupportedProtocol))?;

                // Execute GET request
                let response = http_protocol.get(&url.url)
                    .await
                    .map_err(AdapterError::from)?;
                
                // Copy response data into the provided buffer
                let copy_len = std::cmp::min(request.buffer.len(), response.len());
                request.buffer[..copy_len].copy_from_slice(&response[..copy_len]);
                
                Ok(copy_len)
            } else {
                // Return InvalidUrl error when device is not found
                Err(AdapterError::DeviceError(DeviceError::InvalidUrl))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::common::network::test_mocks::TestNetworkManager;
    use crate::device::DeviceError;

    #[test]
    fn test_http_post_device_not_found() {
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

    #[test]
    fn test_http_post_success() {
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com")
            .with_http_device(Ok(()));

        let context = OperationsContext::new(manager);
        let request = HttpPostRequest {
            device_spec: "N1:http://test.com".to_string(),
            data: vec![1, 2, 3],
        };

        let result = context.http_post(request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_http_post_network_error() {
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com")
            .with_http_device(Err(DeviceError::NetworkError("test error".to_string())));

        let context = OperationsContext::new(manager);
        let request = HttpPostRequest {
            device_spec: "N1:http://test.com".to_string(),
            data: vec![1, 2, 3],
        };

        let result = context.http_post(request);
        assert!(result.is_err());
        if let AdapterError::DeviceError(DeviceError::NetworkError(msg)) = result.unwrap_err() {
            assert_eq!(msg, "test error");
        } else {
            panic!("Expected NetworkError");
        }
    }

    #[test]
    fn test_http_get_device_not_found() {
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com");

        let context = OperationsContext::new(manager);
        let mut request = HttpGetRequest {
            device_spec: "N1:http://test.com".to_string(),
            buffer: vec![0; 1024],
        };

        let result = context.http_get(&mut request);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AdapterError::DeviceError(DeviceError::InvalidUrl)));
    }

    #[test]
    fn test_http_get_success() {
        let test_response = b"Hello, World!".to_vec();
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com")
            .with_http_device_get(Ok(test_response.clone()));

        let context = OperationsContext::new(manager);
        let mut request = HttpGetRequest {
            device_spec: "N1:http://test.com".to_string(),
            buffer: vec![0; 1024],
        };

        let result = context.http_get(&mut request);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_response.len());
    }

    #[test]
    fn test_http_get_network_error() {
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com")
            .with_http_device_get(Err(DeviceError::NetworkError("test error".to_string())));

        let context = OperationsContext::new(manager);
        let mut request = HttpGetRequest {
            device_spec: "N1:http://test.com".to_string(),
            buffer: vec![0; 1024],
        };

        let result = context.http_get(&mut request);
        assert!(result.is_err());
        if let AdapterError::DeviceError(DeviceError::NetworkError(msg)) = result.unwrap_err() {
            assert_eq!(msg, "test error");
        } else {
            panic!("Expected NetworkError");
        }
    }
} 