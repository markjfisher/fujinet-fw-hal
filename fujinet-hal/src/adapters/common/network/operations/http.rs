use crate::device::DeviceError;
use crate::adapters::common::error::AdapterError;
use super::{context::OperationsContext, types::{HttpPostRequest, HttpGetRequest}};
use crate::device::network::manager::NetworkManager;
use crate::device::network::protocols::http::HttpProtocol;

impl<M: NetworkManager + Send + Sync + 'static> OperationsContext<M> {
    /// Perform an HTTP POST operation
    pub fn http_post(&self, mut request: HttpPostRequest) -> Result<(), AdapterError> {
        let mut manager = self.manager.lock().unwrap();
        
        // Parse device spec only if device_id not set
        let device_id = if let Some(id) = request.device_id {
            id
        } else {
            let (id, _) = manager.parse_device_spec(&request.device_spec)
                .map_err(|_| AdapterError::InvalidDeviceSpec)?;
            request.device_id = Some(id);
            id
        };

        // Execute HTTP POST using stored runtime
        self.runtime.block_on(async {
            if let Some(device) = manager.get_network_device(device_id) {
                let protocol = device.protocol_handler();
                
                // Try to downcast to HttpProtocol
                let http_protocol = protocol.as_any_mut()
                    .downcast_mut::<HttpProtocol>()
                    .ok_or(AdapterError::DeviceError(DeviceError::UnsupportedProtocol))?;

                // Execute POST request
                http_protocol.post("", &request.data) // URL already set during open
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
        
        // Parse device spec to get both device ID and URL
        let (device_id, url) = manager.parse_device_spec(&request.device_spec)
            .map_err(|_| AdapterError::InvalidDeviceSpec)?;
        request.device_id = Some(device_id);

        // Get the device state to validate the URL matches what was used in open
        let device_state = manager.get_device(device_id)
            .ok_or_else(|| AdapterError::DeviceError(DeviceError::InvalidUrl))?;

        // Get the stored URL from device state
        let stored_url = device_state.url.as_ref()
            .ok_or_else(|| AdapterError::DeviceError(DeviceError::NotReady))?;

        // Validate that the base URLs match (host and port)
        if !url.has_same_base_url(stored_url) {
            return Err(AdapterError::DeviceError(DeviceError::InvalidUrl));
        }

        // Execute HTTP GET using stored runtime
        self.runtime.block_on(async {
            if let Some(device) = manager.get_network_device(device_id) {
                let protocol = device.protocol_handler();
                
                // Try to downcast to HttpProtocol
                let http_protocol = protocol.as_any_mut()
                    .downcast_mut::<HttpProtocol>()
                    .ok_or(AdapterError::DeviceError(DeviceError::UnsupportedProtocol))?;

                // Execute GET request with empty URL (use the one from open)
                let response = http_protocol.get("")
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
    use crate::device::network::NetworkUrl;

    #[test]
    fn test_http_post_device_not_found() {
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com");

        let context = OperationsContext::new(manager);
        let request = HttpPostRequest::new(
            "N1:http://test.com".to_string(),
            vec![1, 2, 3]
        );

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
        let mut request = HttpPostRequest::new(
            "N1:http://test.com".to_string(),
            vec![1, 2, 3]
        );
        request.device_id = Some(1); // Simulate pre-parsed device ID

        let result = context.http_post(request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_http_get_device_not_found() {
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com");

        let context = OperationsContext::new(manager);
        let mut request = HttpGetRequest::new(
            "N1:http://test.com".to_string(),
            vec![0; 1024]
        );

        let result = context.http_get(&mut request);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AdapterError::DeviceError(DeviceError::InvalidUrl)));
    }

    #[test]
    fn test_http_get_success() {
        let test_response = b"Hello, World!".to_vec();
        let url = "N1:http://test.com";
        let manager = TestNetworkManager::new()
            .with_parse_result(1, url)
            .with_device_state(1, NetworkUrl::parse(url).unwrap())  // Add device state
            .with_http_device_get(Ok(test_response.clone()));

        let context = OperationsContext::new(manager);
        let mut request = HttpGetRequest::new(
            url.to_string(),  // Use the same URL
            vec![0; 1024]
        );
        request.device_id = Some(1); // Simulate pre-parsed device ID

        let result = context.http_get(&mut request);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_response.len());
        assert_eq!(&request.buffer[..test_response.len()], test_response.as_slice());  // Verify buffer contents
    }

    #[test]
    fn test_http_get_with_different_path() {
        let test_response = b"Hello, World!".to_vec();
        let base_url = "N1:http://192.168.1.100:8085/";
        let request_url = "N1:http://192.168.1.100:8085/get?a=1&b=2";
        
        let manager = TestNetworkManager::new()
            .with_parse_result(1, base_url)
            .with_parse_result(1, request_url)  // Add parsing for request URL
            .with_device_state(1, NetworkUrl::parse(base_url).unwrap())
            .with_http_device_get(Ok(test_response.clone()));

        let context = OperationsContext::new(manager);
        let mut request = HttpGetRequest::new(request_url.to_string(), vec![0; 1024]);
        request.device_id = Some(1);

        let result = context.http_get(&mut request);
        assert!(result.is_ok(), "GET request with different path should succeed");
        assert_eq!(result.unwrap(), test_response.len());
    }
} 