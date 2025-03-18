use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::device::manager::{DeviceManager, DeviceState};
use crate::device::network::NetworkUrl;
use crate::device::network::protocols::is_protocol_supported;
use crate::device::DeviceError;
use crate::device::DeviceResult;

pub struct NetworkManager {
    device_manager: DeviceManager,
}

impl NetworkManager {
    pub fn new() -> Self {
        Self {
            device_manager: DeviceManager::new(),
        }
    }

    /// Parses and validates a network URL, returning the device ID and URL
    pub fn parse_device_spec(&self, spec: &str) -> DeviceResult<(usize, NetworkUrl)> {
        // Parse the network URL
        let url = NetworkUrl::parse(spec)?;

        // Validate the protocol scheme
        if let Ok(scheme) = url.scheme() {
            if !is_protocol_supported(scheme) {
                return Err(DeviceError::UnsupportedProtocol);
            }
        } else {
            return Err(DeviceError::InvalidUrl);
        }

        // Get the device ID from the URL (N1-N8)
        let device_id = (url.unit - 1) as usize;
        if device_id >= 8 {
            return Err(DeviceError::InvalidDeviceId);
        }

        Ok((device_id, url))
    }

    /// Finds a device by its spec, returning the device ID and state if found
    /// Returns an error if the devicespec is invalid
    /// Returns None if the devicespec is valid but no device was found with that ID
    pub fn find_device(&mut self, spec: &str) -> DeviceResult<Option<(usize, &mut DeviceState)>> {
        let (device_id, _) = self.parse_device_spec(spec)?;
        
        if let Some(device) = self.device_manager.get_device(device_id) {
            // Consider a device "found" only if it has a URL (has been opened)
            if device.url.is_some() {
                Ok(Some((device_id, device)))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Opens a new device with the given spec, mode, and trans
    pub fn open_device(&mut self, spec: &str, mode: u8, trans: u8) -> DeviceResult<()> {
        let (device_id, url) = self.parse_device_spec(spec)?;

        // Set device state
        if !self.device_manager.set_device_state(device_id, mode, trans, url) {
            return Err(DeviceError::InvalidDeviceId);
        }

        Ok(())
    }

    /// Gets a device by its ID
    pub fn get_device(&mut self, device_id: usize) -> Option<&mut DeviceState> {
        self.device_manager.get_device(device_id)
    }

    /// Closes a device by its ID
    pub fn close_device(&mut self, device_id: usize) -> bool {
        if let Some(device) = self.device_manager.get_device(device_id) {
            device.url = None;
            device.client = None;
            device.mode = 0;
            device.trans = 0;
            true
        } else {
            false
        }
    }
}

// Global network manager
static NETWORK_MANAGER: Lazy<Mutex<NetworkManager>> = Lazy::new(|| {
    Mutex::new(NetworkManager::new())
});

pub fn get_network_manager() -> &'static Mutex<NetworkManager> {
    &NETWORK_MANAGER
} 