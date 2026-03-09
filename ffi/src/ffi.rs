//! Synchronous FFI-based client strategy
//!
//! This module provides a safe wrapper around the C INDIGO library,
//! implementing the `ClientStrategy` trait for use with the core API.

use crate::callback::CallbackHandler;
use crate::conversion::{blob_mode_to_c_str, string_to_c_string};
use async_trait::async_trait;
use libindigo::client::ClientStrategy;
use libindigo::error::{IndigoError, Result};
use libindigo::types::{BlobTransferMode, Property};
use std::sync::{Arc, Mutex as StdMutex};
use tracing::{debug, error, info, warn};

// Conditional compilation for sys types
#[cfg(feature = "sys-available")]
use libindigo_sys::*;

/// FFI-based client strategy using the C INDIGO library
///
/// This strategy wraps the official C INDIGO library, providing maximum
/// compatibility with INDIGO servers while exposing a safe Rust API.
///
/// # Thread Safety
///
/// The underlying C library may not be thread-safe. This implementation
/// uses internal synchronization to ensure safe concurrent access.
///
/// # Platform Support
///
/// This strategy requires the `libindigo-sys` crate to be built successfully,
/// which in turn requires the INDIGO C library. On platforms where the C
/// library is not available, all methods will return `NotSupported` errors.
///
/// # Example
///
/// ```rust,ignore
/// use libindigo_ffi::{FfiClientStrategy, ClientBuilder};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let strategy = FfiClientStrategy::new()?;
///     let mut client = ClientBuilder::new()
///         .with_strategy(strategy)
///         .build();
///
///     client.connect("localhost:7624").await?;
///     // Use client...
///     Ok(())
/// }
/// ```
pub struct FfiClientStrategy {
    /// Callback handler for property updates from C library.
    callback_handler: Arc<CallbackHandler>,

    /// Connection state.
    connected: Arc<StdMutex<bool>>,

    /// C client handle (only available when sys-available feature is enabled).
    /// Wrapped in a Send-safe wrapper.
    #[cfg(feature = "sys-available")]
    client_handle: Arc<StdMutex<Option<SendPtr<indigo_client>>>>,

    /// Placeholder when sys crate is not available.
    #[cfg(not(feature = "sys-available"))]
    _phantom: std::marker::PhantomData<()>,
}

impl FfiClientStrategy {
    /// Creates a new FFI-based client strategy.
    ///
    /// This initializes the callback handler and prepares the strategy for
    /// connection to an INDIGO server.
    ///
    /// # Errors
    ///
    /// Returns an error if the C INDIGO library cannot be initialized.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use libindigo_ffi::FfiClientStrategy;
    ///
    /// let strategy = FfiClientStrategy::new()?;
    /// ```
    #[cfg(feature = "sys-available")]
    pub fn new() -> Result<Self> {
        info!("Creating FFI client strategy");

        let callback_handler = Arc::new(CallbackHandler::new());

        Ok(Self {
            callback_handler,
            connected: Arc::new(StdMutex::new(false)),
            client_handle: Arc::new(StdMutex::new(None)),
        })
    }

    /// Stub implementation when sys crate is not available.
    #[cfg(not(feature = "sys-available"))]
    pub fn new() -> Result<Self> {
        warn!("FFI strategy not available - sys crate not built");
        Err(IndigoError::NotSupported(
            "FFI strategy not available - sys crate not built".to_string(),
        ))
    }

    /// Gets a reference to the callback handler.
    ///
    /// This can be used to subscribe to property update events.
    pub fn callback_handler(&self) -> Arc<CallbackHandler> {
        self.callback_handler.clone()
    }

    /// Checks if currently connected.
    pub fn is_connected(&self) -> bool {
        *self.connected.lock().unwrap()
    }

    // Internal helper methods

    #[cfg(feature = "sys-available")]
    fn parse_url(url: &str) -> Result<(String, u16)> {
        let parts: Vec<&str> = url.split(':').collect();
        if parts.len() != 2 {
            return Err(IndigoError::InvalidParameter(format!(
                "Invalid URL format: {}. Expected host:port",
                url
            )));
        }

        let host = parts[0].to_string();
        let port = parts[1]
            .parse::<u16>()
            .map_err(|e| IndigoError::InvalidParameter(format!("Invalid port number: {}", e)))?;

        Ok((host, port))
    }
}

impl Default for FfiClientStrategy {
    fn default() -> Self {
        // Try to create a new strategy, but if it fails (e.g., sys not available),
        // we need to handle it. For Default trait, we'll panic if creation fails.
        Self::new().expect("Failed to create default FfiClientStrategy")
    }
}

#[async_trait]
impl ClientStrategy for FfiClientStrategy {
    #[cfg(feature = "sys-available")]
    async fn connect(&mut self, url: &str) -> Result<()> {
        info!("FFI connect to {}", url);

        let mut connected = self.connected.lock().unwrap();
        if *connected {
            return Err(IndigoError::InvalidState("Already connected".to_string()));
        }

        let (_host, _port) = Self::parse_url(url)?;

        // TODO: Implement actual C library connection
        // This would involve:
        // 1. Allocating indigo_client structure
        // 2. Setting up callback function pointers
        // 3. Calling indigo_server_connect()
        // 4. Starting the callback handler bridge task

        warn!("FFI connect not fully implemented - using stub");

        // Start the callback handler bridge
        let handler = self.callback_handler.clone();
        tokio::spawn(async move {
            if let Err(e) = handler.start().await {
                error!("Callback handler failed: {}", e);
            }
        });

        *connected = true;
        Ok(())
    }

    #[cfg(not(feature = "sys-available"))]
    async fn connect(&mut self, _url: &str) -> Result<()> {
        Err(IndigoError::NotSupported(
            "FFI not available - sys crate not built".to_string(),
        ))
    }

    #[cfg(feature = "sys-available")]
    async fn disconnect(&mut self) -> Result<()> {
        info!("FFI disconnect");

        let mut connected = self.connected.lock().unwrap();
        if !*connected {
            return Err(IndigoError::InvalidState("Not connected".to_string()));
        }

        // TODO: Implement actual C library disconnection
        // This would involve:
        // 1. Calling indigo_server_disconnect()
        // 2. Freeing the indigo_client structure
        // 3. Stopping the callback handler

        warn!("FFI disconnect not fully implemented - using stub");

        *connected = false;
        Ok(())
    }

    #[cfg(not(feature = "sys-available"))]
    async fn disconnect(&mut self) -> Result<()> {
        Err(IndigoError::NotSupported(
            "FFI not available - sys crate not built".to_string(),
        ))
    }

    #[cfg(feature = "sys-available")]
    async fn enumerate_properties(&mut self, device: Option<&str>) -> Result<()> {
        debug!("FFI enumerate properties for device: {:?}", device);

        let connected = self.connected.lock().unwrap();
        if !*connected {
            return Err(IndigoError::InvalidState("Not connected".to_string()));
        }

        // TODO: Implement actual C library property enumeration
        // This would involve:
        // 1. Converting device name to C string (if provided)
        // 2. Calling indigo_enumerate_properties()
        // 3. Properties will arrive via callbacks

        warn!("FFI enumerate_properties not fully implemented - using stub");

        Ok(())
    }

    #[cfg(not(feature = "sys-available"))]
    async fn enumerate_properties(&mut self, _device: Option<&str>) -> Result<()> {
        Err(IndigoError::NotSupported(
            "FFI not available - sys crate not built".to_string(),
        ))
    }

    #[cfg(feature = "sys-available")]
    async fn send_property(&mut self, property: Property) -> Result<()> {
        debug!("FFI send property: {}.{}", property.device, property.name);

        let connected = self.connected.lock().unwrap();
        if !*connected {
            return Err(IndigoError::InvalidState("Not connected".to_string()));
        }

        // TODO: Implement actual C library property sending
        // This would involve:
        // 1. Converting Rust Property to C indigo_property
        // 2. Calling appropriate indigo_change_*_property() function
        // 3. Freeing the C property structure

        warn!("FFI send_property not fully implemented - using stub");

        Ok(())
    }

    #[cfg(not(feature = "sys-available"))]
    async fn send_property(&mut self, _property: Property) -> Result<()> {
        Err(IndigoError::NotSupported(
            "FFI not available - sys crate not built".to_string(),
        ))
    }

    #[cfg(feature = "sys-available")]
    async fn enable_blob(
        &mut self,
        device: &str,
        name: Option<&str>,
        mode: BlobTransferMode,
    ) -> Result<()> {
        debug!(
            "FFI enable_blob for device: {}, name: {:?}, mode: {:?}",
            device, name, mode
        );

        let connected = self.connected.lock().unwrap();
        if !*connected {
            return Err(IndigoError::InvalidState("Not connected".to_string()));
        }

        // Convert parameters to C strings
        let _device_c = string_to_c_string(device)?;
        let _name_c = name.map(string_to_c_string).transpose()?;
        let _mode_str = blob_mode_to_c_str(mode);

        // TODO: Implement actual C library BLOB enable
        // This would involve:
        // 1. Calling indigo_enable_blob() with the C strings
        // 2. Handling the result

        warn!("FFI enable_blob not fully implemented - using stub");

        Ok(())
    }

    #[cfg(not(feature = "sys-available"))]
    async fn enable_blob(
        &mut self,
        _device: &str,
        _name: Option<&str>,
        _mode: BlobTransferMode,
    ) -> Result<()> {
        Err(IndigoError::NotSupported(
            "FFI not available - sys crate not built".to_string(),
        ))
    }
}

// ============================================================================
// Safety and Memory Management
// ============================================================================

/// Send-safe wrapper for raw C pointers.
///
/// This wrapper allows raw pointers to be sent between threads safely.
/// The caller must ensure that the pointer is valid and that access is
/// properly synchronized.
#[cfg(feature = "sys-available")]
struct SendPtr<T>(*mut T);

#[cfg(feature = "sys-available")]
unsafe impl<T> Send for SendPtr<T> {}
#[cfg(feature = "sys-available")]
unsafe impl<T> Sync for SendPtr<T> {}

#[cfg(feature = "sys-available")]
impl<T> SendPtr<T> {
    fn new(ptr: *mut T) -> Self {
        SendPtr(ptr)
    }

    fn as_ptr(&self) -> *mut T {
        self.0
    }
}

/// RAII guard for C client handle.
///
/// Ensures that the C client is properly cleaned up when dropped.
#[cfg(feature = "sys-available")]
struct ClientHandleGuard {
    handle: SendPtr<indigo_client>,
}

#[cfg(feature = "sys-available")]
impl Drop for ClientHandleGuard {
    fn drop(&mut self) {
        if !self.handle.as_ptr().is_null() {
            // TODO: Call appropriate C cleanup function
            debug!("Cleaning up C client handle");
        }
    }
}

#[cfg(feature = "sys-available")]
unsafe impl Send for ClientHandleGuard {}
#[cfg(feature = "sys-available")]
unsafe impl Sync for ClientHandleGuard {}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "sys-available")]
    #[test]
    fn test_ffi_strategy_creation() {
        let result = FfiClientStrategy::new();
        assert!(result.is_ok());
    }

    #[cfg(not(feature = "sys-available"))]
    #[test]
    fn test_ffi_strategy_creation_without_sys() {
        let result = FfiClientStrategy::new();
        assert!(result.is_err());
        match result {
            Err(IndigoError::NotSupported(_)) => (),
            _ => panic!("Expected NotSupported error"),
        }
    }

    #[cfg(feature = "sys-available")]
    #[test]
    fn test_url_parsing() {
        let (host, port) = FfiClientStrategy::parse_url("localhost:7624").unwrap();
        assert_eq!(host, "localhost");
        assert_eq!(port, 7624);

        let result = FfiClientStrategy::parse_url("invalid");
        assert!(result.is_err());

        let result = FfiClientStrategy::parse_url("localhost:invalid");
        assert!(result.is_err());
    }

    #[cfg(feature = "sys-available")]
    #[tokio::test]
    async fn test_connect_disconnect() {
        let mut strategy = FfiClientStrategy::new().unwrap();
        assert!(!strategy.is_connected());

        // Note: This will fail with stub implementation
        // Full test would require a mock C library
        let result = strategy.connect("localhost:7624").await;
        // We expect it to succeed with the stub
        assert!(result.is_ok());
        assert!(strategy.is_connected());

        let result = strategy.disconnect().await;
        assert!(result.is_ok());
        assert!(!strategy.is_connected());
    }

    #[cfg(feature = "sys-available")]
    #[tokio::test]
    async fn test_double_connect_fails() {
        let mut strategy = FfiClientStrategy::new().unwrap();
        strategy.connect("localhost:7624").await.unwrap();

        let result = strategy.connect("localhost:7624").await;
        assert!(result.is_err());
        match result {
            Err(IndigoError::InvalidState(_)) => (),
            _ => panic!("Expected InvalidState error"),
        }
    }

    #[cfg(feature = "sys-available")]
    #[tokio::test]
    async fn test_disconnect_when_not_connected_fails() {
        let mut strategy = FfiClientStrategy::new().unwrap();
        let result = strategy.disconnect().await;
        assert!(result.is_err());
    }
}
