use libc::size_t;
use tokio::runtime::Runtime;
use crate::device::Device;
use super::{FujiDevice, FujiError};

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
