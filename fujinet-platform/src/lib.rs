pub mod network;
mod platform;

pub use network::http_client::create_http_client;
pub use platform::Platform; 