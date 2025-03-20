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

    pub async fn get_or_create_device(
        &mut self, 
        device_id: usize,
        protocol: NetworkProtocol,
        url: &str
    ) -> DeviceResult<&mut Box<dyn NetworkDevice>> {
        // If we already have an active device
        if let Some(device) = &mut self.active_devices[device_id] {
            // For now, assume existing device matches protocol
            return Ok(device);
        }

        // Create new device based on protocol
        let device = match protocol {
            NetworkProtocol::Http => {
                new_network_device(url.to_string())?
            }
        };

        // First insert the device
        self.active_devices[device_id] = Some(device);
        
        // Then get a mutable reference to it
        match &mut self.active_devices[device_id] {
            Some(device) => Ok(device),
            None => unreachable!("Device was just inserted")
        }
    }

    pub async fn close_device(&mut self, device_id: usize) -> DeviceResult<()> {
        if let Some(device) = &mut self.active_devices[device_id] {
            device.disconnect().await?;
            self.active_devices[device_id] = None;
        }
        Ok(())
    }
}
