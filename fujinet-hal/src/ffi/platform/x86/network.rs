use std::ffi::c_char;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use crate::device::network::protocols::{HttpClient, is_protocol_supported};
use crate::platform::x86::network::X86HttpClient;
use crate::ffi::device_result_to_error;
use crate::device::network::NetworkUrl;

// Maximum number of network devices supported
const MAX_NETWORK_DEVICES: usize = 8;

#[derive(Default)]
struct NetworkDeviceState {
    mode: u8,
    trans: u8,
    url: Option<NetworkUrl>,
    client: Option<Box<dyn HttpClient>>,
}

struct NetworkDeviceManager {
    devices: [NetworkDeviceState; MAX_NETWORK_DEVICES],
}

impl NetworkDeviceManager {
    fn new() -> Self {
        Self {
            devices: std::array::from_fn(|_| NetworkDeviceState::default()),
        }
    }

    fn get_device(&mut self, device_id: usize) -> Option<&mut NetworkDeviceState> {
        if device_id < MAX_NETWORK_DEVICES {
            Some(&mut self.devices[device_id])
        } else {
            None
        }
    }
}

// Global state for C interface
static DEVICE_MANAGER: Lazy<Mutex<NetworkDeviceManager>> = Lazy::new(|| Mutex::new(NetworkDeviceManager::new()));

#[no_mangle]
pub extern "C" fn network_init() -> u8 {
    // Initialize the device manager
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

        // Validate the protocol scheme
        match url.scheme() {
            Ok(scheme) => {
                if !is_protocol_supported(scheme) {
                    return 2; // FN_ERR_BAD_CMD for unsupported protocol
                }
            }
            Err(_) => return 2, // FN_ERR_BAD_CMD for invalid protocol
        }

        // Get the device ID from the URL (N1-N8)
        let device_id = (url.unit - 1) as usize;

        let mut manager = DEVICE_MANAGER.lock().unwrap();
        if let Some(device) = manager.get_device(device_id) {
            // Store the device state
            device.mode = mode;
            device.trans = trans;
            device.url = Some(url);
            
            // Initialize the client if needed
            if device.client.is_none() {
                device.client = Some(Box::new(X86HttpClient::default()));
            }

            0 // FN_ERR_OK
        } else {
            2 // FN_ERR_BAD_CMD for invalid device ID
        }
    }
}

// #[no_mangle]
// pub extern "C" fn network_http_post(devicespec: *const c_char, data: *const c_char) -> u8 {
//     unsafe {
//         if devicespec.is_null() || data.is_null() {
//             return 2; // FN_ERR_BAD_CMD
//         }

//         // Convert C strings to Rust strings without taking ownership
//         let devicespec = match std::ffi::CStr::from_ptr(devicespec).to_str() {
//             Ok(s) => s,
//             Err(_) => return 2, // FN_ERR_BAD_CMD for invalid UTF-8
//         };
        
//         let data = match std::ffi::CStr::from_ptr(data).to_str() {
//             Ok(s) => s.as_bytes(),
//             Err(_) => return 2, // FN_ERR_BAD_CMD for invalid UTF-8
//         };

//         let mut client = HTTP_CLIENT.lock().unwrap();
//         if let Some(client) = client.as_mut() {
//             let rt = Runtime::new().unwrap();
//             let result = rt.block_on(client.post(devicespec, data));
//             device_result_to_error(result.map(|_| ()))
//         } else {
//             5 // FN_ERR_NO_DEVICE
//         }
//     }
// }

// #[no_mangle]
// pub extern "C" fn network_http_post_bin(
//     devicespec: *const c_char,
//     data: *const u8,
//     len: u16,
// ) -> u8 {
//     unsafe {
//         if devicespec.is_null() || data.is_null() {
//             return 2; // FN_ERR_BAD_CMD
//         }

//         // Convert C string to Rust string without taking ownership
//         let devicespec = match std::ffi::CStr::from_ptr(devicespec).to_str() {
//             Ok(s) => s,
//             Err(_) => return 2, // FN_ERR_BAD_CMD for invalid UTF-8
//         };

//         let data = std::slice::from_raw_parts(data, len as usize);

//         let mut client = HTTP_CLIENT.lock().unwrap();
//         if let Some(client) = client.as_mut() {
//             let rt = Runtime::new().unwrap();
//             let result = rt.block_on(client.post(devicespec, data));
//             device_result_to_error(result.map(|_| ()))
//         } else {
//             5 // FN_ERR_NO_DEVICE
//         }
//     }
// }

// #[no_mangle]
// pub extern "C" fn network_http_get(devicespec: *const c_char, buf: *mut u8, len: u16) -> i16 {
//     unsafe {
//         if devicespec.is_null() || buf.is_null() {
//             return -(2 as i16); // -FN_ERR_BAD_CMD
//         }

//         // Convert C string to Rust string without taking ownership
//         let devicespec = match std::ffi::CStr::from_ptr(devicespec).to_str() {
//             Ok(s) => s,
//             Err(_) => return -(2 as i16), // -FN_ERR_BAD_CMD for invalid UTF-8
//         };

//         let mut client = HTTP_CLIENT.lock().unwrap();
//         if let Some(client) = client.as_mut() {
//             let rt = Runtime::new().unwrap();
//             match rt.block_on(client.get(devicespec)) {
//                 Ok(data) => {
//                     // Copy data to the provided buffer
//                     let copy_len = std::cmp::min(data.len(), len as usize);
//                     let buf_slice = std::slice::from_raw_parts_mut(buf, copy_len);
//                     buf_slice.copy_from_slice(&data[..copy_len]);
//                     copy_len as i16
//                 }
//                 Err(_) => -(4 as i16), // -FN_ERR_WARNING for network errors
//             }
//         } else {
//             -(5 as i16) // -FN_ERR_NO_DEVICE
//         }
//     }
// }

// #[no_mangle]
// pub extern "C" fn network_http_delete(devicespec: *const c_char, _trans: u8) -> u8 {
//     unsafe {
//         if devicespec.is_null() {
//             return 2; // FN_ERR_BAD_CMD
//         }

//         // Convert C string to Rust string without taking ownership
//         let devicespec = match std::ffi::CStr::from_ptr(devicespec).to_str() {
//             Ok(s) => s,
//             Err(_) => return 2, // FN_ERR_BAD_CMD for invalid UTF-8
//         };

//         let mut client = HTTP_CLIENT.lock().unwrap();
//         if let Some(client) = client.as_mut() {
//             let rt = Runtime::new().unwrap();
//             let result = rt.block_on(client.delete(devicespec));
//             device_result_to_error(result.map(|_| ()))
//         } else {
//             5 // FN_ERR_NO_DEVICE
//         }
//     }
// }

// #[no_mangle]
// pub extern "C" fn network_http_set_channel_mode(_devicespec: *const c_char, _mode: u8) -> u8 {
//     // TODO: Implement channel mode setting
//     0 // FN_ERR_OK for now
// }

// #[no_mangle]
// pub extern "C" fn network_http_start_add_headers(_devicespec: *const c_char) -> u8 {
//     // TODO: Implement header collection start
//     0 // FN_ERR_OK for now
// }

// #[no_mangle]
// pub extern "C" fn network_http_end_add_headers(_devicespec: *const c_char) -> u8 {
//     // TODO: Implement header collection end
//     0 // FN_ERR_OK for now
// }

// #[no_mangle]
// pub extern "C" fn network_http_add_header(devicespec: *const c_char, header: *const c_char) -> u8 {
//     unsafe {
//         if devicespec.is_null() || header.is_null() {
//             return 2; // FN_ERR_BAD_CMD
//         }

//         // Convert C strings to Rust strings without taking ownership
//         let _devicespec = match std::ffi::CStr::from_ptr(devicespec).to_str() {
//             Ok(s) => s,
//             Err(_) => return 2, // FN_ERR_BAD_CMD for invalid UTF-8
//         };
        
//         let _header = match std::ffi::CStr::from_ptr(header).to_str() {
//             Ok(s) => s,
//             Err(_) => return 2, // FN_ERR_BAD_CMD for invalid UTF-8
//         };

//         let mut client = HTTP_CLIENT.lock().unwrap();
//         if let Some(_client) = client.as_mut() {
//             // TODO: Implement header addition
//             device_result_to_error(Ok(()))
//         } else {
//             5 // FN_ERR_NO_DEVICE
//         }
//     }
// }
