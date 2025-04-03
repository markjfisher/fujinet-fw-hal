use crate::device::{DeviceError, DeviceResult};
use async_trait::async_trait;

#[async_trait]
pub trait ProtocolHandler: Send + Sync + std::any::Any {
    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
    
    /// Convert to Any for mutable downcasting
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    /// Open a connection to the endpoint
    async fn open(&mut self, endpoint: &str) -> DeviceResult<()>;
    
    /// Close the connection
    async fn close(&mut self) -> DeviceResult<()>;
    
    /// Read data from the connection
    /// Returns the number of bytes read
    async fn read(&mut self, buf: &mut [u8]) -> DeviceResult<usize>;
    
    /// Write data to the connection
    /// Returns the number of bytes written
    async fn write(&mut self, buf: &[u8]) -> DeviceResult<usize>;
    
    /// Get the current status of the connection
    async fn status(&self) -> DeviceResult<ConnectionStatus>;
    
    /// Get the number of bytes available to read
    async fn available(&self) -> DeviceResult<usize>;
}

#[derive(Debug, PartialEq, Clone, Default)]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Error(DeviceError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    // Mock protocol for testing the trait
    struct MockProtocol {
        state: Arc<Mutex<MockState>>,
    }

    #[derive(Default)]
    struct MockState {
        is_connected: bool,
        endpoint: String,
        data: Vec<u8>,
        read_pos: usize,
    }

    impl MockProtocol {
        fn new() -> Self {
            Self {
                state: Arc::new(Mutex::new(MockState::default())),
            }
        }
    }

    #[async_trait]
    impl ProtocolHandler for MockProtocol {
        fn as_any(&self) -> &dyn std::any::Any { self }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
        
        async fn open(&mut self, endpoint: &str) -> DeviceResult<()> {
            let mut state = self.state.lock().unwrap();
            state.endpoint = endpoint.to_string();
            state.is_connected = true;
            Ok(())
        }

        async fn close(&mut self) -> DeviceResult<()> {
            let mut state = self.state.lock().unwrap();
            state.is_connected = false;
            Ok(())
        }

        async fn read(&mut self, buf: &mut [u8]) -> DeviceResult<usize> {
            let state = self.state.lock().unwrap();
            if !state.is_connected {
                return Err(DeviceError::NotReady);
            }
            
            let remaining = state.data.len() - state.read_pos;
            let to_read = std::cmp::min(remaining, buf.len());
            if to_read > 0 {
                buf[..to_read].copy_from_slice(&state.data[state.read_pos..state.read_pos + to_read]);
            }
            Ok(to_read)
        }

        async fn write(&mut self, buf: &[u8]) -> DeviceResult<usize> {
            let mut state = self.state.lock().unwrap();
            if !state.is_connected {
                return Err(DeviceError::NotReady);
            }
            state.data.extend_from_slice(buf);
            Ok(buf.len())
        }

        async fn status(&self) -> DeviceResult<ConnectionStatus> {
            let state = self.state.lock().unwrap();
            Ok(if state.is_connected {
                ConnectionStatus::Connected
            } else {
                ConnectionStatus::Disconnected
            })
        }

        async fn available(&self) -> DeviceResult<usize> {
            let state = self.state.lock().unwrap();
            if !state.is_connected {
                return Err(DeviceError::NotReady);
            }
            Ok(state.data.len() - state.read_pos)
        }
    }

    #[tokio::test]
    async fn test_protocol_lifecycle() {
        let mut protocol = MockProtocol::new();
        
        // Test initial state
        assert!(matches!(protocol.status().await.unwrap(), ConnectionStatus::Disconnected));
        
        // Test open
        protocol.open("test://endpoint").await.unwrap();
        assert!(matches!(protocol.status().await.unwrap(), ConnectionStatus::Connected));
        
        // Test write and read
        let test_data = b"Hello, World!";
        let written = protocol.write(test_data).await.unwrap();
        assert_eq!(written, test_data.len());
        
        let mut read_buf = vec![0; test_data.len()];
        let read = protocol.read(&mut read_buf).await.unwrap();
        assert_eq!(read, test_data.len());
        assert_eq!(&read_buf, test_data);
        
        // Test close
        protocol.close().await.unwrap();
        assert!(matches!(protocol.status().await.unwrap(), ConnectionStatus::Disconnected));
        
        // Test operations after close
        assert!(matches!(protocol.write(b"test").await, Err(DeviceError::NotReady)));
        assert!(matches!(protocol.read(&mut read_buf).await, Err(DeviceError::NotReady)));
        assert!(matches!(protocol.available().await, Err(DeviceError::NotReady)));
    }

    #[tokio::test]
    async fn test_protocol_any_casting() {
        let protocol = MockProtocol::new();
        let any_ref = protocol.as_any();
        assert!(any_ref.is::<MockProtocol>());
    }
} 