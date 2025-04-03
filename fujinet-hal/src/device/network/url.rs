use crate::device::{DeviceResult, DeviceError};
use super::protocols::NetworkProtocol;

/// Represents a parsed network URL with unit number
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkUrl {
    /// The network unit number (1-8)
    pub unit: u8,
    /// The actual URL without the N[x]: prefix
    pub url: String,
    protocol: NetworkProtocol,
}

impl NetworkUrl {
    /// Parse a network URL of the form N[x]:protocol://...
    /// where x is an optional unit number 1-8 (defaults to 1)
    /// The network indicator (N) is case-insensitive
    pub fn parse(spec: &str) -> DeviceResult<Self> {
        // Parse N: prefix and unit number (case insensitive)
        if !spec.to_uppercase().starts_with('N') {
            return Err(DeviceError::InvalidUrl);
        }

        // Extract unit number (N1-N8, or just N for N1)
        let (unit, rest) = if spec.len() > 1 && spec.chars().nth(1).unwrap().is_ascii_digit() {
            let unit = spec.chars().nth(1).unwrap().to_digit(10).unwrap() as u8;
            // Validate unit is in range 1-8
            if unit < 1 || unit > 8 {
                return Err(DeviceError::InvalidUrl);
            }
            (unit, &spec[2..])
        } else {
            (1, &spec[1..])
        };

        // Must start with colon
        if !rest.starts_with(':') {
            return Err(DeviceError::InvalidUrl);
        }

        let url = rest[1..].to_string();

        // Parse and validate protocol from URL scheme
        let scheme = Self::extract_scheme(&url)?;
        let protocol = NetworkProtocol::from_str(scheme)
            .ok_or(DeviceError::UnsupportedProtocol)?;

        Ok(Self { url, unit, protocol })
    }

    fn extract_scheme(url: &str) -> DeviceResult<&str> {
        // URL must contain "://" to be valid
        if !url.contains("://") {
            return Err(DeviceError::InvalidUrl);
        }
        
        url.split("://")
            .next()
            .ok_or(DeviceError::InvalidUrl)
    }

    /// Get the protocol scheme from the URL (e.g., "http", "https", "tcp")
    /// Returns InvalidProtocol error if the URL doesn't contain "://"
    pub fn scheme(&self) -> DeviceResult<&str> {
        Self::extract_scheme(&self.url)
    }

    pub fn protocol(&self) -> NetworkProtocol {
        self.protocol.clone()
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

        // Test all valid unit numbers with valid protocol
        for unit in 1..=8 {
            let url = format!("N{}:http://test.com", unit);
            let parsed = NetworkUrl::parse(&url).unwrap();
            assert_eq!(parsed.unit, unit as u8);
            assert_eq!(parsed.url, "http://test.com");
        }
    }

    #[test]
    fn test_network_url_parse_invalid_cases() {
        // Test missing network indicator
        assert!(matches!(
            NetworkUrl::parse("http://ficticious_example.madeup"),
            Err(DeviceError::InvalidUrl)
        ));

        // Test invalid unit number (0)
        assert!(matches!(
            NetworkUrl::parse("N0:http://ficticious_example.madeup"),
            Err(DeviceError::InvalidUrl)
        ));

        // Test invalid unit number (9)
        assert!(matches!(
            NetworkUrl::parse("N9:http://ficticious_example.madeup"),
            Err(DeviceError::InvalidUrl)
        ));

        // Test invalid unit number (lowercase)
        assert!(matches!(
            NetworkUrl::parse("n0:http://ficticious_example.madeup"),
            Err(DeviceError::InvalidUrl)
        ));

        // Test malformed network indicator
        assert!(matches!(
            NetworkUrl::parse("Nx:http://ficticious_example.madeup"),
            Err(DeviceError::InvalidUrl)
        ));

        // Test missing colon
        assert!(matches!(
            NetworkUrl::parse("N1http://ficticious_example.madeup"),
            Err(DeviceError::InvalidUrl)
        ));

        // Test missing protocol scheme
        assert!(matches!(
            NetworkUrl::parse("N:example.com"),
            Err(DeviceError::InvalidUrl)
        ));

        // Test unsupported protocol
        assert!(matches!(
            NetworkUrl::parse("N:ftp://example.com"),
            Err(DeviceError::UnsupportedProtocol)
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

        // Test invalid URL (no scheme) - should return InvalidUrl error
        assert!(matches!(
            NetworkUrl::parse("N:example.com"),
            Err(DeviceError::InvalidUrl)
        ));
    }
} 