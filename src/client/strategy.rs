//! Strategy trait for client implementations.
//!
//! This module defines the [`ClientStrategy`] trait that abstracts over different
//! client implementation strategies (FFI-based or pure Rust).

use crate::error::Result;
use crate::types::Property;
use async_trait::async_trait;

/// Strategy trait for INDIGO client implementations.
///
/// This trait defines the interface that all client strategies must implement,
/// whether using FFI bindings to the C library or a pure Rust implementation.
///
/// # Async
///
/// All methods are async to support non-blocking I/O operations. Implementations
/// that wrap synchronous FFI calls should use `tokio::task::spawn_blocking` to
/// avoid blocking the async runtime.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to allow usage across async tasks.
///
/// # Example
///
/// ```ignore
/// use libindigo::client::ClientStrategy;
/// use async_trait::async_trait;
///
/// struct MyStrategy;
///
/// #[async_trait]
/// impl ClientStrategy for MyStrategy {
///     async fn connect(&mut self, url: &str) -> libindigo::Result<()> {
///         // Implementation
///         Ok(())
///     }
///
///     // ... other methods
/// }
/// ```
#[async_trait]
pub trait ClientStrategy: Send + Sync {
    /// Connects to an INDIGO server at the specified URL.
    ///
    /// # Arguments
    ///
    /// * `url` - Server URL in the format "host:port" (e.g., "localhost:7624")
    ///
    /// # Errors
    ///
    /// Returns an error if the connection fails.
    async fn connect(&mut self, url: &str) -> Result<()>;

    /// Disconnects from the INDIGO server.
    ///
    /// # Errors
    ///
    /// Returns an error if disconnection fails or if not currently connected.
    async fn disconnect(&mut self) -> Result<()>;

    /// Requests enumeration of properties from the server.
    ///
    /// # Arguments
    ///
    /// * `device` - Optional device name to enumerate properties for.
    ///              If `None`, enumerates properties for all devices.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    async fn enumerate_properties(&mut self, device: Option<&str>) -> Result<()>;

    /// Sends a property update to the server.
    ///
    /// # Arguments
    ///
    /// * `property` - The property to send
    ///
    /// # Errors
    ///
    /// Returns an error if sending fails.
    async fn send_property(&mut self, property: Property) -> Result<()>;

    // TODO: Phase 2 - Add property stream methods
    // fn property_stream(&self) -> PropertyStream;

    // TODO: Phase 2 - Add device enumeration
    // async fn enumerate_devices(&mut self) -> Result<Vec<Device>>;

    // TODO: Phase 2 - Add BLOB handling
    // async fn enable_blob(&mut self, device: &str, name: Option<&str>) -> Result<()>;
}
