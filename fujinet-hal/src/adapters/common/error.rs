use crate::device::DeviceError;

/// Common error types for device operations
#[derive(Debug)]
pub enum AdapterError {
    /// The device specification was invalid
    InvalidDeviceSpec,
    /// The requested mode was invalid
    InvalidMode,
    /// The translation setting was invalid
    InvalidTranslation,
    /// An error occurred while operating the device
    DeviceError(DeviceError),
}

impl From<DeviceError> for AdapterError {
    fn from(err: DeviceError) -> Self {
        AdapterError::DeviceError(err)
    }
}