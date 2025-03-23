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

// Global operations context
static OPERATIONS: OnceLock<OperationsContext<NetworkManagerImpl>> = OnceLock::new();

#[no_mangle]
pub extern "C" fn network_init() -> u8 {
    // Initialize the operations context
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
    let ops = OPERATIONS.get().expect("Operations context not initialized");
    adapter_result_to_ffi(ops.http_post(request))
}
