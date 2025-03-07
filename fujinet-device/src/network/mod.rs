pub mod protocols;

use fujinet_core::error::DeviceResult;
use fujinet_core::device::Device;
use fujinet_core::platform::network::NetworkDriver;
use async_trait::async_trait;

pub use protocols::{ProtocolHandler, ConnectionStatus, AnyProtocolHandler};
pub use protocols::http::{HttpProtocol, HttpProtocolHandler};

pub struct NetworkDevice<P: ProtocolHandler> {
    endpoint: String,
    protocol: Option<P>,
}

impl<P: ProtocolHandler> NetworkDevice<P> {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            protocol: None,
        }
    }
}

#[async_trait]
impl<P: ProtocolHandler> Device for NetworkDevice<P> {
    fn name(&self) -> &str {
        "network"
    }

    fn set_network_driver(&mut self, driver: Box<dyn NetworkDriver>) {
        if let Some(protocol) = &mut self.protocol {
            protocol.set_network_driver(driver);
        }
    }

    fn get_network_driver(&mut self) -> Option<&mut dyn NetworkDriver> {
        if let Some(protocol) = &mut self.protocol {
            protocol.get_network_driver()
        } else {
            None
        }
    }

    async fn open(&mut self) -> DeviceResult<()> {
        println!("Opening device with protocol: {:?}", self.protocol.is_some());  // Debug
        if let Some(protocol) = &mut self.protocol {
            protocol.open(&self.endpoint).await
        } else {
            println!("No protocol found!");  // Debug
            Err(fujinet_core::error::DeviceError::NotReady)
        }
    }

    async fn close(&mut self) -> DeviceResult<()> {
        if let Some(protocol) = &mut self.protocol {
            protocol.close().await
        } else {
            Err(fujinet_core::error::DeviceError::NotReady)
        }
    }

    async fn read_bytes(&mut self, buf: &mut [u8]) -> DeviceResult<usize> {
        if let Some(protocol) = &mut self.protocol {
            protocol.read(buf).await
        } else {
            Err(fujinet_core::error::DeviceError::NotReady)
        }
    }

    async fn write_bytes(&mut self, buf: &[u8]) -> DeviceResult<usize> {
        if let Some(protocol) = &mut self.protocol {
            protocol.write(buf).await
        } else {
            Err(fujinet_core::error::DeviceError::NotReady)
        }
    }

    async fn read_block(&mut self, _block: u32, _buf: &mut [u8]) -> DeviceResult<usize> {
        Err(fujinet_core::error::DeviceError::NotSupported)
    }

    async fn write_block(&mut self, _block: u32, _buf: &[u8]) -> DeviceResult<usize> {
        Err(fujinet_core::error::DeviceError::NotSupported)
    }

    async fn get_status(&self) -> DeviceResult<fujinet_core::device::DeviceStatus> {
        if let Some(protocol) = &self.protocol {
            match protocol.status().await? {
                ConnectionStatus::Connected => Ok(fujinet_core::device::DeviceStatus::Ready),
                ConnectionStatus::Disconnected => Ok(fujinet_core::device::DeviceStatus::Disconnected),
                ConnectionStatus::Connecting => Ok(fujinet_core::device::DeviceStatus::Disconnected),
                ConnectionStatus::Error(_) => Ok(fujinet_core::device::DeviceStatus::Error),
            }
        } else {
            Ok(fujinet_core::device::DeviceStatus::Disconnected)
        }
    }
}

// Factory function to create the right device type based on URL
pub fn new_network_device(endpoint: String) -> DeviceResult<Box<dyn Device>> {
    let scheme = endpoint.split("://").next().ok_or(fujinet_core::error::DeviceError::InvalidProtocol)?;
    println!("Creating device for scheme: {}", scheme);  // Debug
    
    match scheme {
        "http" | "https" => {
            let mut device = NetworkDevice::<HttpProtocol>::new(endpoint);
            device.protocol = Some(HttpProtocol::default());  // Create protocol here
            println!("Created HTTP protocol");  // Debug
            Ok(Box::new(device))
        },
        // Add more protocols here as they are implemented
        _ => Err(fujinet_core::error::DeviceError::InvalidProtocol),
    }
}