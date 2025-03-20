use crate::device::manager::{DeviceManager, DeviceState};
use crate::device::network::NetworkUrl;
use crate::device::network::protocols::{NetworkProtocol, ProtocolFactory};
use crate::device::network::network_device::NetworkDevice;
use crate::device::DeviceError;
use crate::device::DeviceResult;
use async_trait::async_trait;

/// Interface for network manager operations
#[async_trait]
pub trait NetworkManager {
    /// Parses and validates a network URL, returning the device ID and URL
    fn parse_device_spec(&self, spec: &str) -> DeviceResult<(usize, NetworkUrl)>;

    /// Opens a new device with the given spec, mode, and trans
    async fn open_device(&mut self, spec: &str, mode: u8, trans: u8) -> DeviceResult<()>;

    /// Finds a device by its spec, returning the device ID and state if found
    async fn find_device(&mut self, spec: &str) -> DeviceResult<Option<(usize, &mut DeviceState)>>;

    /// Gets a device by its ID
    fn get_device(&mut self, device_id: usize) -> Option<&mut DeviceState>;

    /// Closes a device by its ID
    async fn close_device(&mut self, device_id: usize) -> DeviceResult<bool>;

    /// Gets a network device by its ID
    fn get_network_device(&mut self, device_id: usize) -> Option<&mut Box<dyn NetworkDevice>>;
}

/// Concrete implementation of the NetworkManager trait
pub struct NetworkManagerImpl {
    device_manager: DeviceManager,
    protocol_factory: ProtocolFactory,
}

impl NetworkManagerImpl {
    pub fn new() -> Self {
        Self {
            device_manager: DeviceManager::new(),
            protocol_factory: ProtocolFactory::new(),
        }
    }
}

#[async_trait]
impl NetworkManager for NetworkManagerImpl {
    fn parse_device_spec(&self, spec: &str) -> DeviceResult<(usize, NetworkUrl)> {
        // Parse the network URL
        let url = NetworkUrl::parse(spec)?;

        // Get the device ID from the URL (N1-N8)
        let device_id = (url.unit - 1) as usize;
        if device_id >= 8 {
            return Err(DeviceError::InvalidDeviceId);
        }

        Ok((device_id, url))
    }

    // In NetworkManagerImpl::open_device
    async fn open_device(&mut self, spec: &str, mode: u8, trans: u8) -> DeviceResult<()> {
        let (device_id, url) = self.parse_device_spec(spec)?;

        // First set device state
        if !self.device_manager.set_device_state(device_id, mode, trans, url.clone()) {
            return Err(DeviceError::InvalidDeviceId);
        }

        // Then create/get protocol handler
        let protocol = NetworkProtocol::from_str(url.scheme()?).ok_or(DeviceError::UnsupportedProtocol)?;
        self.protocol_factory.get_or_create_device(device_id, protocol, &url.url).await?;

        Ok(())
    }

    async fn find_device(&mut self, spec: &str) -> DeviceResult<Option<(usize, &mut DeviceState)>> {
        let (device_id, _) = self.parse_device_spec(spec)?;
        
        if let Some(device) = self.device_manager.get_device(device_id) {
            // Get device from protocol factory to ensure it exists
            if let Some(_) = self.protocol_factory.get_device(device_id) {
                Ok(Some((device_id, device)))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn get_device(&mut self, device_id: usize) -> Option<&mut DeviceState> {
        self.device_manager.get_device(device_id)
    }

    async fn close_device(&mut self, device_id: usize) -> DeviceResult<bool> {
        // Close device in protocol factory
        self.protocol_factory.close_device(device_id).await?;
        
        // Clear device state
        if let Some(device) = self.device_manager.get_device(device_id) {
            device.url = None;
            device.mode = 0;
            device.trans = 0;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_network_device(&mut self, device_id: usize) -> Option<&mut Box<dyn NetworkDevice>> {
        // Get the device directly from protocol factory
        self.protocol_factory.get_device(device_id)
    }
} 