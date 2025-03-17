pub mod protocols;
pub mod manager;
mod network_device;
mod url;

pub use protocols::http::{HttpProtocol, HttpProtocolHandler};
pub use network_device::{NetworkDevice, NetworkDeviceImpl, new_network_device};
pub use url::NetworkUrl;
pub use protocols::*;
pub use manager::{NetworkManager, get_network_manager}; 