/// Common request structure for opening a network device
#[derive(Debug)]
pub struct DeviceOpenRequest {
    /// The device specification string (e.g. "N1:http://ficticious_example.madeup")
    pub device_spec: String,
    /// The mode for opening the device
    pub mode: u8,
    /// The translation setting
    pub translation: u8,
}

/// Common request structure for HTTP POST operations
#[derive(Debug)]
pub struct HttpPostRequest {
    /// The device spec string (only used at adapter layer)
    pub device_spec: String,
    /// The device ID for internal operations
    pub device_id: Option<usize>,
    /// The data to POST
    pub data: Vec<u8>,
}

/// Common request structure for HTTP GET operations
#[derive(Debug)]
pub struct HttpGetRequest {
    /// The device spec string (only used at adapter layer)
    pub device_spec: String,
    /// The device ID for internal operations
    pub device_id: Option<usize>,
    /// The buffer to store the response
    pub buffer: Vec<u8>,
}

impl HttpGetRequest {
    pub fn new(device_spec: String, buffer: Vec<u8>) -> Self {
        Self {
            device_spec,
            device_id: None,
            buffer,
        }
    }
}

impl HttpPostRequest {
    pub fn new(device_spec: String, data: Vec<u8>) -> Self {
        Self {
            device_spec,
            device_id: None,
            data,
        }
    }
} 