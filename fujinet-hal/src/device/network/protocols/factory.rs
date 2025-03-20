use super::NetworkProtocol;
use crate::device::network::{NetworkDevice, new_network_device};
use crate::device::DeviceResult;

pub struct ProtocolFactory {
    // Each device can have one active protocol handler
    active_devices: [Option<Box<dyn NetworkDevice>>; 8],
}

impl ProtocolFactory {
    pub fn new() -> Self {
        Self {
            active_devices: Default::default()
        }
    }

    // Returns device_id if successful
    pub async fn get_or_create_device(
        &mut self, 
        device_id: usize,
        protocol: NetworkProtocol,
        url: &str
    ) -> DeviceResult<usize> {
        // If we already have an active device
        if self.active_devices[device_id].is_some() {
            return Ok(device_id);
        }

        // Create new device based on protocol
        let device = match protocol {
            NetworkProtocol::Http => {
                new_network_device(url.to_string())?
            }
        };

        self.active_devices[device_id] = Some(device);
        Ok(device_id)
    }

    // Separate method to get device by ID
    pub fn get_device(&mut self, device_id: usize) -> Option<&mut Box<dyn NetworkDevice>> {
        self.active_devices[device_id].as_mut()
    }

    pub async fn close_device(&mut self, device_id: usize) -> DeviceResult<()> {
        if let Some(device) = &mut self.active_devices[device_id] {
            device.disconnect().await?;
            self.active_devices[device_id] = None;
        }
        Ok(())
    }
}
