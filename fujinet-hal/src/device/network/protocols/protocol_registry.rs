use std::collections::HashSet;
use once_cell::sync::Lazy;
use std::sync::RwLock;

/// Represents a supported network protocol
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NetworkProtocol {
    Http,
    Https,
}

impl NetworkProtocol {
    /// Convert a string to a NetworkProtocol
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "http" => Some(Self::Http),
            "https" => Some(Self::Https),
            _ => None,
        }
    }

    /// Convert a NetworkProtocol to a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
        }
    }
}

/// Registry of supported network protocols
#[derive(Default)]
struct ProtocolRegistry {
    protocols: HashSet<NetworkProtocol>,
}

impl ProtocolRegistry {
    fn new() -> Self {
        let mut registry = Self::default();
        // Register all supported protocols
        registry.register(NetworkProtocol::Http);
        registry.register(NetworkProtocol::Https);
        registry
    }

    fn register(&mut self, protocol: NetworkProtocol) {
        self.protocols.insert(protocol);
    }

    fn is_supported(&self, protocol: &NetworkProtocol) -> bool {
        self.protocols.contains(protocol)
    }

    fn get_supported_protocols(&self) -> Vec<NetworkProtocol> {
        self.protocols.iter().cloned().collect()
    }
}

// Global protocol registry
static PROTOCOL_REGISTRY: Lazy<RwLock<ProtocolRegistry>> = Lazy::new(|| RwLock::new(ProtocolRegistry::new()));

/// Check if a protocol string is supported
pub fn is_protocol_supported(protocol_str: &str) -> bool {
    if let Some(protocol) = NetworkProtocol::from_str(protocol_str) {
        PROTOCOL_REGISTRY.read().unwrap().is_supported(&protocol)
    } else {
        false
    }
}

/// Register a new protocol
pub fn register_protocol(protocol: NetworkProtocol) {
    PROTOCOL_REGISTRY.write().unwrap().register(protocol);
}

/// Get a list of all supported protocols
pub fn get_supported_protocols() -> Vec<NetworkProtocol> {
    PROTOCOL_REGISTRY.read().unwrap().get_supported_protocols()
} 