use libc::size_t;
use tokio::runtime::Runtime;
use crate::device::Device;
use crate::device::DeviceError;
use crate::adapters::ffi::{FN_ERR_OK, FN_ERR_IO_ERROR, FN_ERR_OFFLINE, FN_ERR_NO_DEVICE, FN_ERR_BAD_CMD, FN_ERR_WARNING};
use super::FujiDevice;

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
