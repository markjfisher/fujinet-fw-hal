use crate::device::network::manager::NetworkManager;
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

/// Opens a network device with the given parameters
/// 
/// # Arguments
/// * `request` - The device open request containing specification, mode and translation
/// 
/// # Returns
/// * `Ok(usize)` - The device ID if successful
/// * `Err(AdapterError)` - The error if opening failed
pub fn open_device(request: DeviceOpenRequest) -> Result<usize, AdapterError> {
    let mut manager = NetworkManager::new();
    
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
    let mut manager = NetworkManager::new();
    
    if !manager.close_device(device_id) {
        return Err(AdapterError::DeviceError(DeviceError::InvalidUrl));
    }

    Ok(())
} 