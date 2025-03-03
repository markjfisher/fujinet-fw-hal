use thiserror::Error;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum DeviceError {
    #[error("I/O error: {0}")]
    Io(String),
    #[error("Device not ready")]
    NotReady,
    #[error("Operation not supported")]
    NotSupported,
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
}

pub type DeviceResult<T> = Result<T, DeviceError>; 