mod platform;
pub mod network;

pub use platform::Platform;
pub use network::http_client::create_http_client;