//! Error types for the libindigo crate.
//!
//! This module provides a comprehensive error type hierarchy using `thiserror`
//! for idiomatic Rust error handling.

use thiserror::Error;

/// The main error type for libindigo operations.
///
/// This enum covers all possible error conditions that can occur when
/// interacting with INDIGO devices and servers.
#[derive(Error, Debug)]
pub enum IndigoError {
    /// Connection to INDIGO server failed.
    #[error("Connection failed: {0}")]
    ConnectionError(String),

    /// INDIGO protocol error occurred.
    #[error("Protocol error: {0}")]
    ProtocolError(String),

    /// FFI-related error (only available with `ffi-strategy` feature).
    #[cfg(feature = "ffi")]
    #[error("FFI error: {0}")]
    FfiError(String),

    /// I/O error occurred during network or file operations.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Requested property was not found.
    #[error("Property not found: {0}")]
    PropertyNotFound(String),

    /// Device was not found.
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    /// Invalid state transition or operation in current state.
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Failed to parse data (XML, values, etc.).
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Timeout occurred during operation.
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Invalid parameter or argument provided.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Operation not supported by current strategy or configuration.
    #[error("Not supported: {0}")]
    NotSupported(String),
}

/// A specialized `Result` type for libindigo operations.
///
/// This type is used throughout the libindigo API as a convenient alias
/// for `Result<T, IndigoError>`.
pub type Result<T> = std::result::Result<T, IndigoError>;
