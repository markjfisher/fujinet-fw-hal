use std::ffi::CString;
use fujinet_hal::adapters::ffi::network::{network_init, network_open};

#[test]
fn test_network_init() {
    assert_eq!(network_init(), 0); // FN_ERR_OK
}

#[test]
fn test_network_open_null_devicespec() {
    assert_eq!(network_open(std::ptr::null(), 0, 0), 2); // FN_ERR_BAD_CMD
}

#[test]
fn test_network_open_invalid_utf8() {
    let invalid = unsafe { CString::from_vec_unchecked([0xFF, 0xFE, 0x00].to_vec()) };
    assert_eq!(network_open(invalid.as_ptr(), 0, 0), 2); // FN_ERR_BAD_CMD
}

#[test]
fn test_network_open_invalid_url() {
    let invalid = CString::new("not_a_url").unwrap();
    assert_eq!(network_open(invalid.as_ptr(), 0, 0), 2); // FN_ERR_BAD_CMD
}

#[test]
fn test_network_open_valid_urls() {
    // Test N1: (default unit)
    let url1 = CString::new("N:http://example.com").unwrap();
    assert_eq!(network_open(url1.as_ptr(), 4, 0), 0); // FN_ERR_OK

    // Test N2:
    let url2 = CString::new("N2:http://example.com").unwrap();
    assert_eq!(network_open(url2.as_ptr(), 8, 0), 0); // FN_ERR_OK

    // Test N8:
    let url8 = CString::new("N8:http://example.com").unwrap();
    assert_eq!(network_open(url8.as_ptr(), 12, 0), 0); // FN_ERR_OK

    // Test N9: (invalid unit)
    let url9 = CString::new("N9:http://example.com").unwrap();
    assert_eq!(network_open(url9.as_ptr(), 4, 0), 2); // FN_ERR_BAD_CMD
}

#[test]
fn test_network_open_different_protocols() {
    // Test HTTP
    let http = CString::new("N1:http://example.com").unwrap();
    assert_eq!(network_open(http.as_ptr(), 4, 0), 0); // FN_ERR_OK

    // Test HTTPS
    let https = CString::new("N2:https://example.com").unwrap();
    assert_eq!(network_open(https.as_ptr(), 4, 0), 0); // FN_ERR_OK

    // Test invalid protocol
    let invalid = CString::new("N3:ftp://example.com").unwrap();
    assert_eq!(network_open(invalid.as_ptr(), 4, 0), 2); // FN_ERR_BAD_CMD
}

#[test]
fn test_network_open_multiple_devices() {
    // Open multiple devices with different modes
    let urls = [
        ("N1:http://example.com", 4),  // Read mode
        ("N2:http://example.com", 8),  // Write mode
        ("N3:http://example.com", 12), // Read/Write mode
    ];

    for (url, mode) in urls.iter() {
        let c_url = CString::new(*url).unwrap();
        assert_eq!(network_open(c_url.as_ptr(), *mode, 0), 0); // FN_ERR_OK
    }
} 