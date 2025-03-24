use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::OnceLock;
use crate::adapters::common::network::operations::OperationsContext;
use crate::adapters::common::network::{DeviceOpenRequest, HttpPostRequest};
use crate::adapters::ffi::error::{
    device_result_to_error,
    adapter_result_to_ffi,
    FN_ERR_BAD_CMD,
};
use crate::device::network::manager::NetworkManagerImpl;

#[cfg(test)]
use crate::adapters::ffi::test_operations::{setup_test_context, get_test_operations};

// Global operations context
static OPERATIONS: OnceLock<OperationsContext<NetworkManagerImpl>> = OnceLock::new();

#[no_mangle]
pub extern "C" fn network_init() -> u8 {
    // Initialize the operations context
    #[cfg(not(test))]
    let _ = OPERATIONS.get_or_init(|| OperationsContext::default());
    device_result_to_error(Ok(()))
}

#[no_mangle]
pub extern "C" fn network_open(devicespec: *const c_char, mode: u8, trans: u8) -> u8 {
    // Validate devicespec pointer
    if devicespec.is_null() {
        return FN_ERR_BAD_CMD;
    }

    // Convert C string to Rust string
    let device_spec = unsafe {
        match CStr::from_ptr(devicespec).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return FN_ERR_BAD_CMD
        }
    };

    // Create the request
    let request = DeviceOpenRequest {
        device_spec,
        mode,
        translation: trans,
    };

    // Get operations context and open device
    #[cfg(test)]
    let ops = get_test_operations();
    #[cfg(not(test))]
    let ops = OPERATIONS.get().expect("Operations context not initialized");

    adapter_result_to_ffi(ops.open_device(request))
}

#[no_mangle]
pub extern "C" fn network_http_post(devicespec: *const c_char, data: *const c_char) -> u8 {
    // Validate pointers
    if devicespec.is_null() || data.is_null() {
        return FN_ERR_BAD_CMD;
    }

    // Convert C strings to Rust strings
    let (device_spec, data) = unsafe {
        match (
            CStr::from_ptr(devicespec).to_str(),
            CStr::from_ptr(data).to_str()
        ) {
            (Ok(d), Ok(p)) => (d.to_string(), p.as_bytes().to_vec()),
            _ => return FN_ERR_BAD_CMD
        }
    };

    // Create the request
    let request = HttpPostRequest {
        device_spec,
        data,
    };

    // Get operations context and perform HTTP POST
    #[cfg(test)]
    let ops = get_test_operations();
    #[cfg(not(test))]
    let ops = OPERATIONS.get().expect("Operations context not initialized");

    adapter_result_to_ffi(ops.http_post(request))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use crate::adapters::common::network::test_mocks::TestNetworkManager;

    #[test]
    fn test_network_init() {
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com")
            .with_open_result(true);
        setup_test_context(manager);

        assert_eq!(network_init(), FN_ERR_OK);
    }

    #[test]
    fn test_ffi_null_pointers() {
        let manager = TestNetworkManager::new();
        setup_test_context(manager);

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
        let manager = TestNetworkManager::new();
        setup_test_context(manager);

        // Test invalid UTF-8 sequences
        let invalid_utf8 = unsafe { CString::from_vec_unchecked(vec![0xFF, 0xFE, 0x00]) };
        assert_eq!(network_open(invalid_utf8.as_ptr(), 0, 0), FN_ERR_BAD_CMD);
        
        let invalid_utf8_post = unsafe { CString::from_vec_unchecked(vec![0xFF, 0xFE, 0x00]) };
        let url = CString::new("N1:http://ficticious_example.madeup").unwrap();
        assert_eq!(network_http_post(url.as_ptr(), invalid_utf8_post.as_ptr()), FN_ERR_BAD_CMD);
    }

    #[test]
    fn test_ffi_error_codes() {
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com")
            .with_open_result(true);
        setup_test_context(manager);

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
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com");
        setup_test_context(manager);

        // Test that posting to an unopened device returns BAD_CMD
        let url = CString::new("N1:http://ficticious_example.madeup").unwrap();
        let data = CString::new("test_data").unwrap();
        assert_eq!(network_http_post(url.as_ptr(), data.as_ptr()), FN_ERR_BAD_CMD);
    }

    #[test]
    fn test_ffi_device_modes() {
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com")
            .with_open_result(true);
        setup_test_context(manager);

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
}
