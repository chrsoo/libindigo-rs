//! Strategy trait for client implementations.
//!
//! This module defines the [`ClientStrategy`] trait that abstracts over different
//! client implementation strategies (FFI-based or pure Rust).

use crate::error::Result;
use crate::types::{BlobTransferMode, Property};
use async_trait::async_trait;

#[cfg(feature = "monitoring")]
use crate::client::monitoring::{ClientEvent, MonitoringConfig};
#[cfg(feature = "monitoring")]
use tokio::sync::mpsc;

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

    /// Enables or configures BLOB transfer mode for a device.
    ///
    /// This method sends an `enableBLOB` message to the server to control
    /// how BLOBs (Binary Large Objects) are transferred for the specified device.
    ///
    /// # Arguments
    ///
    /// * `device` - The device name to configure BLOB transfer for
    /// * `name` - Optional property name to limit BLOB configuration to a specific property.
    ///            If `None`, applies to all BLOB properties on the device.
    /// * `mode` - The BLOB transfer mode to use
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or if not connected.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use libindigo::types::BlobTransferMode;
    ///
    /// // Enable BLOB transfer for CCD images
    /// client.enable_blob("CCD Simulator", None, BlobTransferMode::Also).await?;
    ///
    /// // Disable BLOB transfer for a specific property
    /// client.enable_blob("CCD Simulator", Some("CCD_IMAGE"), BlobTransferMode::Never).await?;
    /// ```
    async fn enable_blob(
        &mut self,
        device: &str,
        name: Option<&str>,
        mode: BlobTransferMode,
    ) -> Result<()>;

    /// Sets the monitoring configuration for the strategy.
    ///
    /// This method is called by the ClientBuilder when monitoring is enabled.
    /// Strategies that support monitoring should store the configuration and
    /// start monitoring when `connect()` is called.
    ///
    /// # Arguments
    ///
    /// * `config` - The monitoring configuration to use
    ///
    /// # Default Implementation
    ///
    /// The default implementation does nothing, allowing strategies that don't
    /// support monitoring to ignore this call.
    #[cfg(feature = "monitoring")]
    fn set_monitoring_config(&mut self, _config: MonitoringConfig) {
        // Default: no-op for strategies that don't support monitoring
    }

    /// Subscribes to server status events.
    ///
    /// Returns a receiver for monitoring status change events. Each event indicates
    /// a change in server availability (Available, Degraded, or Unavailable).
    ///
    /// # Returns
    ///
    /// An `UnboundedReceiver` that will receive `ClientEvent` notifications when
    /// the server status changes. Returns `None` if monitoring is not enabled or
    /// not supported by the strategy.
    ///
    /// # Default Implementation
    ///
    /// The default implementation returns `None`, indicating that monitoring is
    /// not supported by this strategy.
    #[cfg(feature = "monitoring")]
    fn subscribe_status(&self) -> Option<mpsc::UnboundedReceiver<ClientEvent>> {
        None
    }

    // TODO: Phase 2 - Add property stream methods
    // fn property_stream(&self) -> PropertyStream;

    // TODO: Phase 2 - Add device enumeration
    // async fn enumerate_devices(&mut self) -> Result<Vec<Device>>;
}
