// use fujinet_hal::device::{DeviceResult, DeviceStatus};

// #[tokio::test]
// async fn test_network_device_basic_lifecycle() -> DeviceResult<()> {
//     let mut device = create_network_device("http://example.com".to_string())?;
    
//     assert_eq!(device.name(), "network");
//     let status = device.get_status().await?;
//     assert_eq!(status, DeviceStatus::Disconnected);

//     device.open().await?;
//     let status = device.get_status().await?;
//     assert_eq!(status, DeviceStatus::Ready);

//     device.close().await?;
//     let status = device.get_status().await?;
//     assert_eq!(status, DeviceStatus::Disconnected);

//     Ok(())
// }

// #[tokio::test]
// async fn test_network_device_read_write() {
//     // Create a mock client with pre-configured responses
//     let mut mock = MockHttpClient::new();
//     mock.add_response("http://test.com", vec![7, 8, 9]);

//     // Create protocol and device
//     let mut protocol = HttpProtocol::default();
//     protocol.set_http_client(Box::new(mock));
//     let mut device = NetworkDevice::new(Box::new(protocol));

//     // Test the device
//     device.open("http://test.com").await.unwrap();
    
//     // Write should trigger POST
//     let write_data = vec![1, 2, 3];
//     assert_eq!(device.write(&write_data).await.unwrap(), 3);
    
//     // Read should trigger GET
//     let mut read_buf = vec![0; 3];
//     assert_eq!(device.read(&mut read_buf).await.unwrap(), 3);
//     assert_eq!(read_buf, vec![7, 8, 9]);
// }

// #[tokio::test]
// async fn test_network_device_error_handling() {
//     let mut mock = MockHttpClient::new();
    
//     // Create protocol and device
//     let mut protocol = HttpProtocol::default();
//     protocol.set_http_client(Box::new(mock));
//     let mut device = NetworkDevice::new(Box::new(protocol));

//     // Try operations before opening - should fail
//     let mut read_buf = vec![0; 3];
//     assert!(device.read(&mut read_buf).await.is_err());
//     assert!(device.write(&[1, 2, 3]).await.is_err());
// } 