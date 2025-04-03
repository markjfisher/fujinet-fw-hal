use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::Arc;

#[cfg(not(test))]
use std::sync::OnceLock;

#[cfg(test)]
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::adapters::common::network::operations::OperationsContext;
use crate::adapters::common::network::{DeviceOpenRequest, HttpPostRequest};
use crate::adapters::common::error::AdapterError;
use crate::adapters::ffi::error::{
    device_result_to_error,
    adapter_result_to_ffi,
    FN_ERR_BAD_CMD,
};
use crate::device::network::manager::NetworkManager;

#[cfg(not(test))]
use crate::device::network::manager::NetworkManagerImpl;


// Trait to abstract over different OperationsContext types
trait NetworkOperations: Send + Sync {
    fn open_device(&self, request: DeviceOpenRequest) -> Result<usize, AdapterError>;
    fn http_post(&self, request: HttpPostRequest) -> Result<(), AdapterError>;
}

// Implement NetworkOperations for any OperationsContext with a NetworkManager
impl<M: NetworkManager + Send + Sync + 'static> NetworkOperations for OperationsContext<M> {
    fn open_device(&self, request: DeviceOpenRequest) -> Result<usize, AdapterError> {
        self.open_device(request)
    }

    fn http_post(&self, request: HttpPostRequest) -> Result<(), AdapterError> {
        self.http_post(request)
    }
}

#[cfg(not(test))]
// Production global state
static OPERATIONS: OnceLock<Arc<dyn NetworkOperations>> = OnceLock::new();

// Test-specific global state
#[cfg(test)]
static TEST_OPERATIONS: AtomicPtr<Arc<dyn NetworkOperations>> = AtomicPtr::new(std::ptr::null_mut());

// Get the appropriate operations context
fn get_operations() -> Arc<dyn NetworkOperations> {
    #[cfg(test)]
    {
        // Safety: We know this pointer is valid because:
        // 1. Tests are serialized with #[serial]
        // 2. We always set it before use in setup_test_context
        // 3. The Arc ensures the data lives long enough
        unsafe {
            (*TEST_OPERATIONS.load(Ordering::SeqCst)).clone()
        }
    }
    #[cfg(not(test))]
    {
        OPERATIONS.get_or_init(|| {
            Arc::new(OperationsContext::<NetworkManagerImpl>::default())
        }).clone()
    }
}

#[no_mangle]
pub extern "C" fn network_init() -> u8 {
    // Initialize operations context
    let _ = get_operations();
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
    let ops = get_operations();
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
    let ops = get_operations();
    adapter_result_to_ffi(ops.http_post(request))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use serial_test::serial;
    use crate::adapters::{common::network::test_mocks::TestNetworkManager, ffi::FN_ERR_OK};

    fn setup_test_context(manager: TestNetworkManager) {
        // Create new Arc with explicit trait object
        let ops: Arc<dyn NetworkOperations> = Arc::new(OperationsContext::new(manager));
        // Box the Arc trait object
        let ops = Box::new(ops);
        // Get raw pointer to the Arc trait object
        let ptr = Box::into_raw(ops);
        // Store the pointer, replacing any previous value
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
}
