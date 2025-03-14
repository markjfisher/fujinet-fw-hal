use crate::device::DeviceResult;
use crate::device::DeviceError;

/// Represents a parsed network URL with unit number
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkUrl {
    /// The network unit number (1-8)
    pub unit: u8,
    /// The actual URL without the N[x]: prefix
    pub url: String,
}

impl NetworkUrl {
    /// Parse a network URL of the form N[x]:protocol://...
    /// where x is an optional unit number 1-8 (defaults to 1)
    /// The network indicator (N) is case-insensitive
    pub fn parse(url: &str) -> DeviceResult<Self> {
        // Check if URL starts with N: or Nx: where x is 1-8 (case insensitive)
        if let Some(rest) = url.strip_prefix("N:").or_else(|| url.strip_prefix("n:")) {
            Ok(Self {
                unit: 1,
                url: rest.to_string(),
            })
        } else if (url.starts_with('N') || url.starts_with('n')) 
            && url.len() >= 3 
            && url.chars().nth(1).unwrap().is_ascii_digit() 
            && url.chars().nth(2) == Some(':') {
            let unit = url.chars().nth(1).unwrap().to_digit(10).unwrap() as u8;
            if unit == 0 || unit > 8 {
                return Err(DeviceError::InvalidProtocol);
            }
            Ok(Self {
                unit,
                url: url[3..].to_string(),
            })
        } else {
            Err(DeviceError::InvalidProtocol)
        }
    }

    /// Get the protocol scheme from the URL (e.g., "http", "https", "tcp")
    /// Returns InvalidProtocol error if the URL doesn't contain "://"
    pub fn scheme(&self) -> DeviceResult<&str> {
        if !self.url.contains("://") {
            return Err(DeviceError::InvalidProtocol);
        }
        self.url.split("://").next().ok_or(DeviceError::InvalidProtocol)
    }
} 