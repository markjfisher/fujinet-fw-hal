use super::HttpClient;

/// Trait for creating platform-specific HTTP clients
pub trait HttpClientProvider: Send {
    /// Creates a new HTTP client
    fn create_http_client(&self) -> Box<dyn HttpClient>;
} 