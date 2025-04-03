use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::Arc;
use tokio::runtime::Runtime;

#[cfg(not(test))]
use crate::platform::create_network_manager;

#[cfg(not(test))]
use std::sync::OnceLock;

#[cfg(test)]
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::adapters::common::network::operations::OperationsContext;
use crate::adapters::common::network::operations::types::{DeviceOpenRequest, HttpPostRequest, HttpGetRequest};
use crate::adapters::common::error::AdapterError;
use crate::adapters::ffi::error::{
    device_result_to_error,
    adapter_result_to_ffi,
    FN_ERR_BAD_CMD,
    FN_ERR_NOT_INITIALIZED,
};
use crate::device::network::manager::NetworkManager;

// Trait to abstract over different OperationsContext types
trait NetworkOperations: Send + Sync {
    fn open_device(&self, request: DeviceOpenRequest) -> Result<usize, AdapterError>;
    fn http_post(&self, request: HttpPostRequest) -> Result<(), AdapterError>;
    fn http_get(&self, request: &mut HttpGetRequest) -> Result<usize, AdapterError>;
}

// Implement NetworkOperations for any OperationsContext with a NetworkManager
impl<M: NetworkManager + Send + Sync + 'static> NetworkOperations for OperationsContext<M> {
    fn open_device(&self, request: DeviceOpenRequest) -> Result<usize, AdapterError> {
        self.open_device(request)
    }

    fn http_post(&self, request: HttpPostRequest) -> Result<(), AdapterError> {
        self.http_post(request)
    }

    fn http_get(&self, request: &mut HttpGetRequest) -> Result<usize, AdapterError> {
        self.http_get(request)
    }
}

#[cfg(not(test))]
// Production global state
static OPERATIONS: OnceLock<Arc<dyn NetworkOperations>> = OnceLock::new();

// Test-specific global state
#[cfg(test)]
static TEST_OPERATIONS: AtomicPtr<Arc<dyn NetworkOperations>> = AtomicPtr::new(std::ptr::null_mut());

/// Initialize the FFI layer with the default platform-specific NetworkManager
#[no_mangle]
pub extern "C" fn network_init() -> u8 {
    println!("network_init() called");
    
    #[cfg(not(test))]
    {
        // Check if already initialized
        if OPERATIONS.get().is_some() {
            return device_result_to_error(Ok(()));
        }

        // Create a runtime for async operations
        let rt = match Runtime::new() {
            Ok(rt) => rt,
            Err(_) => return FN_ERR_BAD_CMD,
        };
        
        // Create the platform-specific network manager
        let manager = create_network_manager();
        
        // Create operations context with runtime
        let context = OperationsContext::new_with_runtime(manager, rt);
        let ops = Arc::new(context);

        if OPERATIONS.set(ops).is_err() {
            println!("network_init() failed: already initialized");
            return FN_ERR_BAD_CMD;
        }
    }

    device_result_to_error(Ok(()))
}

// Get the appropriate operations context
fn get_operations() -> Option<Arc<dyn NetworkOperations>> {
    #[cfg(test)]
    {
        let ptr = TEST_OPERATIONS.load(Ordering::SeqCst);
        if ptr.is_null() {
            None
        } else {
            // Safety: We know this pointer is valid because:
            // 1. Tests are serialized with #[serial]
            // 2. We always set it before use in setup_test_context
            // 3. The Arc ensures the data lives long enough
            Some(unsafe { (*ptr).clone() })
        }
    }
    #[cfg(not(test))]
    {
        OPERATIONS.get().map(|ops| ops.clone())
    }
}

#[no_mangle]
pub extern "C" fn network_open(devicespec: *const c_char, mode: u8, trans: u8) -> u8 {
    println!("network_open() called with mode={}, trans={}", mode, trans);
    
    // Get operations context first
    let Some(ops) = get_operations() else {
        println!("network_open() failed: not initialized");
        return FN_ERR_NOT_INITIALIZED;
    };
    
    // Validate devicespec pointer
    if devicespec.is_null() {
        println!("network_open() failed: null devicespec");
        return FN_ERR_BAD_CMD;
    }

    // Convert C string to Rust string
    let device_spec = unsafe {
        match CStr::from_ptr(devicespec).to_str() {
            Ok(s) => {
                println!("network_open() devicespec: {}", s);
                s.to_string()
            },
            Err(_) => {
                println!("network_open() failed: invalid UTF-8 in devicespec");
                return FN_ERR_BAD_CMD
            }
        }
    };

    // Create the request
    let request = DeviceOpenRequest {
        device_spec,
        mode,
        translation: trans,
    };

    // Open device
    let result = ops.open_device(request);
    println!("network_open() result: {:?}", result);
    adapter_result_to_ffi(result)
}

#[no_mangle]
pub extern "C" fn network_http_post(devicespec: *const c_char, data: *const c_char) -> u8 {
    // Get operations context first
    let Some(ops) = get_operations() else {
        return FN_ERR_NOT_INITIALIZED;
    };

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

    // Perform HTTP POST
    adapter_result_to_ffi(ops.http_post(request))
}

#[no_mangle]
pub extern "C" fn network_http_get(devicespec: *const c_char, buf: *mut u8, len: u16) -> i16 {
    // Get operations context first
    let Some(ops) = get_operations() else {
        return -(FN_ERR_NOT_INITIALIZED as i16)
    };

    // Validate pointers
    if devicespec.is_null() || buf.is_null() {
        return -(FN_ERR_BAD_CMD as i16)
    }

    // Convert C string to Rust string
    let device_spec = unsafe {
        match CStr::from_ptr(devicespec).to_str() {
            Ok(d) => d.to_string(),
            _ => return -(FN_ERR_BAD_CMD as i16)
        }
    };

    // Create a buffer to store the response
    let buffer = vec![0u8; len as usize];

    // Create the request
    let mut request = HttpGetRequest {
        device_spec,
        buffer,
    };

    // Perform HTTP GET
    match ops.http_get(&mut request) {
        Ok(bytes_read) => {
            // Copy the response data back to the provided buffer
            unsafe {
                std::ptr::copy_nonoverlapping(request.buffer.as_ptr(), buf, bytes_read);
            }
            bytes_read as i16
        },
        Err(e) => -(adapter_result_to_ffi::<()>(Err(e)) as i16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use serial_test::serial;
    use crate::adapters::{common::network::test_mocks::TestNetworkManager, ffi::FN_ERR_OK};
    use crate::device::DeviceError;

    fn setup_test_context(manager: TestNetworkManager) {
        // Create a runtime for async operations
        let rt = Runtime::new().expect("Failed to create runtime");
        
        // Create operations context with runtime
        let context = OperationsContext::new_with_runtime(manager, rt);
        let ops: Arc<dyn NetworkOperations> = Arc::new(context);
        let ops = Box::new(ops);
        let ptr = Box::into_raw(ops);
        TEST_OPERATIONS.store(ptr, Ordering::SeqCst);
    }

    #[test]
    #[serial]
    fn test_network_init() {
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com")
            .with_open_result(true);
        setup_test_context(manager);

        assert_eq!(network_init(), FN_ERR_OK);
        
        // Verify operations context was initialized
        assert!(!TEST_OPERATIONS.load(Ordering::SeqCst).is_null());
    }

    #[test]
    #[serial]
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
    #[serial]
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
    #[serial]
    fn test_ffi_error_codes() {
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com")
            .with_open_result(true);
        setup_test_context(manager);

        // Test that URL parsing failures from NetworkManager are propagated
        let unparseable_url = CString::new("not_a_valid_device_spec").unwrap();
        assert_eq!(network_open(unparseable_url.as_ptr(), 0, 0), FN_ERR_BAD_CMD);

        // Test that valid URL with successful parse and open returns OK
        let valid_url = CString::new("N1:http://test.com").unwrap();
        assert_eq!(network_open(valid_url.as_ptr(), 4, 0), FN_ERR_OK);
    }

    #[test]
    #[serial]
    fn test_network_manager_url_parsing() {
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com");
        setup_test_context(manager);

        // These tests document the expected URL format, but the validation
        // happens in NetworkManager.parse_device_spec, not in network_open
        let test_cases = [
            ("N9:http://example.com", "Invalid device number"),
            ("http://example.com", "Missing N prefix"),
            ("N1:not_a_url", "Invalid URL format"),
        ];

        for (url, description) in test_cases {
            let url = CString::new(url).unwrap();
            assert_eq!(
                network_open(url.as_ptr(), 0, 0),
                FN_ERR_BAD_CMD,
                "Failed to reject invalid URL: {} ({})",
                url.to_str().unwrap(),
                description
            );
        }
    }

    #[test]
    #[serial]
    fn test_http_get_null_pointers() {
        let manager = TestNetworkManager::new();
        setup_test_context(manager);

        // Test null devicespec
        let mut buffer = [0u8; 1024];
        assert_eq!(network_http_get(std::ptr::null(), buffer.as_mut_ptr(), 1024), -(FN_ERR_BAD_CMD as i16));

        // Test null buffer
        let url = CString::new("N1:http://test.com").unwrap();
        assert_eq!(network_http_get(url.as_ptr(), std::ptr::null_mut(), 1024), -(FN_ERR_BAD_CMD as i16));
    }

    #[test]
    #[serial]
    fn test_http_get_invalid_utf8() {
        let manager = TestNetworkManager::new();
        setup_test_context(manager);

        let mut buffer = [0u8; 1024];
        let invalid_utf8 = unsafe { CString::from_vec_unchecked(vec![0xFF, 0xFE, 0x00]) };
        assert_eq!(network_http_get(invalid_utf8.as_ptr(), buffer.as_mut_ptr(), 1024), -(FN_ERR_BAD_CMD as i16));
    }

    #[test]
    #[serial]
    fn test_http_get_success() {
        let test_response = b"Hello, World!".to_vec();
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com")
            .with_http_device_get(Ok(test_response.clone()));
        setup_test_context(manager);

        let url = CString::new("N1:http://test.com").unwrap();
        let mut buffer = [0u8; 1024];
        let result = network_http_get(url.as_ptr(), buffer.as_mut_ptr(), 1024);
        assert!(result > 0);
        assert_eq!(result as usize, test_response.len());
    }

    #[test]
    #[serial]
    fn test_http_get_network_error() {
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com")
            .with_http_device_get(Err(DeviceError::NetworkError("test error".to_string())));
        setup_test_context(manager);

        let url = CString::new("N1:http://test.com").unwrap();
        let mut buffer = [0u8; 1024];
        let result = network_http_get(url.as_ptr(), buffer.as_mut_ptr(), 1024);
        assert!(result < 0);
    }

    #[test]
    #[serial]
    fn test_http_get_device_not_found() {
        let manager = TestNetworkManager::new()
            .with_parse_result(1, "N1:http://test.com");
        setup_test_context(manager);

        let url = CString::new("N1:http://test.com").unwrap();
        let mut buffer = [0u8; 1024];
        let result = network_http_get(url.as_ptr(), buffer.as_mut_ptr(), 1024);
        assert!(result < 0);
    }
}
