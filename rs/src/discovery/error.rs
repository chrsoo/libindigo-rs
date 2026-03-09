//! Error types for server discovery operations.

use std::time::Duration;
use thiserror::Error;

/// Errors that can occur during server discovery.
#[derive(Debug, Clone, Error)]
pub enum DiscoveryError {
    /// mDNS daemon error occurred.
    #[error("mDNS daemon error: {0}")]
    MdnsError(String),

    /// Failed to initialize mDNS browser.
    #[error("Failed to initialize discovery: {0}")]
    InitializationFailed(String),

    /// Discovery operation timed out.
    #[error("Discovery timeout after {0:?}")]
    Timeout(Duration),

    /// Service registration failed.
    #[error("Service registration failed: {0}")]
    RegistrationFailed(String),

    /// Discovery not started.
    #[error("Discovery not started")]
    NotStarted,

    /// No servers were discovered.
    #[error("No INDIGO servers found")]
    NoServersFound,

    /// An error occurred during discovery.
    #[error("Discovery failed: {0}")]
    DiscoveryFailed(String),

    /// Platform-specific error (e.g., network unavailable).
    #[error("Platform error: {0}")]
    PlatformError(String),

    /// IO error occurred.
    #[error("IO error: {0}")]
    Io(String),
}

impl From<std::io::Error> for DiscoveryError {
    fn from(err: std::io::Error) -> Self {
        DiscoveryError::Io(err.to_string())
    }
}
