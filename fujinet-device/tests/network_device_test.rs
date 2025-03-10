use fujinet_core::error::DeviceResult;
mod mock_platform;
use mock_platform::create_network_device;
use crate::mock_platform::MockNetworkDriver;
use fujinet_core::protocol::HttpProtocol;
use fujinet_core::client::MockHttpClient;

#[tokio::test]
async fn test_network_device_basic_lifecycle() -> DeviceResult<()> {
    let mut device = create_network_device("http://example.com".to_string())?;
        
    assert_eq!(device.name(), "network");
    let status = device.get_status().await?;
    assert_eq!(status, fujinet_core::device::DeviceStatus::Disconnected);

    device.open().await?;
    let status = device.get_status().await?;
    assert_eq!(status, fujinet_core::device::DeviceStatus::Ready);

    device.close().await?;
    let status = device.get_status().await?;
    assert_eq!(status, fujinet_core::device::DeviceStatus::Disconnected);

    Ok(())
}

#[tokio::test]
async fn test_network_device_write() -> DeviceResult<()> {
    let mut device = create_network_device("http://example.com".to_string())?;
    device.open().await?;

    let test_request = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    let bytes_written = device.write_bytes(test_request).await?;
    assert_eq!(bytes_written, test_request.len());
    
    let mock_driver = device.get_network_driver().unwrap()
        .as_any().downcast_ref::<MockNetworkDriver>().unwrap();
    assert_eq!(mock_driver.get_written_data(), test_request);

    device.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_network_device_read() -> DeviceResult<()> {
    let mut device = create_network_device("http://example.com".to_string())?;
    device.open().await?;

    let mock_response = b"HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, World!";
    if let Some(driver) = device.get_network_driver() {
        let mock_driver = driver.as_any_mut().downcast_mut::<MockNetworkDriver>().unwrap();
        mock_driver.set_read_data(mock_response.to_vec());
    }

    let mut read_buffer = vec![0u8; 1024];
    let bytes_read = device.read_bytes(&mut read_buffer).await?;
    assert_eq!(bytes_read, mock_response.len());
    assert_eq!(&read_buffer[..bytes_read], mock_response);

    device.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_network_device_http_request() -> DeviceResult<()> {
    let mut device = create_network_device("http://example.com".to_string())?;
    
    // Open the device
    device.open().await?;

    // Write the request
    let request = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    device.write_bytes(request).await?;

    // Read the response (should be empty in this test implementation)
    let mut response_buffer = vec![0u8; 1024];
    let bytes_read = device.read_bytes(&mut response_buffer).await?;
    assert_eq!(bytes_read, 0);

    device.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_network_device_error_handling() -> DeviceResult<()> {
    let mut device = create_network_device("http://example.com".to_string())?;

    // Try to read before opening
    let mut buffer = vec![0u8; 1024];
    assert!(device.read_bytes(&mut buffer).await.is_err());

    // Try to write before opening
    assert!(device.write_bytes(b"test").await.is_err());

    // Try to close before opening
    assert!(device.close().await.is_err());

    Ok(())
}

#[tokio::test]
async fn test_http_protocol_request_response() -> DeviceResult<()> {
    let mut device = create_network_device("http://example.com".to_string())?;
    
    // Open the device
    device.open().await?;
    
    // Get the protocol handler to test HTTP-specific functionality
    if let Some(http) = device.protocol.as_mut().downcast_mut::<HttpProtocol>() {
        // Set up test response
        let mock_response = b"HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, World!";
        if let Some(client) = http.http_client.as_mut().and_then(|c| c.as_any_mut().downcast_mut::<MockHttpClient>()) {
            client.set_response(200, mock_response.to_vec());
        }
        
        // Test HTTP operations
        http.send_request("GET", "/", &[]).await?;
        let status_code = http.get_status_code().await?;
        assert_eq!(status_code, 200);
        
        // Test headers
        http.add_header("User-Agent", "FujiNet").await?;
        let headers = http.get_headers().await?;
        assert_eq!(headers.get("User-Agent"), Some(&"FujiNet".to_string()));
    }
    
    device.close().await?;
    Ok(())
} 