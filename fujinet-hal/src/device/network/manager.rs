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

    pub fn open_device(&mut self, spec: &str, mode: u8, trans: u8) -> DeviceResult<()> {
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

        // Set device state
        if !self.device_manager.set_device_state(device_id, mode, trans, url) {
            return Err(DeviceError::InvalidDeviceId);
        }

        Ok(())
    }

    pub fn get_device(&mut self, device_id: usize) -> Option<&mut DeviceState> {
        self.device_manager.get_device(device_id)
    }
}

// Global network manager
static NETWORK_MANAGER: Lazy<Mutex<NetworkManager>> = Lazy::new(|| {
    Mutex::new(NetworkManager::new())
});

pub fn get_network_manager() -> &'static Mutex<NetworkManager> {
    &NETWORK_MANAGER
} 