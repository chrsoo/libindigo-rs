//! Error types for server discovery operations.

use std::fmt;

/// Errors that can occur during server discovery.
#[derive(Debug, Clone)]
pub enum DiscoveryError {
    /// Failed to initialize mDNS browser.
    InitializationFailed(String),

    /// Discovery operation timed out.
    Timeout(String),

    /// No servers were discovered.
    NoServersFound,

    /// An error occurred during discovery.
    DiscoveryFailed(String),

    /// Platform-specific error (e.g., network unavailable).
    PlatformError(String),
}

impl fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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
