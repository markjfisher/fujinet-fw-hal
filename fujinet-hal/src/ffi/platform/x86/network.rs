use std::ffi::c_char;
use tokio::runtime::Runtime;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use crate::device::network::protocols::HttpClient;
use crate::platform::x86::network::X86HttpClient;
use crate::ffi::device_result_to_error;
use crate::device::network::NetworkUrl;
use crate::device::DeviceError;

// Global state for C interface
static HTTP_CLIENT: Lazy<Mutex<Option<Box<dyn HttpClient>>>> = Lazy::new(|| Mutex::new(None));

#[no_mangle]
pub extern "C" fn network_init() -> u8 {
    // Initialize the HTTP client
    let mut client = HTTP_CLIENT.lock().unwrap();
    *client = Some(Box::new(X86HttpClient::default()));
    device_result_to_error(Ok(()))
}

#[no_mangle]
pub extern "C" fn network_open(devicespec: *const c_char, mode: u8, trans: u8) -> u8 {
    unsafe {
        if devicespec.is_null() {
            return 2; // FN_ERR_BAD_CMD
        }

        // Convert C string to Rust string without taking ownership
        let devicespec = match std::ffi::CStr::from_ptr(devicespec).to_str() {
            Ok(s) => s,
            Err(_) => return 2, // FN_ERR_BAD_CMD for invalid UTF-8
        };

        // Parse the network URL
        let url = match NetworkUrl::parse(devicespec) {
            Ok(url) => url,
            Err(_) => return 2, // FN_ERR_BAD_CMD for invalid URL
        };

        let mut client = HTTP_CLIENT.lock().unwrap();
        if let Some(client) = client.as_mut() {
            let rt = Runtime::new().unwrap();
            match rt.block_on(client.open(&url, mode, trans)) {
                Ok(_) => 0, // FN_ERR_OK
                Err(e) => device_result_to_error(Err(e)),
            }
        } else {
            5 // FN_ERR_NO_DEVICE
        }
    }
}

#[no_mangle]
pub extern "C" fn network_http_post(devicespec: *const c_char, data: *const c_char) -> u8 {
    unsafe {
        if devicespec.is_null() || data.is_null() {
            return 2; // FN_ERR_BAD_CMD
        }

        // Convert C strings to Rust strings without taking ownership
        let devicespec = match std::ffi::CStr::from_ptr(devicespec).to_str() {
            Ok(s) => s,
            Err(_) => return 2, // FN_ERR_BAD_CMD for invalid UTF-8
        };
        
        let data = match std::ffi::CStr::from_ptr(data).to_str() {
            Ok(s) => s.as_bytes(),
            Err(_) => return 2, // FN_ERR_BAD_CMD for invalid UTF-8
        };

        let mut client = HTTP_CLIENT.lock().unwrap();
        if let Some(client) = client.as_mut() {
            let rt = Runtime::new().unwrap();
            let result = rt.block_on(client.post(devicespec, data));
            device_result_to_error(result.map(|_| ()))
        } else {
            5 // FN_ERR_NO_DEVICE
        }
    }
}

#[no_mangle]
pub extern "C" fn network_http_post_bin(
    devicespec: *const c_char,
    data: *const u8,
    len: u16,
) -> u8 {
    unsafe {
        if devicespec.is_null() || data.is_null() {
            return 2; // FN_ERR_BAD_CMD
        }

        // Convert C string to Rust string without taking ownership
        let devicespec = match std::ffi::CStr::from_ptr(devicespec).to_str() {
            Ok(s) => s,
            Err(_) => return 2, // FN_ERR_BAD_CMD for invalid UTF-8
        };

        let data = std::slice::from_raw_parts(data, len as usize);

        let mut client = HTTP_CLIENT.lock().unwrap();
        if let Some(client) = client.as_mut() {
            let rt = Runtime::new().unwrap();
            let result = rt.block_on(client.post(devicespec, data));
            device_result_to_error(result.map(|_| ()))
        } else {
            5 // FN_ERR_NO_DEVICE
        }
    }
}

#[no_mangle]
pub extern "C" fn network_http_get(devicespec: *const c_char, buf: *mut u8, len: u16) -> i16 {
    unsafe {
        if devicespec.is_null() || buf.is_null() {
            return -(2 as i16); // -FN_ERR_BAD_CMD
        }

        // Convert C string to Rust string without taking ownership
        let devicespec = match std::ffi::CStr::from_ptr(devicespec).to_str() {
            Ok(s) => s,
            Err(_) => return -(2 as i16), // -FN_ERR_BAD_CMD for invalid UTF-8
        };

        let mut client = HTTP_CLIENT.lock().unwrap();
        if let Some(client) = client.as_mut() {
            let rt = Runtime::new().unwrap();
            match rt.block_on(client.get(devicespec)) {
                Ok(data) => {
                    // Copy data to the provided buffer
                    let copy_len = std::cmp::min(data.len(), len as usize);
                    let buf_slice = std::slice::from_raw_parts_mut(buf, copy_len);
                    buf_slice.copy_from_slice(&data[..copy_len]);
                    copy_len as i16
                }
                Err(_) => -(4 as i16), // -FN_ERR_WARNING for network errors
            }
        } else {
            -(5 as i16) // -FN_ERR_NO_DEVICE
        }
    }
}

#[no_mangle]
pub extern "C" fn network_http_delete(devicespec: *const c_char, _trans: u8) -> u8 {
    unsafe {
        if devicespec.is_null() {
            return 2; // FN_ERR_BAD_CMD
        }

        // Convert C string to Rust string without taking ownership
        let devicespec = match std::ffi::CStr::from_ptr(devicespec).to_str() {
            Ok(s) => s,
            Err(_) => return 2, // FN_ERR_BAD_CMD for invalid UTF-8
        };

        let mut client = HTTP_CLIENT.lock().unwrap();
        if let Some(client) = client.as_mut() {
            let rt = Runtime::new().unwrap();
            let result = rt.block_on(client.delete(devicespec));
            device_result_to_error(result.map(|_| ()))
        } else {
            5 // FN_ERR_NO_DEVICE
        }
    }
}

#[no_mangle]
pub extern "C" fn network_http_set_channel_mode(_devicespec: *const c_char, _mode: u8) -> u8 {
    // TODO: Implement channel mode setting
    0 // FN_ERR_OK for now
}

#[no_mangle]
pub extern "C" fn network_http_start_add_headers(_devicespec: *const c_char) -> u8 {
    // TODO: Implement header collection start
    0 // FN_ERR_OK for now
}

#[no_mangle]
pub extern "C" fn network_http_end_add_headers(_devicespec: *const c_char) -> u8 {
    // TODO: Implement header collection end
    0 // FN_ERR_OK for now
}

#[no_mangle]
pub extern "C" fn network_http_add_header(devicespec: *const c_char, header: *const c_char) -> u8 {
    unsafe {
        if devicespec.is_null() || header.is_null() {
            return 2; // FN_ERR_BAD_CMD
        }

        // Convert C strings to Rust strings without taking ownership
        let _devicespec = match std::ffi::CStr::from_ptr(devicespec).to_str() {
            Ok(s) => s,
            Err(_) => return 2, // FN_ERR_BAD_CMD for invalid UTF-8
        };
        
        let _header = match std::ffi::CStr::from_ptr(header).to_str() {
            Ok(s) => s,
            Err(_) => return 2, // FN_ERR_BAD_CMD for invalid UTF-8
        };

        let mut client = HTTP_CLIENT.lock().unwrap();
        if let Some(_client) = client.as_mut() {
            // TODO: Implement header addition
            device_result_to_error(Ok(()))
        } else {
            5 // FN_ERR_NO_DEVICE
        }
    }
}
