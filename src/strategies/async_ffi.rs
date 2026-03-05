//! Async FFI-based strategy implementation.
//!
//! This module provides an async client strategy that wraps synchronous FFI calls
//! to the C INDIGO library in `tokio::task::spawn_blocking` for non-blocking operation.
//!
//! # Architecture
//!
//! The async FFI strategy uses:
//! - `tokio::task::spawn_blocking` to run synchronous FFI calls without blocking the async runtime
//! - `tokio::sync::mpsc` channels to convert C callbacks into async streams
//! - `Arc<Mutex<>>` for thread-safe shared state between async tasks and FFI callbacks
//!
//! # Example
//!
//! ```ignore
//! use libindigo::strategies::AsyncFfiStrategy;
//! use libindigo::client::ClientStrategy;
//!
//! #[tokio::main]
//! async fn main() -> libindigo::Result<()> {
//!     let mut strategy = AsyncFfiStrategy::new();
//!     strategy.connect("localhost:7624").await?;
//!     strategy.enumerate_properties(None).await?;
//!     Ok(())
//! }
//! ```

use crate::client::ClientStrategy;
use crate::error::{IndigoError, Result};
use crate::types::Property;
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::sync::mpsc;

#[cfg(feature = "ffi")]
use libindigo_sys::*;

/// Channel capacity for property event streams.
const PROPERTY_CHANNEL_CAPACITY: usize = 100;

/// Async FFI-based client strategy.
///
/// This strategy wraps synchronous FFI calls to the C INDIGO library in async operations,
/// allowing non-blocking interaction with INDIGO servers.
///
/// # Thread Safety
///
/// The strategy uses `Arc<Mutex<>>` internally to ensure thread-safe access to FFI state
/// from both async tasks and C callbacks.
#[cfg(feature = "ffi")]
pub struct AsyncFfiStrategy {
    /// Shared state between async tasks and FFI callbacks.
    inner: Arc<Mutex<AsyncFfiInner>>,
    /// Receiver for property update events.
    property_rx: Option<mpsc::Receiver<Property>>,
}

#[cfg(feature = "ffi")]
struct AsyncFfiInner {
    /// FFI client structure.
    client: Option<Box<indigo_client>>,
    /// Sender for property update events.
    property_tx: mpsc::Sender<Property>,
    /// Connection state.
    connected: bool,
}

#[cfg(feature = "ffi")]
impl AsyncFfiStrategy {
    /// Creates a new async FFI client strategy.
    ///
    /// This initializes the internal state and sets up channels for property events.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use libindigo::strategies::AsyncFfiStrategy;
    ///
    /// let strategy = AsyncFfiStrategy::new();
    /// ```
    pub fn new() -> Self {
        let (property_tx, property_rx) = mpsc::channel(PROPERTY_CHANNEL_CAPACITY);

        let inner = AsyncFfiInner {
            client: None,
            property_tx,
            connected: false,
        };

        AsyncFfiStrategy {
            inner: Arc::new(Mutex::new(inner)),
            property_rx: Some(property_rx),
        }
    }

    /// Returns a stream of property updates.
    ///
    /// This stream will yield property updates as they are received from the INDIGO server.
    /// The stream can only be obtained once; subsequent calls will return `None`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use futures::StreamExt;
    /// use libindigo::strategies::AsyncFfiStrategy;
    ///
    /// let mut strategy = AsyncFfiStrategy::new();
    /// let mut stream = strategy.property_stream().unwrap();
    ///
    /// while let Some(property) = stream.next().await {
    ///     println!("Property update: {}.{}", property.device, property.name);
    /// }
    /// ```
    pub fn property_stream(&mut self) -> Option<PropertyStream> {
        self.property_rx.take().map(PropertyStream::new)
    }
}

#[cfg(feature = "ffi")]
impl Default for AsyncFfiStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "ffi")]
#[async_trait]
impl ClientStrategy for AsyncFfiStrategy {
    /// Connects to an INDIGO server at the specified URL.
    ///
    /// This method wraps the synchronous FFI connection call in `spawn_blocking`
    /// to avoid blocking the async runtime.
    ///
    /// # Arguments
    ///
    /// * `url` - Server URL in the format "host:port" (e.g., "localhost:7624")
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The URL format is invalid
    /// - The connection to the server fails
    /// - Already connected to a server
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use libindigo::strategies::AsyncFfiStrategy;
    /// # use libindigo::client::ClientStrategy;
    /// # #[tokio::main]
    /// # async fn main() -> libindigo::Result<()> {
    /// let mut strategy = AsyncFfiStrategy::new();
    /// strategy.connect("localhost:7624").await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn connect(&mut self, url: &str) -> Result<()> {
        // Parse URL
        let parts: Vec<&str> = url.split(':').collect();
        if parts.len() != 2 {
            return Err(IndigoError::InvalidParameter(format!(
                "Invalid URL format: {}. Expected 'host:port'",
                url
            )));
        }

        let host = parts[0].to_string();
        let port: u16 = parts[1].parse().map_err(|_| {
            IndigoError::InvalidParameter(format!("Invalid port number: {}", parts[1]))
        })?;

        // Check if already connected
        {
            let inner = self.inner.lock().unwrap();
            if inner.connected {
                return Err(IndigoError::InvalidState(
                    "Already connected to a server".to_string(),
                ));
            }
        }

        let inner = Arc::clone(&self.inner);

        // Perform connection in blocking task
        tokio::task::spawn_blocking(move || {
            let mut inner = inner.lock().unwrap();

            // TODO: Phase 2 - Implement actual FFI connection
            // This is a placeholder that will be implemented with:
            // 1. Initialize indigo_client structure
            // 2. Set up callbacks (attach, define_property, update_property, detach)
            // 3. Call indigo_attach_client
            // 4. Call indigo_connect_server with host and port

            // For now, just mark as connected for testing
            inner.connected = true;

            Ok::<(), IndigoError>(())
        })
        .await
        .map_err(|e| IndigoError::ConnectionError(format!("Task join error: {}", e)))??;

        Ok(())
    }

    /// Disconnects from the INDIGO server.
    ///
    /// This method wraps the synchronous FFI disconnection call in `spawn_blocking`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Not currently connected to a server
    /// - The disconnection fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use libindigo::strategies::AsyncFfiStrategy;
    /// # use libindigo::client::ClientStrategy;
    /// # #[tokio::main]
    /// # async fn main() -> libindigo::Result<()> {
    /// let mut strategy = AsyncFfiStrategy::new();
    /// strategy.connect("localhost:7624").await?;
    /// strategy.disconnect().await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn disconnect(&mut self) -> Result<()> {
        // Check if connected
        {
            let inner = self.inner.lock().unwrap();
            if !inner.connected {
                return Err(IndigoError::InvalidState(
                    "Not connected to a server".to_string(),
                ));
            }
        }

        let inner = Arc::clone(&self.inner);

        // Perform disconnection in blocking task
        tokio::task::spawn_blocking(move || {
            let mut inner = inner.lock().unwrap();

            // TODO: Phase 2 - Implement actual FFI disconnection
            // This will include:
            // 1. Call indigo_disconnect_server
            // 2. Call indigo_detach_client
            // 3. Clean up client structure

            inner.connected = false;

            Ok::<(), IndigoError>(())
        })
        .await
        .map_err(|e| IndigoError::ConnectionError(format!("Task join error: {}", e)))??;

        Ok(())
    }

    /// Requests enumeration of properties from the server.
    ///
    /// This method wraps the synchronous FFI property enumeration call in `spawn_blocking`.
    /// Property updates will be delivered via the property stream.
    ///
    /// # Arguments
    ///
    /// * `device` - Optional device name to enumerate properties for.
    ///              If `None`, enumerates properties for all devices.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Not currently connected to a server
    /// - The enumeration request fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use libindigo::strategies::AsyncFfiStrategy;
    /// # use libindigo::client::ClientStrategy;
    /// # #[tokio::main]
    /// # async fn main() -> libindigo::Result<()> {
    /// let mut strategy = AsyncFfiStrategy::new();
    /// strategy.connect("localhost:7624").await?;
    ///
    /// // Enumerate all properties
    /// strategy.enumerate_properties(None).await?;
    ///
    /// // Enumerate properties for a specific device
    /// strategy.enumerate_properties(Some("CCD Simulator")).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn enumerate_properties(&mut self, device: Option<&str>) -> Result<()> {
        // Check if connected
        {
            let inner = self.inner.lock().unwrap();
            if !inner.connected {
                return Err(IndigoError::InvalidState(
                    "Not connected to a server".to_string(),
                ));
            }
        }

        let device = device.map(|s| s.to_string());
        let inner = Arc::clone(&self.inner);

        // Perform enumeration in blocking task
        tokio::task::spawn_blocking(move || {
            let _inner = inner.lock().unwrap();

            // TODO: Phase 2 - Implement actual FFI property enumeration
            // This will include:
            // 1. Create indigo_property structure for enumeration request
            // 2. Call indigo_enumerate_properties with device filter
            // 3. Properties will be delivered via callbacks to the property stream

            let _device_name = device;

            Ok::<(), IndigoError>(())
        })
        .await
        .map_err(|e| IndigoError::ProtocolError(format!("Task join error: {}", e)))??;

        Ok(())
    }

    /// Sends a property update to the server.
    ///
    /// This method wraps the synchronous FFI property send call in `spawn_blocking`.
    ///
    /// # Arguments
    ///
    /// * `property` - The property to send
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Not currently connected to a server
    /// - The property send fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use libindigo::strategies::AsyncFfiStrategy;
    /// # use libindigo::client::ClientStrategy;
    /// # use libindigo::types::{Property, PropertyType, PropertyState, PropertyPerm};
    /// # #[tokio::main]
    /// # async fn main() -> libindigo::Result<()> {
    /// let mut strategy = AsyncFfiStrategy::new();
    /// strategy.connect("localhost:7624").await?;
    ///
    /// let property = Property::builder()
    ///     .device("CCD Simulator")
    ///     .name("CONNECTION")
    ///     .property_type(PropertyType::Switch)
    ///     .build();
    ///
    /// strategy.send_property(property).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn send_property(&mut self, property: Property) -> Result<()> {
        // Check if connected
        {
            let inner = self.inner.lock().unwrap();
            if !inner.connected {
                return Err(IndigoError::InvalidState(
                    "Not connected to a server".to_string(),
                ));
            }
        }

        let inner = Arc::clone(&self.inner);

        // Perform send in blocking task
        tokio::task::spawn_blocking(move || {
            let _inner = inner.lock().unwrap();

            // TODO: Phase 2 - Implement actual FFI property send
            // This will include:
            // 1. Convert Property to indigo_property FFI structure
            // 2. Call indigo_change_property or appropriate FFI function
            // 3. Handle response

            let _prop = property;

            Ok::<(), IndigoError>(())
        })
        .await
        .map_err(|e| IndigoError::ProtocolError(format!("Task join error: {}", e)))??;

        Ok(())
    }
}

/// Stream of property updates from the INDIGO server.
///
/// This stream yields [`Property`] updates as they are received from the server.
/// It wraps a `tokio::sync::mpsc::Receiver` and implements the `Stream` trait.
///
/// # Example
///
/// ```ignore
/// use futures::StreamExt;
/// use libindigo::strategies::AsyncFfiStrategy;
///
/// # #[tokio::main]
/// # async fn main() -> libindigo::Result<()> {
/// let mut strategy = AsyncFfiStrategy::new();
/// let mut stream = strategy.property_stream().unwrap();
///
/// strategy.connect("localhost:7624").await?;
/// strategy.enumerate_properties(None).await?;
///
/// while let Some(property) = stream.next().await {
///     println!("Received property: {}.{}", property.device, property.name);
/// }
/// # Ok(())
/// # }
/// ```
pub struct PropertyStream {
    receiver: mpsc::Receiver<Property>,
}

impl PropertyStream {
    fn new(receiver: mpsc::Receiver<Property>) -> Self {
        PropertyStream { receiver }
    }
}

impl Stream for PropertyStream {
    type Item = Property;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}

// Stub implementation when ffi-strategy feature is not enabled
#[cfg(not(feature = "ffi"))]
pub struct AsyncFfiStrategy;

#[cfg(not(feature = "ffi"))]
impl AsyncFfiStrategy {
    pub fn new() -> Self {
        AsyncFfiStrategy
    }
}

#[cfg(not(feature = "ffi"))]
impl Default for AsyncFfiStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "ffi"))]
#[async_trait]
impl ClientStrategy for AsyncFfiStrategy {
    async fn connect(&mut self, _url: &str) -> Result<()> {
        Err(IndigoError::NotSupported(
            "FFI strategy not available (compile with 'ffi-strategy' feature)".to_string(),
        ))
    }

    async fn disconnect(&mut self) -> Result<()> {
        Err(IndigoError::NotSupported(
            "FFI strategy not available (compile with 'ffi-strategy' feature)".to_string(),
        ))
    }

    async fn enumerate_properties(&mut self, _device: Option<&str>) -> Result<()> {
        Err(IndigoError::NotSupported(
            "FFI strategy not available (compile with 'ffi-strategy' feature)".to_string(),
        ))
    }

    async fn send_property(&mut self, _property: Property) -> Result<()> {
        Err(IndigoError::NotSupported(
            "FFI strategy not available (compile with 'ffi-strategy' feature)".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_ffi_strategy_creation() {
        let _strategy = AsyncFfiStrategy::new();
    }

    #[cfg(feature = "ffi")]
    #[tokio::test]
    async fn test_connect_invalid_url() {
        let mut strategy = AsyncFfiStrategy::new();
        let result = strategy.connect("invalid").await;
        assert!(result.is_err());
    }

    #[cfg(feature = "ffi")]
    #[tokio::test]
    async fn test_disconnect_when_not_connected() {
        let mut strategy = AsyncFfiStrategy::new();
        let result = strategy.disconnect().await;
        assert!(result.is_err());
    }

    #[cfg(feature = "ffi")]
    #[tokio::test]
    async fn test_enumerate_when_not_connected() {
        let mut strategy = AsyncFfiStrategy::new();
        let result = strategy.enumerate_properties(None).await;
        assert!(result.is_err());
    }
}
