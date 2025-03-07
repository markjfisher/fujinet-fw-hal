use std::ffi::c_void;
use libc::size_t;
use crate::error::DeviceError;
use crate::device::Device;
use crate::platform::Platform;
use crate::host::HostTranslator;
use tokio::runtime::Runtime;
use std::borrow::Cow;

// Opaque types for C
pub type FujiDevice = c_void;
pub type FujiPlatform = c_void;
pub type FujiHostTranslator = c_void;

// Error codes for C
#[repr(C)]
pub enum FujiError {
    Ok = 0,
    Io = 1,
    NotReady = 2,
    NotSupported = 3,
    InvalidParameter = 4,
    ConnectionError = 5,
}

impl From<DeviceError> for FujiError {
    fn from(err: DeviceError) -> Self {
        match err {
            DeviceError::Io(_) => FujiError::Io,
            DeviceError::NotReady => FujiError::NotReady,
            DeviceError::NotSupported => FujiError::NotSupported,
            DeviceError::InvalidParameter(_) => FujiError::InvalidParameter,
            DeviceError::ConnectionError(_) => FujiError::ConnectionError,
        }
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