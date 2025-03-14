use std::ffi::{c_char, c_void};
use libc::size_t;
use crate::device::{Device, DeviceError};
use crate::platform::Platform;
use crate::host::HostTranslator;
use tokio::runtime::Runtime;
use std::borrow::Cow;
use crate::device::DeviceResult;
use crate::device::network::protocols::HttpClient;
use crate::platform::x86::network::X86HttpClient;
use std::sync::Mutex;
use once_cell::sync::Lazy;

// Opaque types for C
pub type FujiDevice = c_void;
pub type FujiPlatform = c_void;
pub type FujiHostTranslator = c_void;

// Error codes for C
#[repr(C)]
#[derive(Debug)]
pub enum FujiError {
    Ok = 0,
    IoError = 1,
    NotReady = 2,
    NotSupported = 3,
    InvalidParameter = 4,
    InvalidProtocol = 5,
    InvalidOperation = 6,
    NetworkError = 7,
}

impl From<DeviceError> for FujiError {
    fn from(err: DeviceError) -> Self {
        match err {
            DeviceError::IoError(_) => FujiError::IoError,
            DeviceError::NotReady => FujiError::NotReady,
            DeviceError::NotSupported => FujiError::NotSupported,
            DeviceError::InvalidProtocol => FujiError::InvalidProtocol,
            DeviceError::InvalidOperation => FujiError::InvalidOperation,
            DeviceError::NetworkError(_) => FujiError::NetworkError,
        }
    }
}

// Global state for C interface
static HTTP_CLIENT: Lazy<Mutex<Option<Box<dyn HttpClient>>>> = Lazy::new(|| Mutex::new(None));

// Error code conversions
fn device_result_to_error(result: DeviceResult<()>) -> u8 {
    match result {
        Ok(_) => 0, // FN_ERR_OK
        Err(e) => match e {
            crate::device::DeviceError::IoError(_) => 1,    // FN_ERR_IO_ERROR
            crate::device::DeviceError::NotReady => 3,      // FN_ERR_OFFLINE
            crate::device::DeviceError::NetworkError(_) => 4, // FN_ERR_NETWORK
            _ => 255, // FN_ERR_UNKNOWN
        },
    }
}

// Device FFI functions
#[no_mangle]
pub extern "C" fn fuji_device_open(device: *mut FujiDevice) -> FujiError {
    unsafe {
        if let Some(device) = device.cast::<Box<dyn Device>>().as_mut() {
            let rt = Runtime::new().unwrap();
            match rt.block_on(device.open()) {
                Ok(_) => FujiError::Ok,
                Err(e) => e.into(),
            }
        } else {
            FujiError::InvalidParameter
        }
    }
}

#[no_mangle]
pub extern "C" fn fuji_device_close(device: *mut FujiDevice) -> FujiError {
    unsafe {
        if let Some(device) = device.cast::<Box<dyn Device>>().as_mut() {
            let rt = Runtime::new().unwrap();
            match rt.block_on(device.close()) {
                Ok(_) => FujiError::Ok,
                Err(e) => e.into(),
            }
        } else {
            FujiError::InvalidParameter
        }
    }
}

#[no_mangle]
pub extern "C" fn fuji_device_read_bytes(
    device: *mut FujiDevice,
    buffer: *mut u8,
    len: size_t,
    bytes_read: *mut size_t,
) -> FujiError {
    unsafe {
        if let Some(device) = device.cast::<Box<dyn Device>>().as_mut() {
            let buffer = std::slice::from_raw_parts_mut(buffer, len);
            let rt = Runtime::new().unwrap();
            match rt.block_on(device.read_bytes(buffer)) {
                Ok(n) => {
                    *bytes_read = n as size_t;
                    FujiError::Ok
                }
                Err(e) => e.into(),
            }
        } else {
            FujiError::InvalidParameter
        }
    }
}

#[no_mangle]
pub extern "C" fn fuji_device_write_bytes(
    device: *mut FujiDevice,
    buffer: *const u8,
    len: size_t,
    bytes_written: *mut size_t,
) -> FujiError {
    unsafe {
        if let Some(device) = device.cast::<Box<dyn Device>>().as_mut() {
            let buffer = std::slice::from_raw_parts(buffer, len);
            let rt = Runtime::new().unwrap();
            match rt.block_on(device.write_bytes(buffer)) {
                Ok(n) => {
                    *bytes_written = n as size_t;
                    FujiError::Ok
                }
                Err(e) => e.into(),
            }
        } else {
            FujiError::InvalidParameter
        }
    }
}

// Platform FFI functions
#[no_mangle]
pub extern "C" fn fuji_platform_initialize(platform: *mut FujiPlatform) -> FujiError {
    unsafe {
        if let Some(platform) = platform.cast::<Box<dyn Platform>>().as_mut() {
            let rt = Runtime::new().unwrap();
            match rt.block_on(platform.initialize()) {
                Ok(_) => FujiError::Ok,
                Err(e) => e.into(),
            }
        } else {
            FujiError::InvalidParameter
        }
    }
}

#[no_mangle]
pub extern "C" fn fuji_platform_shutdown(platform: *mut FujiPlatform) -> FujiError {
    unsafe {
        if let Some(platform) = platform.cast::<Box<dyn Platform>>().as_mut() {
            let rt = Runtime::new().unwrap();
            match rt.block_on(platform.shutdown()) {
                Ok(_) => FujiError::Ok,
                Err(e) => e.into(),
            }
        } else {
            FujiError::InvalidParameter
        }
    }
}

// Host Translator FFI functions
#[no_mangle]
pub extern "C" fn fuji_host_translator_initialize(translator: *mut FujiHostTranslator) -> FujiError {
    unsafe {
        if let Some(translator) = translator.cast::<Box<dyn HostTranslator>>().as_mut() {
            let rt = Runtime::new().unwrap();
            match rt.block_on(translator.initialize()) {
                Ok(_) => FujiError::Ok,
                Err(e) => e.into(),
            }
        } else {
            FujiError::InvalidParameter
        }
    }
}

#[no_mangle]
pub extern "C" fn fuji_host_translator_process_host_data(
    translator: *mut FujiHostTranslator,
    data: *const u8,
    len: size_t,
    output: *mut *mut u8,
    output_len: *mut size_t,
) -> FujiError {
    unsafe {
        if let Some(translator) = translator.cast::<Box<dyn HostTranslator>>().as_mut() {
            let data = std::slice::from_raw_parts(data, len);
            let rt = Runtime::new().unwrap();
            match rt.block_on(translator.process_host_data(data)) {
                Ok(result) => {
                    let vec = match result {
                        Cow::Borrowed(slice) => Vec::from(slice),
                        Cow::Owned(vec) => vec,
                    };
                    *output = vec.as_ptr() as *mut u8;
                    *output_len = vec.len() as size_t;
                    std::mem::forget(vec);
                    FujiError::Ok
                }
                Err(e) => e.into(),
            }
        } else {
            FujiError::InvalidParameter
        }
    }
}

#[no_mangle]
pub extern "C" fn fuji_host_translator_process_device_data(
    translator: *mut FujiHostTranslator,
    data: *const u8,
    len: size_t,
    output: *mut *mut u8,
    output_len: *mut size_t,
) -> FujiError {
    unsafe {
        if let Some(translator) = translator.cast::<Box<dyn HostTranslator>>().as_mut() {
            let data = std::slice::from_raw_parts(data, len);
            let rt = Runtime::new().unwrap();
            match rt.block_on(translator.process_device_data(data)) {
                Ok(result) => {
                    let vec = match result {
                        Cow::Borrowed(slice) => Vec::from(slice),
                        Cow::Owned(vec) => vec,
                    };
                    *output = vec.as_ptr() as *mut u8;
                    *output_len = vec.len() as size_t;
                    std::mem::forget(vec);
                    FujiError::Ok
                }
                Err(e) => e.into(),
            }
        } else {
            FujiError::InvalidParameter
        }
    }
}

#[no_mangle]
pub extern "C" fn network_init() -> u8 {
    // Initialize the HTTP client
    let mut client = HTTP_CLIENT.lock().unwrap();
    *client = Some(Box::new(X86HttpClient::default()));
    device_result_to_error(Ok(()))
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
pub extern "C" fn network_http_get(devicespec: *const c_char) -> u8 {
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
            let result = rt.block_on(client.get(devicespec));
            device_result_to_error(result.map(|_| ()))
        } else {
            5 // FN_ERR_NO_DEVICE
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