pub mod http;
pub mod protocol_handler;
pub mod http_client;
pub mod protocol_registry;
pub mod client_provider;
pub mod factory;

// Re-export public items
pub use protocol_handler::{ProtocolHandler, ConnectionStatus};
pub use http_client::HttpClient;
pub use http::HttpProtocol;
pub use protocol_registry::{NetworkProtocol, is_protocol_supported};
pub use client_provider::HttpClientProvider;
pub use factory::ProtocolFactory;
// pub use https::HttpsProtocol;
