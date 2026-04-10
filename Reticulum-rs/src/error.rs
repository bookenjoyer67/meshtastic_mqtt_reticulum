use thiserror::Error;

#[derive(Error, Debug)]
pub enum RnsError {
    #[error("Out of memory")]
    OutOfMemory,
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("Incorrect signature")]
    IncorrectSignature,
    
    #[error("Incorrect hash")]
    IncorrectHash,
    
    #[error("Cryptographic error: {0}")]
    CryptoError(String),
    
    #[error("Packet error: {0}")]
    PacketError(String),
    
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Link closed: {0}")]
    LinkClosed(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Interface error: {0}")]
    InterfaceError(String),
    
    #[error("Destination error: {0}")]
    DestinationError(String),
    
    #[error("Transport error: {0}")]
    TransportError(String),
    
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),
    
    #[error("Operation not supported: {0}")]
    NotSupported(String),
}

impl RnsError {
    pub fn is_recoverable(&self) -> bool {
        match self {
            RnsError::ConnectionError(_) |
            RnsError::TimeoutError(_) |
            RnsError::NetworkError(_) => true,
            _ => false,
        }
    }
    
    pub fn is_fatal(&self) -> bool {
        match self {
            RnsError::OutOfMemory |
            RnsError::ResourceExhausted(_) => true,
            _ => false,
        }
    }
    
    // Helper methods for common error cases
    pub fn invalid_argument(msg: impl Into<String>) -> Self {
        RnsError::InvalidArgument(msg.into())
    }
    
    pub fn crypto_error(msg: impl Into<String>) -> Self {
        RnsError::CryptoError(msg.into())
    }
    
    pub fn packet_error(msg: impl Into<String>) -> Self {
        RnsError::PacketError(msg.into())
    }
    
    pub fn link_closed(msg: impl Into<String>) -> Self {
        RnsError::LinkClosed(msg.into())
    }
    
    pub fn connection_error(msg: impl Into<String>) -> Self {
        RnsError::ConnectionError(msg.into())
    }
    
    pub fn serialization_error(msg: impl Into<String>) -> Self {
        RnsError::SerializationError(msg.into())
    }
    
    pub fn deserialization_error(msg: impl Into<String>) -> Self {
        RnsError::DeserializationError(msg.into())
    }
    
    pub fn timeout_error(msg: impl Into<String>) -> Self {
        RnsError::TimeoutError(msg.into())
    }
    
    pub fn config_error(msg: impl Into<String>) -> Self {
        RnsError::ConfigError(msg.into())
    }
    
    pub fn network_error(msg: impl Into<String>) -> Self {
        RnsError::NetworkError(msg.into())
    }
    
    pub fn interface_error(msg: impl Into<String>) -> Self {
        RnsError::InterfaceError(msg.into())
    }
    
    pub fn destination_error(msg: impl Into<String>) -> Self {
        RnsError::DestinationError(msg.into())
    }
    
    pub fn transport_error(msg: impl Into<String>) -> Self {
        RnsError::TransportError(msg.into())
    }
    
    pub fn resource_exhausted(msg: impl Into<String>) -> Self {
        RnsError::ResourceExhausted(msg.into())
    }
    
    pub fn not_supported(msg: impl Into<String>) -> Self {
        RnsError::NotSupported(msg.into())
    }
}

// Convenience type alias for Result<T, RnsError>
pub type Result<T> = std::result::Result<T, RnsError>;
