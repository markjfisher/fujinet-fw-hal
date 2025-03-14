use fujinet_hal::device::{DeviceError, DeviceResult};
use fujinet_hal::device::network::protocols::{HttpProtocol, ProtocolHandler, ConnectionStatus};
use fujinet_hal::device::network::HttpProtocolHandler;
use super::test_utils::{TestHttpClient, TestHttpClientHelpers};

#[tokio::test]
async fn test_http_protocol_default_and_clone() {
    // Test default
    let protocol = HttpProtocol::default();
    assert_eq!(protocol.status().await.unwrap(), ConnectionStatus::Disconnected);
    
    // Test clone
    let mut protocol = HttpProtocol::default();
    protocol.set_http_client(Box::new(TestHttpClient::default()));
    let mut cloned = protocol.clone();
    
    // Cloned instance should be disconnected and have no client
    assert_eq!(cloned.status().await.unwrap(), ConnectionStatus::Disconnected);
    assert!(cloned.get("test").await.is_err());
}

#[tokio::test]
async fn test_http_protocol_convenience_methods() -> DeviceResult<()> {
    let mut protocol = HttpProtocol::default();
    let test_client = TestHttpClient::default();
    protocol.set_http_client(Box::new(test_client.clone()));

    let test_url = "http://test.com";
    let test_body = b"test data";

    // Test GET
    protocol.get(test_url).await?;
    let (method, url, body) = test_client.get_last_request().unwrap();
    assert_eq!(method, "GET");
    assert_eq!(url, test_url);
    assert!(body.is_empty());

    // Test POST
    protocol.post(test_url, test_body).await?;
    let (method, url, body) = test_client.get_last_request().unwrap();
    assert_eq!(method, "POST");
    assert_eq!(url, test_url);
    assert_eq!(body, test_body);

    // Test PUT
    protocol.put(test_url, test_body).await?;
    let (method, url, body) = test_client.get_last_request().unwrap();
    assert_eq!(method, "PUT");
    assert_eq!(url, test_url);
    assert_eq!(body, test_body);

    // Test DELETE
    protocol.delete(test_url).await?;
    let (method, url, body) = test_client.get_last_request().unwrap();
    assert_eq!(method, "DELETE");
    assert_eq!(url, test_url);
    assert!(body.is_empty());

    // Test HEAD
    protocol.head(test_url).await?;
    let (method, url, body) = test_client.get_last_request().unwrap();
    assert_eq!(method, "HEAD");
    assert_eq!(url, test_url);
    assert!(body.is_empty());

    // Test PATCH
    protocol.patch(test_url, test_body).await?;
    let (method, url, body) = test_client.get_last_request().unwrap();
    assert_eq!(method, "PATCH");
    assert_eq!(url, test_url);
    assert_eq!(body, test_body);

    Ok(())
}

#[tokio::test]
async fn test_http_protocol_error_scenarios() {
    let mut protocol = HttpProtocol::default();
    
    // Test operations without client
    assert!(matches!(protocol.open("test").await, Err(DeviceError::NotReady)));
    assert!(matches!(protocol.close().await, Err(DeviceError::NotReady)));
    assert!(matches!(protocol.read(&mut [0; 10]).await, Err(DeviceError::NotReady)));
    assert!(matches!(protocol.write(&[0; 10]).await, Err(DeviceError::NotReady)));
    assert!(matches!(protocol.get("test").await, Err(DeviceError::NotReady)));
    assert!(matches!(protocol.add_header("key", "value").await, Err(DeviceError::NotReady)));
    assert!(matches!(protocol.get_status_code().await, Err(DeviceError::NotReady)));
    assert!(matches!(protocol.get_headers().await, Err(DeviceError::NotReady)));

    // Test invalid request method
    protocol.set_http_client(Box::new(TestHttpClient::default()));
    assert!(matches!(
        protocol.send_request("INVALID", "test", &[]).await,
        Err(DeviceError::InvalidOperation)
    ));
}

#[tokio::test]
async fn test_http_protocol_headers() {
    let mut protocol = HttpProtocol::default();
    let test_client = TestHttpClient::default();
    protocol.set_http_client(Box::new(test_client.clone()));
    
    // Connect first to ensure we have a connection state
    protocol.open("N1:http://example.com").await.unwrap();
    
    // Add headers and verify they're stored
    protocol.add_header("Content-Type", "application/json").await.unwrap();
    protocol.add_header("Authorization", "Bearer token").await.unwrap();
    let headers = protocol.get_headers().await.unwrap();
    assert_eq!(headers.get("Content-Type").unwrap(), "application/json");
    assert_eq!(headers.get("Authorization").unwrap(), "Bearer token");

    // Make a request and verify headers were sent
    protocol.get("http://example.com").await.unwrap();
    let request_headers = test_client.get_last_request_headers().unwrap();
    assert_eq!(request_headers.get("Content-Type").unwrap(), "application/json");
    assert_eq!(request_headers.get("Authorization").unwrap(), "Bearer token");
}

#[tokio::test]
async fn test_http_protocol_connection_state() {
    let mut protocol = HttpProtocol::default();
    let test_client = TestHttpClient::default();
    protocol.set_http_client(Box::new(test_client.clone()));
    
    // Initially not connected
    assert!(!test_client.is_connected());
    
    // Connect and verify state
    protocol.open("N1:http://example.com").await.unwrap();
    assert!(test_client.is_connected());
    
    // Disconnect and verify state
    protocol.close().await.unwrap();
    assert!(!test_client.is_connected());
}

#[tokio::test]
async fn test_http_protocol_response_buffer() -> DeviceResult<()> {
    let mut protocol = HttpProtocol::default();
    let test_client = TestHttpClient::default();
    protocol.set_http_client(Box::new(test_client.clone()));

    // Set up test response data
    test_client.set_response_data(b"test response data");

    // Connect first
    protocol.open("http://test.com").await?;
    
    // Make a request that returns a response
    protocol.get("http://test.com").await?;
    
    // Verify request details
    let (method, url, body) = test_client.get_last_request().unwrap();
    assert_eq!(method, "GET");
    assert_eq!(url, "http://test.com");
    assert!(body.is_empty());
    
    // Read all data in 4-byte chunks
    let mut buf = [0u8; 4];
    let mut total_read = 0;
    loop {
        let read = protocol.read(&mut buf).await?;
        if read == 0 {
            break;
        }
        total_read += read;
    }
    
    // Verify we read all the data
    assert_eq!(total_read, b"test response data".len());
    
    // Verify buffer is exhausted
    let read = protocol.read(&mut buf).await?;
    assert_eq!(read, 0); // EOF

    Ok(())
}

#[tokio::test]
async fn test_http_protocol_handler_implementation() -> DeviceResult<()> {
    let mut protocol = HttpProtocol::default();
    let test_client = TestHttpClient::default();
    protocol.set_http_client(Box::new(test_client.clone()));

    // Connect first
    protocol.open("http://test.com").await?;

    // Test write as POST
    let test_data = b"test data";
    let written = protocol.write(test_data).await?;
    assert_eq!(written, test_data.len());
    
    // Verify it was sent as POST with correct URL
    let (method, url, body) = test_client.get_last_request().unwrap();
    assert_eq!(method, "POST");
    assert_eq!(url, "http://test.com");
    assert_eq!(body, test_data);

    // Test read as GET when no response data
    let mut buf = [0u8; 10];
    let read = protocol.read(&mut buf).await?;
    assert_eq!(read, 0); // No data available initially
    
    // Verify it was sent as GET with correct URL
    let (method, url, body) = test_client.get_last_request().unwrap();
    assert_eq!(method, "GET");
    assert_eq!(url, "http://test.com");
    assert!(body.is_empty());

    Ok(())
} 