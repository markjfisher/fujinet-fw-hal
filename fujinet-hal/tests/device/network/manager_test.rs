// use fujinet_hal::device::network::manager::{NetworkManagerImpl, NetworkManager};
// use fujinet_hal::device::DeviceError;
// use fujinet_hal::device::network::protocols::{HttpClient, HttpClientProvider};
// use crate::common::MockHttpClient;

// struct MockHttpClientProvider;

// impl HttpClientProvider for MockHttpClientProvider {
//     fn create_http_client(&self) -> Box<dyn HttpClient> {
//         Box::new(MockHttpClient::new())
//     }
// }

// #[test]
// fn test_parse_device_spec_valid() {
//     let manager = NetworkManagerImpl::new(Box::new(MockHttpClientProvider));
    
//     // Test default unit (N:)
//     let result = manager.parse_device_spec("N:http://example.com");
//     assert!(result.is_ok());
//     let (device_id, url) = result.unwrap();
//     assert_eq!(device_id, 0);
//     assert_eq!(url.unit, 1);
//     assert_eq!(url.url, "http://example.com");
    
//     // Test specific units
//     for i in 1..=8 {
//         let spec = format!("N{}:http://example.com", i);
//         let result = manager.parse_device_spec(&spec);
//         assert!(result.is_ok());
//         let (device_id, url) = result.unwrap();
//         assert_eq!(device_id, i-1);
//         assert_eq!(url.unit, i as u8);
//         assert_eq!(url.url, "http://example.com");
//     }
// }

// #[test]
// fn test_parse_device_spec_invalid() {
//     let manager = NetworkManagerImpl::new(Box::new(MockHttpClientProvider));
    
//     // Test invalid URL format
//     let result = manager.parse_device_spec("not_a_valid_url");
//     assert!(result.is_err());
    
//     // Test missing protocol
//     let result = manager.parse_device_spec("N:example.com");
//     assert!(result.is_err());
    
//     // Test unsupported protocol
//     let result = manager.parse_device_spec("N:ftp://example.com");
//     assert!(result.is_err());
//     if let Err(err) = result {
//         assert!(matches!(err, DeviceError::UnsupportedProtocol));
//     }
    
//     // Test invalid unit number
//     let result = manager.parse_device_spec("N9:http://example.com");
//     assert!(result.is_err());
    
//     // Test invalid unit number (0)
//     let result = manager.parse_device_spec("N0:http://example.com");
//     assert!(result.is_err());
// }

// #[test]
// fn test_open_close_device() {
//     let mut manager = NetworkManagerImpl::new(Box::new(MockHttpClientProvider));
    
//     // Open a device
//     let result = manager.open_device("N1:http://example.com", 4, 0);
//     assert!(result.is_ok());
    
//     // Verify device is open by fetching it
//     let device = manager.get_device(0);
//     assert!(device.is_some());
    
//     // Unwrap once and store the reference
//     let device_ref = device.unwrap();
    
//     // Check that the device is open by checking the URL was set
//     assert!(device_ref.url.is_some());
    
//     // check the url is correct
//     assert_eq!(device_ref.url.as_ref().unwrap().url, "http://example.com");
    
//     // Close the device
//     let closed = manager.close_device(0);
//     assert!(closed);
    
//     // Verify device is closed (it should still exist but be in an inactive state)
//     let device = manager.get_device(0);
//     assert!(device.is_some());
//     // Check that the device is closed by checking the URL was removed
//     assert!(device.unwrap().url.is_none());
// }

// #[test]
// fn test_find_device() {
//     let mut manager = NetworkManagerImpl::new(Box::new(MockHttpClientProvider));
    
//     // Try to find a device that hasn't been opened yet
//     let spec = "N1:http://example.com";
//     let find_result = manager.find_device(spec);
    
//     // Should be Ok(None) - valid spec but no device
//     assert!(find_result.is_ok());
//     if let Ok(option) = find_result {
//         assert!(option.is_none());
//     }
    
//     // Open a device
//     let open_result = manager.open_device(spec, 4, 0);
//     assert!(open_result.is_ok());
    
//     // Now finding the device should succeed with Some
//     let find_result = manager.find_device(spec);
//     assert!(find_result.is_ok());
//     if let Ok(option) = find_result {
//         assert!(option.is_some());
//         if let Some((device_id, _)) = option {
//             assert_eq!(device_id, 0);
//         }
//     }
    
//     // Finding with a different URL but same unit should still work
//     let find_result = manager.find_device("N1:https://different.com");
//     assert!(find_result.is_ok());
//     if let Ok(option) = find_result {
//         assert!(option.is_some());
//         if let Some((device_id, _)) = option {
//             assert_eq!(device_id, 0);
//         }
//     }
    
//     // Finding with an invalid URL should fail with error
//     let find_result = manager.find_device("invalid_url");
//     assert!(find_result.is_err());
    
//     // Clean up
//     manager.close_device(0);
// }

// #[test]
// fn test_multiple_devices() {
//     let mut manager = NetworkManagerImpl::new(Box::new(MockHttpClientProvider));
    
//     // Open multiple devices
//     for i in 1..=4 {
//         let spec = format!("N{}:http://example{}.com", i, i);
//         let result = manager.open_device(&spec, 4, 0);
//         assert!(result.is_ok());
//     }
    
//     // Verify all devices can be found
//     for i in 1..=4 {
//         let spec = format!("N{}:http://different{}.com", i, i);
//         let find_result = manager.find_device(&spec);
//         assert!(find_result.is_ok());
//         if let Ok(option) = find_result {
//             assert!(option.is_some());
//             if let Some((device_id, _)) = option {
//                 assert_eq!(device_id, i-1);
//             }
//         }
//     }
    
//     // Verify each device can be fetched directly
//     for i in 0..4 {
//         let device = manager.get_device(i);
//         assert!(device.is_some());
//     }
    
//     // Clean up
//     for i in 0..4 {
//         let closed = manager.close_device(i);
//         assert!(closed);
//     }
// }

// #[test]
// fn test_device_replacement() {
//     let mut manager = NetworkManagerImpl::new(Box::new(MockHttpClientProvider));
    
//     // Open a device
//     let result1 = manager.open_device("N1:http://example1.com", 4, 0);
//     assert!(result1.is_ok());
    
//     // Get the device
//     let device1 = manager.get_device(0);
//     let device1_ref = device1.unwrap();

//     // check the url is correct
//     assert_eq!(device1_ref.url.as_ref().unwrap().url, "http://example1.com");

//     // Open a new device with the same unit
//     let result2 = manager.open_device("N1:http://example2.com", 8, 0);
//     assert!(result2.is_ok());
    
//     // Get the new device (same ID)
//     let device2 = manager.get_device(0);
//     let device2_ref = device2.unwrap();

//     // check the url is correct
//     assert_eq!(device2_ref.url.as_ref().unwrap().url, "http://example2.com");
    
//     // Verify that we can find the new URL
//     let find_result = manager.find_device("N1:http://example2.com");
//     assert!(find_result.is_ok());
//     if let Ok(option) = find_result {
//         assert!(option.is_some());
//     }

//     // TODO: Add this back in once we have a reason to support this
//     // // Verify that we don't find the device with a different URL
//     // let find_result = manager.find_device("N1:http://something_else.com");
//     // assert!(find_result.is_ok());
//     // if let Ok(option) = find_result {
//     //     assert!(option.is_none());
//     // }
    
//     // Clean up
//     manager.close_device(0);
// } 