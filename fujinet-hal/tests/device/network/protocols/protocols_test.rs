use fujinet_hal::device::network::protocols::is_protocol_supported;

#[test]
fn test_supported_protocols() {
    // Test built-in protocols
    assert!(is_protocol_supported("http"));
    assert!(is_protocol_supported("https"));
    
    // Test unsupported protocols
    assert!(!is_protocol_supported("ftp"));
    assert!(!is_protocol_supported("smtp"));
    assert!(!is_protocol_supported("invalid"));

    // Test case insensitivity
    assert!(is_protocol_supported("HTTP"));
    assert!(is_protocol_supported("HTTPS"));
}
