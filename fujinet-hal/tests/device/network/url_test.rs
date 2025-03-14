use fujinet_hal::device::network::NetworkUrl;
use fujinet_hal::device::DeviceError;

#[test]
fn test_network_url_parse_default_unit() {
    // Test uppercase N:
    let url = NetworkUrl::parse("N:http://example.com").unwrap();
    assert_eq!(url.unit, 1);
    assert_eq!(url.url, "http://example.com");

    // Test lowercase n:
    let url = NetworkUrl::parse("n:http://example.com").unwrap();
    assert_eq!(url.unit, 1);
    assert_eq!(url.url, "http://example.com");
}

#[test]
fn test_network_url_parse_specific_unit() {
    // Test uppercase N with unit
    let url = NetworkUrl::parse("N2:http://example.com").unwrap();
    assert_eq!(url.unit, 2);
    assert_eq!(url.url, "http://example.com");

    // Test lowercase n with unit
    let url = NetworkUrl::parse("n3:http://example.com").unwrap();
    assert_eq!(url.unit, 3);
    assert_eq!(url.url, "http://example.com");

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
        NetworkUrl::parse("http://example.com"),
        Err(DeviceError::InvalidProtocol)
    ));

    // Test invalid unit number (0)
    assert!(matches!(
        NetworkUrl::parse("N0:http://example.com"),
        Err(DeviceError::InvalidProtocol)
    ));

    // Test invalid unit number (9)
    assert!(matches!(
        NetworkUrl::parse("N9:http://example.com"),
        Err(DeviceError::InvalidProtocol)
    ));

    // Test invalid unit number (lowercase)
    assert!(matches!(
        NetworkUrl::parse("n0:http://example.com"),
        Err(DeviceError::InvalidProtocol)
    ));

    // Test malformed network indicator
    assert!(matches!(
        NetworkUrl::parse("Nx:http://example.com"),
        Err(DeviceError::InvalidProtocol)
    ));

    // Test missing colon
    assert!(matches!(
        NetworkUrl::parse("N1http://example.com"),
        Err(DeviceError::InvalidProtocol)
    ));
}

#[test]
fn test_network_url_scheme() {
    // Test HTTP
    let url = NetworkUrl::parse("N:http://example.com").unwrap();
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