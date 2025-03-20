use crate::device::network::manager::NetworkManager;
use crate::platform::x86::network::get_network_manager as get_manager;
use crate::device::DeviceError;
use crate::adapters::common::error::AdapterError;
use tokio::runtime::Runtime;
use crate::device::network::protocols::http::{HttpProtocol, HttpProtocolHandler};

/// Common request structure for opening a network device
#[derive(Debug)]
pub struct DeviceOpenRequest {
    /// The device specification string (e.g. "N1:http://example.com")
    pub device_spec: String,
    /// The mode for opening the device
    pub mode: u8,
    /// The translation setting
    pub translation: u8,
}

/// Common request structure for HTTP POST operations
#[derive(Debug)]
pub struct HttpPostRequest {
    /// The device specification string (e.g. "N1:http://example.com")
    pub device_spec: String,
    /// The data to POST
    pub data: Vec<u8>,
}

/// Opens a network device with the given parameters
/// 
/// # Arguments
/// * `manager` - The network manager implementation
/// * `request` - The device open request containing specification, mode and translation
/// 
/// # Returns
/// * `Ok(usize)` - The device ID if successful
/// * `Err(AdapterError)` - The error if opening failed
pub fn open_device(
    manager: &mut impl NetworkManager,
    request: DeviceOpenRequest
) -> Result<usize, AdapterError> {
    // Parse and validate the device specification
    let (device_id, _url) = manager.parse_device_spec(&request.device_spec)
        .map_err(|_| AdapterError::InvalidDeviceSpec)?;

    // Create runtime and execute open_device
    let rt = Runtime::new().unwrap();
    rt.block_on(manager.open_device(&request.device_spec, request.mode, request.translation))
        .map_err(AdapterError::from)?;

    Ok(device_id)
}

/// Closes a network device
/// 
/// # Arguments
/// * `manager` - The network manager implementation
/// * `device_id` - The ID of the device to close
/// 
/// # Returns
/// * `Ok(())` - If the device was closed successfully
/// * `Err(AdapterError)` - If closing failed
pub fn close_device(
    manager: &mut impl NetworkManager,
    device_id: usize
) -> Result<(), AdapterError> {
    // Create runtime and execute close_device
    let rt = Runtime::new().unwrap();
    let closed = rt.block_on(manager.close_device(device_id))
        .map_err(AdapterError::from)?;

    if !closed {
        return Err(AdapterError::DeviceError(DeviceError::InvalidUrl));
    }

    Ok(())
}

/// Performs an HTTP POST request
/// 
/// # Arguments
/// * `manager` - The network manager implementation
/// * `request` - The HTTP POST request containing device spec and data
/// 
/// # Returns
/// * `Ok(())` - If the POST was successful
/// * `Err(AdapterError)` - If the POST failed
pub fn http_post(
    manager: &mut impl NetworkManager,
    request: HttpPostRequest
) -> Result<(), AdapterError> {
    let rt = Runtime::new().unwrap();
    
    // Parse device spec to get device ID and URL
    let (device_id, url) = manager.parse_device_spec(&request.device_spec)
        .map_err(|_| AdapterError::InvalidDeviceSpec)?;

    // Get the device from protocol factory
    let device = manager.get_network_device(device_id)
        .ok_or(AdapterError::DeviceError(DeviceError::InvalidUrl))?;

    // Downcast to HTTP protocol handler
    let http_handler = device.as_any_mut()
        .downcast_mut::<HttpProtocol>()
        .ok_or(AdapterError::DeviceError(DeviceError::InvalidUrl))?;

    // Execute POST request with raw URL
    rt.block_on(http_handler.post(&url.url, &request.data))
        .map(|_| ())  // Discard the response data
        .map_err(AdapterError::from)?;

    Ok(())
}

// Public wrapper functions that use the global manager
pub mod global {
    use super::*;
    
    pub fn open_device(request: DeviceOpenRequest) -> Result<usize, AdapterError> {
        let manager = get_manager();
        let mut manager = manager.lock().unwrap();
        super::open_device(&mut *manager, request)
    }

    pub fn close_device(device_id: usize) -> Result<(), AdapterError> {
        let manager = get_manager();
        let mut manager = manager.lock().unwrap();
        super::close_device(&mut *manager, device_id)
    }

    pub fn http_post(request: HttpPostRequest) -> Result<(), AdapterError> {
        let manager = get_manager();
        let mut manager = manager.lock().unwrap();
        super::http_post(&mut *manager, request)
    }
} 