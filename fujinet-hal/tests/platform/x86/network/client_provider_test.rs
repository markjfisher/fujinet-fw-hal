use crate::device::network::protocols::HttpClientProvider;
use crate::platform::x86::network::X86HttpClientProvider;

#[test]
fn test_create_http_client() {
    let provider = X86HttpClientProvider::new();
    let client = provider.create_http_client();
    
    // Verify we can create a client
    assert!(client.as_any().is::<crate::platform::x86::network::X86HttpClient>());
}

#[test]
fn test_provider_is_send() {
    // This test verifies at compile time that the provider implements Send
    fn assert_send<T: Send>() {}
    assert_send::<X86HttpClientProvider>();
} 