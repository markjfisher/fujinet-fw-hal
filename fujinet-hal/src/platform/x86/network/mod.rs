mod http_client;
mod manager;

pub use http_client::{X86HttpClient, DefaultHttpClientProvider};
pub use manager::get_network_manager;