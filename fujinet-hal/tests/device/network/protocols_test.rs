use fujinet_hal::device::network::protocols::{is_protocol_supported, NetworkProtocol, register_protocol};

#[test]
fn test_supported_protocols() {
    // Test built-in protocols
    assert!(is_protocol_supported("http"));
    assert!(is_protocol_supported("https"));
    assert!(!is_protocol_supported("ftp"));
    assert!(!is_protocol_supported("invalid"));

    // Test case insensitivity
    assert!(is_protocol_supported("HTTP"));
    assert!(is_protocol_supported("HTTPS"));
}

#[test]
fn test_protocol_registration() {
    // Register a new protocol
    register_protocol(NetworkProtocol::Http); // Re-registering HTTP should be fine
    assert!(is_protocol_supported("http"));

    // Test that unsupported protocols remain unsupported
    assert!(!is_protocol_supported("ftp"));
    assert!(!is_protocol_supported("smtp"));
    assert!(!is_protocol_supported("invalid"));
} 