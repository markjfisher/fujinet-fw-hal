pub mod protocols;
mod network_device;

pub use protocols::http::{HttpProtocol, HttpProtocolHandler};
pub use network_device::{NetworkDevice, NetworkDeviceImpl, new_network_device};