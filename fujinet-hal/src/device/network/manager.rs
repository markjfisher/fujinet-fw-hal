use std::sync::Mutex;
use crate::device::manager::{DeviceManager, DeviceState};
use crate::device::network::NetworkUrl;
use crate::device::network::protocols::is_protocol_supported;
use crate::device::DeviceError;
use crate::device::DeviceResult;
use crate::device::network::protocols::HttpClientProvider;

/// Interface for network manager operations
pub trait NetworkManager {
    /// Parses and validates a network URL, returning the device ID and URL
    fn parse_device_spec(&self, spec: &str) -> DeviceResult<(usize, NetworkUrl)>;

    /// Opens a new device with the given spec, mode, and trans
    fn open_device(&mut self, spec: &str, mode: u8, trans: u8) -> DeviceResult<()>;

    /// Finds a device by its spec, returning the device ID and state if found
    fn find_device(&mut self, spec: &str) -> DeviceResult<Option<(usize, &mut DeviceState)>>;

    /// Gets a device by its ID
    fn get_device(&mut self, device_id: usize) -> Option<&mut DeviceState>;

    /// Closes a device by its ID
    fn close_device(&mut self, device_id: usize) -> bool;
}

/// Concrete implementation of the NetworkManager trait
pub struct NetworkManagerImpl {
    device_manager: DeviceManager,
    client_provider: Box<dyn HttpClientProvider>,
}

impl NetworkManagerImpl {
    pub fn new(client_provider: Box<dyn HttpClientProvider>) -> Self {
        Self {
            device_manager: DeviceManager::new(),
            client_provider,
        }
    }
}

impl NetworkManager for NetworkManagerImpl {
    fn parse_device_spec(&self, spec: &str) -> DeviceResult<(usize, NetworkUrl)> {
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

    fn find_device(&mut self, spec: &str) -> DeviceResult<Option<(usize, &mut DeviceState)>> {
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

    fn open_device(&mut self, spec: &str, mode: u8, trans: u8) -> DeviceResult<()> {
        let (device_id, url) = self.parse_device_spec(spec)?;

        // Set device state
        if !self.device_manager.set_device_state(device_id, mode, trans, url.clone()) {
            return Err(DeviceError::InvalidDeviceId);
        }

        // Create and attach HTTP client if it's an HTTP URL
        if let Ok(scheme) = url.scheme() {
            if scheme == "http" || scheme == "https" {
                let client = self.client_provider.create_http_client();
                self.device_manager.set_device_client(device_id, client);
            }
        }

        Ok(())
    }

    fn get_device(&mut self, device_id: usize) -> Option<&mut DeviceState> {
        self.device_manager.get_device(device_id)
    }

    fn close_device(&mut self, device_id: usize) -> bool {
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