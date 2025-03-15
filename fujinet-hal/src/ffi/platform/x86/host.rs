use std::borrow::Cow;
use libc::size_t;
use tokio::runtime::Runtime;
use crate::host::HostTranslator;
use crate::ffi::{FujiHostTranslator, FujiError};

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
