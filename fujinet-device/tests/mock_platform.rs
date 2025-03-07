use async_trait::async_trait;
use fujinet_core::platform::network::NetworkDriver;
use fujinet_core::error::{DeviceError, DeviceResult};
use fujinet_core::device::Device;
use fujinet_device::network::new_network_device;
use std::any::Any;

pub struct MockNetworkDriver {
    connected: bool,
    write_buffer: Vec<u8>,
    read_buffer: Vec<u8>,
}

impl MockNetworkDriver {
    pub fn new() -> Self {
        Self {
            connected: false,
            write_buffer: Vec::new(),
            read_buffer: Vec::new(),
        }
    }

    // Test helper methods
    pub fn set_read_data(&mut self, data: Vec<u8>) {
        self.read_buffer = data;
    }

    pub fn get_written_data(&self) -> &[u8] {
        &self.write_buffer
    }
}

#[async_trait]
impl NetworkDriver for MockNetworkDriver {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    
    async fn connect(&mut self, _endpoint: &str) -> DeviceResult<()> {
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> DeviceResult<()> {
        self.connected = false;
        Ok(())
    }

    async fn write(&mut self, data: &[u8]) -> DeviceResult<usize> {
        if !self.connected {
            return Err(DeviceError::NotReady);
        }
        self.write_buffer.extend_from_slice(data);
        Ok(data.len())
    }

    async fn read(&mut self, buffer: &mut [u8]) -> DeviceResult<usize> {
        if !self.connected {
            return Err(DeviceError::NotReady);
        }
        let amount = buffer.len().min(self.read_buffer.len());
        if amount > 0 {
            buffer[..amount].copy_from_slice(&self.read_buffer[..amount]);
            self.read_buffer.drain(..amount);
        }
        Ok(amount)
    }
}

// Modify create_network_device to return both
pub fn create_network_device(endpoint: String) -> DeviceResult<Box<dyn Device>> {
    let mut device = new_network_device(endpoint)?;
    device.set_network_driver(Box::new(MockNetworkDriver::new()));
    Ok(device)
}
