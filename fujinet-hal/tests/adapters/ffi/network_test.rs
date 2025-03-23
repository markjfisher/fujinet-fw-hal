use std::ffi::CString;
use fujinet_hal::adapters::ffi::network::{network_init, network_open, network_http_post};
use fujinet_hal::adapters::ffi::error::{FN_ERR_BAD_CMD, FN_ERR_OK};

#[test]
fn test_network_init() {
    assert_eq!(network_init(), FN_ERR_OK);
}

#[test]
fn test_ffi_null_pointers() {
    // Test null pointers in network_open
    assert_eq!(network_open(std::ptr::null(), 0, 0), FN_ERR_BAD_CMD);
    
    // Test null pointers in network_http_post
    assert_eq!(network_http_post(std::ptr::null(), std::ptr::null()), FN_ERR_BAD_CMD);
    
    // Test one null pointer in network_http_post
    let url = CString::new("N1:http://ficticious_example.madeup").unwrap();
    assert_eq!(network_http_post(url.as_ptr(), std::ptr::null()), FN_ERR_BAD_CMD);
}

#[test]
fn test_ffi_invalid_utf8() {
    // Test invalid UTF-8 sequences
    let invalid_utf8 = unsafe { CString::from_vec_unchecked(vec![0xFF, 0xFE, 0x00]) };
    assert_eq!(network_open(invalid_utf8.as_ptr(), 0, 0), FN_ERR_BAD_CMD);
    
    let invalid_utf8_post = unsafe { CString::from_vec_unchecked(vec![0xFF, 0xFE, 0x00]) };
    let url = CString::new("N1:http://ficticious_example.madeup").unwrap();
    assert_eq!(network_http_post(url.as_ptr(), invalid_utf8_post.as_ptr()), FN_ERR_BAD_CMD);
}

#[test]
fn test_ffi_error_codes() {
    // Test that invalid URLs return BAD_CMD
    let invalid_url = CString::new("not_a_url").unwrap();
    assert_eq!(network_open(invalid_url.as_ptr(), 0, 0), FN_ERR_BAD_CMD);
    
    // Test that valid URLs with invalid units return BAD_CMD
    let invalid_unit = CString::new("N9:http://ficticious_example.madeup").unwrap();
    assert_eq!(network_open(invalid_unit.as_ptr(), 0, 0), FN_ERR_BAD_CMD);
    
    // Test that valid URLs with valid units return OK
    let valid_url = CString::new("N1:http://ficticious_example.madeup").unwrap();
    assert_eq!(network_open(valid_url.as_ptr(), 4, 0), FN_ERR_OK);
}

#[test]
fn test_ffi_post_without_open() {
    // Test that posting to an unopened device returns BAD_CMD
    let url = CString::new("N1:http://ficticious_example.madeup").unwrap();
    let data = CString::new("test_data").unwrap();
    assert_eq!(network_http_post(url.as_ptr(), data.as_ptr()), FN_ERR_BAD_CMD);
}

#[test]
fn test_ffi_device_modes() {
    // Test different device modes through FFI
    let modes = [
        4,   // Read mode
        8,   // Write mode
        12,  // Read/Write mode
    ];

    let url = CString::new("N1:http://ficticious_example.madeup").unwrap();
    for mode in modes {
        assert_eq!(network_open(url.as_ptr(), mode, 0), FN_ERR_OK,
            "Failed to open with mode {}", mode);
    }
}