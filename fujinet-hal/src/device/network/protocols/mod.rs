pub mod http;
pub mod protocol_handler;
pub mod http_client;
pub mod protocol_registry;
pub mod client_provider;

// Re-export public items
pub use protocol_handler::{ProtocolHandler, ConnectionStatus};
pub use http_client::HttpClient;
pub use http::HttpProtocol;
pub use protocol_registry::{NetworkProtocol, register_protocol, get_supported_protocols, is_protocol_supported};
pub use client_provider::HttpClientProvider;
