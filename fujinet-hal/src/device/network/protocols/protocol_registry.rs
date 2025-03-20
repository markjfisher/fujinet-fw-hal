#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NetworkProtocol {
    Http, // Represents both HTTP and HTTPS
    // Add other protocols as needed
}

impl NetworkProtocol {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "http" | "https" => Some(Self::Http),
            _ => None,
        }
    }
}

pub fn is_protocol_supported(protocol_str: &str) -> bool {
    NetworkProtocol::from_str(protocol_str).is_some()
}