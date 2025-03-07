use fujinet_device::network::NetworkDevice;
use fujinet_device::network::protocols::http::HttpProtocol;
use fujinet_core::device::Device;
use fujinet_core::error::DeviceResult;
use std::boxed::Box;

#[tokio::test]
async fn test_network_device_lifecycle() -> DeviceResult<()> {
    // Create a network device with HTTP protocol
    let mut device = NetworkDevice::new(
        "http://example.com".to_string(),
        Box::new(HttpProtocol::new()),
    );

    // Test device name
    assert_eq!(device.name(), "network");

    // Test initial status (should be disconnected)
    let status = device.get_status().await?;
    println!("Initial status: {:?}", status);
    assert_eq!(status, fujinet_core::device::DeviceStatus::Disconnected);

    // Test opening the device
    println!("Opening device...");
    device.open().await?;
    let status = device.get_status().await?;
    println!("Status after open: {:?}", status);
    assert_eq!(status, fujinet_core::device::DeviceStatus::Ready);

    // Set up the request
    println!("Setting up request...");
    device.send_request("GET".to_string(), "/".to_string()).await?;

    // Test writing data
    println!("Writing data...");
    let test_data = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    let bytes_written = device.write_bytes(test_data).await?;
    println!("Bytes written: {}", bytes_written);
    assert_eq!(bytes_written, test_data.len());

    // Test reading data (should be empty initially)
    println!("Reading data...");
    let mut read_buffer = vec![0u8; 1024];
    let bytes_read = device.read_bytes(&mut read_buffer).await?;
    println!("Bytes read: {}", bytes_read);
    assert_eq!(bytes_read, 0);

    // Test closing the device
    println!("Closing device...");
    device.close().await?;
    let status = device.get_status().await?;
    println!("Final status: {:?}", status);
    assert_eq!(status, fujinet_core::device::DeviceStatus::Disconnected);

    Ok(())
}

#[tokio::test]
async fn test_network_device_http_request() -> DeviceResult<()> {
    let mut device = NetworkDevice::new(
        "http://example.com".to_string(),
        Box::new(HttpProtocol::new()),
    );

    // Open the device
    device.open().await?;

    // Send an HTTP request
    device.send_request("GET".to_string(), "/".to_string()).await?;

    // Write the request body
    let request_body = b"Host: example.com\r\n\r\n";
    device.write_bytes(request_body).await?;

    // Read the response (should be empty in this test implementation)
    let mut response_buffer = vec![0u8; 1024];
    let bytes_read = device.read_bytes(&mut response_buffer).await?;
    assert_eq!(bytes_read, 0);

    device.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_network_device_error_handling() -> DeviceResult<()> {
    let mut device = NetworkDevice::new(
        "http://example.com".to_string(),
        Box::new(HttpProtocol::new()),
    );

    // Try to read before opening
    let mut buffer = vec![0u8; 1024];
    assert!(device.read_bytes(&mut buffer).await.is_err());

    // Try to write before opening
    assert!(device.write_bytes(b"test").await.is_err());

    // Try to close before opening
    assert!(device.close().await.is_err());

    Ok(())
} 