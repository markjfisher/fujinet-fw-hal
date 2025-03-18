use crate::device::network::protocols::HttpClientProvider;
use crate::device::network::protocols::HttpClient;
use super::X86HttpClient;

/// X86 platform implementation of HttpClientProvider
pub struct X86HttpClientProvider;

impl X86HttpClientProvider {
    pub fn new() -> Self {
        Self
    }
}

impl HttpClientProvider for X86HttpClientProvider {
    fn create_http_client(&self) -> Box<dyn HttpClient> {
        Box::new(X86HttpClient::default())
    }
} 