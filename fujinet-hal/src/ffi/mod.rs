use std::ffi::c_void;
use crate::device::DeviceError;

// Declare modules first
mod device;
pub mod platform;

// Then re-export what we want to be public
pub use device::*;
pub use platform::*;

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

// Common error conversion function
pub(crate) fn device_result_to_error(result: crate::device::DeviceResult<()>) -> u8 {
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
