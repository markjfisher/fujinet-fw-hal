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

// FujiNet error codes
pub const FN_ERR_OK: u8 = 0x00;      /* No error */
pub const FN_ERR_IO_ERROR: u8 = 0x01; /* There was IO error/issue with the device */
pub const FN_ERR_BAD_CMD: u8 = 0x02;  /* Function called with bad arguments */
pub const FN_ERR_OFFLINE: u8 = 0x03;  /* The device is offline */
pub const FN_ERR_WARNING: u8 = 0x04;  /* Device specific non-fatal warning issued */
pub const FN_ERR_NO_DEVICE: u8 = 0x05; /* There is no network device */
pub const FN_ERR_UNKNOWN: u8 = 0xff;   /* Device specific error we didn't handle */

// Error codes for C
#[repr(C)]
#[derive(Debug)]
pub enum FujiError {
    Ok = FN_ERR_OK as isize,
    IoError = FN_ERR_IO_ERROR as isize,
    NotReady = FN_ERR_OFFLINE as isize,
    NotSupported = FN_ERR_NO_DEVICE as isize,
    InvalidParameter = FN_ERR_BAD_CMD as isize,
    NetworkError = FN_ERR_WARNING as isize,
}

impl From<DeviceError> for FujiError {
    fn from(err: DeviceError) -> Self {
        match err {
            DeviceError::IoError(_) => FujiError::IoError,
            DeviceError::NotReady => FujiError::NotReady,
            DeviceError::NotSupported => FujiError::NotSupported,
            DeviceError::InvalidProtocol => FujiError::InvalidParameter,
            DeviceError::InvalidOperation => FujiError::InvalidParameter,
            DeviceError::NetworkError(_) => FujiError::NetworkError,
            DeviceError::UnsupportedProtocol => FujiError::InvalidParameter,
            DeviceError::InvalidUrl => FujiError::InvalidParameter,
            DeviceError::InvalidDeviceId => FujiError::InvalidParameter,
        }
    }
}

// Common error conversion function
pub(crate) fn device_result_to_error(result: crate::device::DeviceResult<()>) -> u8 {
    match result {
        Ok(_) => FN_ERR_OK,
        Err(e) => match e {
            DeviceError::IoError(_) => FN_ERR_IO_ERROR,
            DeviceError::NotReady => FN_ERR_OFFLINE,
            DeviceError::NetworkError(_) => FN_ERR_WARNING,
            DeviceError::UnsupportedProtocol | 
            DeviceError::InvalidUrl | 
            DeviceError::InvalidDeviceId |
            DeviceError::InvalidProtocol |
            DeviceError::InvalidOperation => FN_ERR_BAD_CMD,
            DeviceError::NotSupported => FN_ERR_NO_DEVICE,
        },
    }
}
