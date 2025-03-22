use fujinet_hal::device::DeviceError;
use fujinet_hal::device::network::protocols::http_client::BaseHttpClient;

#[test]
fn test_default_state() {
    let client = BaseHttpClient::default();
    assert_eq!(client.get_network_unit(), 1);
    assert!(client.get_current_state().is_none());
}

#[test]
fn test_parse_network_url_default() {
    let mut client = BaseHttpClient::default();
    let result = client.parse_network_url("N:http://ficticious_example.madeup").unwrap();
    assert_eq!(result, "http://ficticious_example.madeup");
    assert_eq!(client.get_network_unit(), 1);
}

#[test]
fn test_parse_network_url_with_unit() {
    let mut client = BaseHttpClient::default();
    let result = client.parse_network_url("N3:http://ficticious_example.madeup").unwrap();
    assert_eq!(result, "http://ficticious_example.madeup");
    assert_eq!(client.get_network_unit(), 3);
}

#[test]
fn test_parse_network_url_invalid_unit() {
    let mut client = BaseHttpClient::default();
    assert!(matches!(
        client.parse_network_url("N9:http://ficticious_example.madeup"),
        Err(DeviceError::InvalidProtocol)
    ));
    assert!(matches!(
        client.parse_network_url("N0:http://ficticious_example.madeup"),
        Err(DeviceError::InvalidProtocol)
    ));
}

#[test]
fn test_parse_network_url_invalid_format() {
    let mut client = BaseHttpClient::default();
    assert!(matches!(
        client.parse_network_url("http://ficticious_example.madeup"),
        Err(DeviceError::InvalidProtocol)
    ));
    assert!(matches!(
        client.parse_network_url("Nx:http://ficticious_example.madeup"),
        Err(DeviceError::InvalidProtocol)
    ));
}

#[test]
fn test_connection_state_management() {
    let mut client = BaseHttpClient::default();
    
    // Initially no state exists
    assert!(client.get_current_state().is_none());
    
    // Get or create creates new state
    let state = client.get_connection_state();
    assert_eq!(state.url, "");
    assert_eq!(state.status_code, 200);
    assert!(state.headers.is_empty());
    
    // State now exists
    assert!(client.get_current_state().is_some());
    
    // Remove state
    client.remove_current_connection();
    assert!(client.get_current_state().is_none());
}

#[test]
fn test_url_update() {
    let mut client = BaseHttpClient::default();
    let url1 = "http://ficticious_example.madeup/1".to_string();
    let url2 = "http://ficticious_example.madeup/2".to_string();
    
    // Initial URL set
    client.update_url_if_changed(url1.clone());
    assert_eq!(client.get_connection_state().url, url1);
    
    // Update to new URL
    client.update_url_if_changed(url2.clone());
    assert_eq!(client.get_connection_state().url, url2);
}

#[test]
fn test_multiple_units() {
    let mut client = BaseHttpClient::default();
    
    // Set up unit 1
    client.parse_network_url("N1:http://example1.com").unwrap();
    client.get_connection_state().url = "http://example1.com".to_string();
    
    // Set up unit 2
    client.parse_network_url("N2:http://example2.com").unwrap();
    client.get_connection_state().url = "http://example2.com".to_string();
    
    // Verify unit 1 state
    client.parse_network_url("N1:anything").unwrap();
    assert_eq!(client.get_connection_state().url, "http://example1.com");
    
    // Verify unit 2 state
    client.parse_network_url("N2:anything").unwrap();
    assert_eq!(client.get_connection_state().url, "http://example2.com");
    
    // Remove unit 1
    client.parse_network_url("N1:anything").unwrap();
    client.remove_current_connection();
    
    // Unit 1 should be gone, unit 2 should remain
    assert!(client.parse_network_url("N1:anything").is_ok());
    assert!(client.get_current_state().is_none());
    assert!(client.parse_network_url("N2:anything").is_ok());
    assert!(client.get_current_state().is_some());
} 