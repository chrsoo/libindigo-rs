//! Error types for server discovery operations.

use std::fmt;

/// Errors that can occur during server discovery.
#[derive(Debug, Clone)]
pub enum DiscoveryError {
    /// Discovery feature is not enabled (requires `auto` feature flag).
    NotSupported(String),

    /// Failed to initialize mDNS browser.
    InitializationFailed(String),

    /// Discovery operation timed out.
    Timeout(String),

    /// No servers were discovered.
    NoServersFound,

    /// An error occurred during discovery.
    DiscoveryFailed(String),

    /// Platform-specific error (e.g., Avahi not installed on Linux).
    PlatformError(String),
}

impl fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiscoveryError::NotSupported(msg) => {
                write!(f, "Discovery not supported: {}", msg)
            }
            DiscoveryError::InitializationFailed(msg) => {
                write!(f, "Failed to initialize discovery: {}", msg)
            }
            DiscoveryError::Timeout(msg) => {
                write!(f, "Discovery timeout: {}", msg)
            }
            DiscoveryError::NoServersFound => {
                write!(f, "No INDIGO servers found")
            }
            DiscoveryError::DiscoveryFailed(msg) => {
                write!(f, "Discovery failed: {}", msg)
            }
            DiscoveryError::PlatformError(msg) => {
                write!(f, "Platform error: {}", msg)
            }
        }
    }
}

impl std::error::Error for DiscoveryError {}

// Convert DiscoveryError to IndigoError
impl From<DiscoveryError> for crate::error::IndigoError {
    fn from(err: DiscoveryError) -> Self {
        match err {
            DiscoveryError::NotSupported(msg) => {
                crate::error::IndigoError::NotSupported(msg)
            }
            DiscoveryError::InitializationFailed(msg) => {
                crate::error::IndigoError::ConnectionError(msg)
            }
            DiscoveryError::Timeout(msg) => {
                crate::error::IndigoError::Timeout(msg)
            }
            DiscoveryError::NoServersFound => {
                crate::error::IndigoError::Timeout(
                    "No INDIGO servers discovered".to_string()
                )
            }
            DiscoveryError::DiscoveryFailed(msg) => {
                crate::error::IndigoError::ConnectionError(msg)
            }
            DiscoveryError::PlatformError(msg) => {
                crate::error::IndigoError::ConnectionError(msg)
            }
        }
    }
}
