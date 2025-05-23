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
        println!("NetworkUrl::parse() called with spec: {}", spec);

        // Parse N: prefix and unit number (case insensitive)
        if !spec.to_uppercase().starts_with('N') {
            println!("NetworkUrl::parse() failed: missing N prefix");
            return Err(DeviceError::InvalidUrl);
        }

        // Extract unit number (N1-N8, or just N for N1)
        let (unit, rest) = if spec.len() > 1 && spec.chars().nth(1).unwrap().is_ascii_digit() {
            let unit = spec.chars().nth(1).unwrap().to_digit(10).unwrap() as u8;
            println!("NetworkUrl::parse() explicit unit: {}", unit);
            // Validate unit is in range 1-8
            if unit < 1 || unit > 8 {
                println!("NetworkUrl::parse() failed: invalid unit number");
                return Err(DeviceError::InvalidUrl);
            }
            (unit, &spec[2..])
        } else {
            println!("NetworkUrl::parse() using default unit: 1");
            (1, &spec[1..])
        };

        // Must start with colon
        if !rest.starts_with(':') {
            println!("NetworkUrl::parse() failed: missing colon after unit");
            return Err(DeviceError::InvalidUrl);
        }

        let url = rest[1..].to_string();
        println!("NetworkUrl::parse() url: {}", url);
        
        // Parse and validate protocol from URL scheme
        let scheme = Self::extract_scheme(&url)?;
        println!("NetworkUrl::parse() scheme: {}", scheme);
        
        let protocol = NetworkProtocol::from_str(scheme)
            .ok_or(DeviceError::UnsupportedProtocol)?;
        println!("NetworkUrl::parse() protocol: {:?}", protocol);

        Ok(Self { url, unit, protocol })
    }

    fn extract_scheme(url: &str) -> DeviceResult<&str> {
        // URL must contain "://" to be valid
        if !url.contains("://") {
            println!("extract_scheme() failed: missing ://");
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

    /// Check if two URLs have the same base (host and port)
    pub fn has_same_base_url(&self, other: &NetworkUrl) -> bool {
        // Extract base URL (scheme, host, port) by removing path and query
        let self_base = self.url.split("://").nth(1).unwrap_or("").split('/').next().unwrap_or("");
        let other_base = other.url.split("://").nth(1).unwrap_or("").split('/').next().unwrap_or("");
        
        // Also check that protocols match
        self.protocol == other.protocol && self_base == other_base
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

    #[test]
    fn test_has_same_base_url() {
        let base = NetworkUrl::parse("N1:http://192.168.1.100:8085/").unwrap();
        let same_base = NetworkUrl::parse("N1:http://192.168.1.100:8085/get?a=1&b=2").unwrap();
        let same_base_no_slash = NetworkUrl::parse("N1:http://192.168.1.100:8085").unwrap();
        let different_base = NetworkUrl::parse("N1:http://192.168.1.101:8085/").unwrap();
        let different_port = NetworkUrl::parse("N1:http://192.168.1.100:8086/").unwrap();

        assert!(base.has_same_base_url(&same_base), "URLs with same base but different paths should match");
        assert!(base.has_same_base_url(&same_base_no_slash), "URLs with/without trailing slash should match");
        assert!(!base.has_same_base_url(&different_base), "URLs with different hosts should not match");
        assert!(!base.has_same_base_url(&different_port), "URLs with different ports should not match");
    }
} 