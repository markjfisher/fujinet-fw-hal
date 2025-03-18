use crate::device::network::manager::get_network_manager;
use super::error::AdapterError;
use crate::device::DeviceError;

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
/// * `request` - The device open request containing specification, mode and translation
/// 
/// # Returns
/// * `Ok(usize)` - The device ID if successful
/// * `Err(AdapterError)` - The error if opening failed
pub fn open_device(request: DeviceOpenRequest) -> Result<usize, AdapterError> {
    let manager = get_network_manager();
    let mut manager = manager.lock().unwrap();
    
    // Parse and validate the device specification
    let (device_id, _url) = manager.parse_device_spec(&request.device_spec)
        .map_err(|_| AdapterError::InvalidDeviceSpec)?;

    // Open the device with the specified parameters
    manager.open_device(&request.device_spec, request.mode, request.translation)
        .map_err(AdapterError::from)?;

    Ok(device_id)
}

/// Closes a network device
/// 
/// # Arguments
/// * `device_id` - The ID of the device to close
/// 
/// # Returns
/// * `Ok(())` - If the device was closed successfully
/// * `Err(AdapterError)` - If closing failed
pub fn close_device(device_id: usize) -> Result<(), AdapterError> {
    let manager = get_network_manager();
    let mut manager = manager.lock().unwrap();
    
    if !manager.close_device(device_id) {
        return Err(AdapterError::DeviceError(DeviceError::InvalidUrl));
    }

    Ok(())
}

/// Performs an HTTP POST request
/// 
/// # Arguments
/// * `request` - The HTTP POST request containing device spec and data
/// 
/// # Returns
/// * `Ok(())` - If the POST was successful
/// * `Err(AdapterError)` - If the POST failed
pub fn http_post(request: HttpPostRequest) -> Result<(), AdapterError> {
    let manager = get_network_manager();
    let mut manager = manager.lock().unwrap();
    
    // Find the device
    let (_device_id, device) = manager.find_device(&request.device_spec)
        .map_err(|_| AdapterError::InvalidDeviceSpec)?
        .ok_or(AdapterError::DeviceError(DeviceError::InvalidUrl))?;

    // Get the HTTP client from the device's client field
    let client = device.client
        .as_mut()
        .ok_or(AdapterError::DeviceError(DeviceError::InvalidUrl))?;

    // Create runtime and execute POST request
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(client.post(&request.device_spec, &request.data))
        .map(|_| ())  // Discard the response data
        .map_err(AdapterError::from)?;

    Ok(())
} 