#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceStatus {
    Ready,
    Busy,
    Error,
    Disconnected,
} 