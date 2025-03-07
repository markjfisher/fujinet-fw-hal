use async_trait::async_trait;
use crate::error::DeviceResult;
use crate::platform::network::NetworkDriver;

pub mod status;
pub mod network;

pub use network::NetworkDevice;
pub use status::DeviceStatus;

#[async_trait]
pub trait Device: Send + Sync {
    /// Returns the name of the device
    fn name(&self) -> &str;

    /// Opens the device
    async fn open(&mut self) -> DeviceResult<()>;

    /// Closes the device
    async fn close(&mut self) -> DeviceResult<()>;

    /// Reads bytes from the device
    async fn read_bytes(&mut self, buf: &mut [u8]) -> DeviceResult<usize>;

    /// Writes bytes to the device
    async fn write_bytes(&mut self, buf: &[u8]) -> DeviceResult<usize>;

    /// Reads a block of data from the device
    async fn read_block(&mut self, block: u32, buf: &mut [u8]) -> DeviceResult<usize>;

    /// Writes a block of data to the device
    async fn write_block(&mut self, block: u32, buf: &[u8]) -> DeviceResult<usize>;

    /// Gets the current status of the device
    async fn get_status(&self) -> DeviceResult<DeviceStatus>;

    fn set_network_driver(&mut self, driver: Box<dyn NetworkDriver>);

    fn get_network_driver(&mut self) -> Option<&mut dyn NetworkDriver>;
} 