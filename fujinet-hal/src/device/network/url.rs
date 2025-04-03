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
            // Default to unit 1 when no number specified
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_url_parse_default_unit() {
        // Test uppercase N:
        let url = NetworkUrl::parse("N:http://ficticious_example.madeup").unwrap();
        assert_eq!(url.unit, 1);
        assert_eq!(url.url, "http://ficticious_example.madeup");

        // Test lowercase n:
        let url = NetworkUrl::parse("n:http://ficticious_example.madeup").unwrap();
        assert_eq!(url.unit, 1);
        assert_eq!(url.url, "http://ficticious_example.madeup");
    }

    #[test]
    fn test_network_url_parse_specific_unit() {
        // Test uppercase N with unit
        let url = NetworkUrl::parse("N2:http://ficticious_example.madeup").unwrap();
        assert_eq!(url.unit, 2);
        assert_eq!(url.url, "http://ficticious_example.madeup");

        // Test lowercase n with unit
        let url = NetworkUrl::parse("n3:http://ficticious_example.madeup").unwrap();
        assert_eq!(url.unit, 3);
        assert_eq!(url.url, "http://ficticious_example.madeup");

        // Test all valid unit numbers
        for unit in 1..=8 {
            let url = format!("N{}:test", unit);
            let parsed = NetworkUrl::parse(&url).unwrap();
            assert_eq!(parsed.unit, unit as u8);
            assert_eq!(parsed.url, "test");
        }
    }

    #[test]
    fn test_network_url_parse_invalid_cases() {
        // Test missing network indicator
        assert!(matches!(
            NetworkUrl::parse("http://ficticious_example.madeup"),
            Err(DeviceError::InvalidProtocol)
        ));

        // Test invalid unit number (0)
        assert!(matches!(
            NetworkUrl::parse("N0:http://ficticious_example.madeup"),
            Err(DeviceError::InvalidProtocol)
        ));

        // Test invalid unit number (9)
        assert!(matches!(
            NetworkUrl::parse("N9:http://ficticious_example.madeup"),
            Err(DeviceError::InvalidProtocol)
        ));

        // Test invalid unit number (lowercase)
        assert!(matches!(
            NetworkUrl::parse("n0:http://ficticious_example.madeup"),
            Err(DeviceError::InvalidProtocol)
        ));

        // Test malformed network indicator
        assert!(matches!(
            NetworkUrl::parse("Nx:http://ficticious_example.madeup"),
            Err(DeviceError::InvalidProtocol)
        ));

        // Test missing colon
        assert!(matches!(
            NetworkUrl::parse("N1http://ficticious_example.madeup"),
            Err(DeviceError::InvalidProtocol)
        ));
    }

    #[test]
    fn test_network_url_scheme() {
        // Test HTTP
        let url = NetworkUrl::parse("N:http://ficticious_example.madeup").unwrap();
        assert_eq!(url.scheme().unwrap(), "http");

        // Test HTTPS
        let url = NetworkUrl::parse("N:https://example.com").unwrap();
        assert_eq!(url.scheme().unwrap(), "https");

        // Test TCP
        let url = NetworkUrl::parse("N:tcp://192.168.1.1:8080").unwrap();
        assert_eq!(url.scheme().unwrap(), "tcp");

        // Test invalid URL (no scheme)
        let url = NetworkUrl::parse("N:example.com").unwrap();
        assert!(url.scheme().is_err());
    }
} 