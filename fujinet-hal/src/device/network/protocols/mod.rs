pub mod http;
pub mod protocol_handler;
pub mod http_client;
pub mod protocol_registry;

// Re-export public items
pub use protocol_handler::{ProtocolHandler, ConnectionStatus};
pub use http_client::HttpClient;
pub use http::HttpProtocol;
pub use protocol_registry::{NetworkProtocol, is_protocol_supported, register_protocol, get_supported_protocols};
