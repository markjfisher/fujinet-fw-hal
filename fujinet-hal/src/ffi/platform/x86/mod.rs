use tokio::runtime::Runtime;
use crate::platform::Platform;
use crate::ffi::{FujiPlatform, FujiError};

pub mod network;
pub mod host;

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
