use std::ffi::c_void;
use crate::device::DeviceResult;
use crate::adapters::common::error::AdapterError;

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

/// Maps a basic DeviceResult to an FFI error code
/// Use this only for simple operations that don't need specific error handling
pub fn device_result_to_error<T>(result: DeviceResult<T>) -> u8 {
    match result {
        Ok(_) => FN_ERR_OK,
        Err(_) => FN_ERR_IO_ERROR,
    }
}

/// Maps an AdapterError to an FFI error code with specific error handling
pub fn adapter_error_to_ffi(error: AdapterError) -> u8 {
    match error {
        AdapterError::InvalidDeviceSpec => FN_ERR_BAD_CMD,
        AdapterError::InvalidMode => FN_ERR_BAD_CMD,
        AdapterError::InvalidTranslation => FN_ERR_BAD_CMD,
        AdapterError::DeviceError(device_error) => match device_error {
            // Map specific device errors to appropriate FFI codes
            crate::device::DeviceError::InvalidUrl => FN_ERR_NO_DEVICE,
            crate::device::DeviceError::InvalidDeviceId => FN_ERR_NO_DEVICE,
            crate::device::DeviceError::UnsupportedProtocol => FN_ERR_BAD_CMD,
            _ => FN_ERR_IO_ERROR,
        },
    }
}

/// Converts a Result<T, AdapterError> to an FFI error code
pub fn adapter_result_to_ffi<T>(result: Result<T, AdapterError>) -> u8 {
    match result {
        Ok(_) => FN_ERR_OK,
        Err(e) => adapter_error_to_ffi(e),
    }
}
