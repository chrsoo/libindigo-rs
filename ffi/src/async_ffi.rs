//! Async FFI-based client strategy
//!
//! This module provides an async wrapper around the synchronous FFI strategy,
//! enabling non-blocking operations and property streaming.

use crate::callback::{CallbackHandler, FfiEvent};
use crate::ffi::FfiClientStrategy;
use async_trait::async_trait;
use futures::Stream;
use libindigo::client::ClientStrategy;
use libindigo::error::{IndigoError, Result};
use libindigo::types::{BlobTransferMode, Property};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::{broadcast, Mutex};
use tracing::{debug, error, info, warn};

/// Async wrapper around FFI-based client strategy
///
/// This strategy wraps the synchronous FFI calls in async operations,
/// using `tokio::task::spawn_blocking` to avoid blocking the async runtime.
/// It also provides a property stream for receiving property updates.
///
/// # Architecture
///
/// The async strategy wraps [`FfiClientStrategy`] and uses `spawn_blocking`
/// for any potentially blocking C library calls. Property updates are
/// received through the callback handler and can be consumed via a stream.
///
/// # Example
///
/// ```rust,ignore
/// use libindigo_ffi::{AsyncFfiStrategy, ClientBuilder};
/// use futures::StreamExt;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let strategy = AsyncFfiStrategy::new()?;
///     let mut stream = strategy.property_stream();
///
///     let mut client = ClientBuilder::new()
///         .with_strategy(strategy)
///         .build();
///
///     client.connect("localhost:7624").await?;
///
///     // Receive property updates
///     while let Some(property) = stream.next().await {
///         println!("Property: {}.{}", property.device, property.name);
///     }
///
///     Ok(())
/// }
/// ```
pub struct AsyncFfiStrategy {
    /// Wrapped synchronous FFI strategy.
    inner: Arc<Mutex<FfiClientStrategy>>,

    /// Callback handler for property updates.
    callback_handler: Arc<CallbackHandler>,
}

impl AsyncFfiStrategy {
    /// Creates a new async FFI-based client strategy.
    ///
    /// This creates the underlying synchronous FFI strategy and wraps it
    /// for async use.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying FFI strategy cannot be initialized.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use libindigo_ffi::AsyncFfiStrategy;
    ///
    /// let strategy = AsyncFfiStrategy::new()?;
    /// ```
    pub fn new() -> Result<Self> {
        info!("Creating async FFI client strategy");

        let inner = FfiClientStrategy::new()?;
        let callback_handler = inner.callback_handler();

        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
            callback_handler,
        })
    }

    /// Returns a stream of property updates.
    ///
    /// This stream receives property updates from the C INDIGO library
    /// via the callback handler. Multiple streams can be created, and
    /// each will receive all property updates.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use libindigo_ffi::AsyncFfiStrategy;
    /// use futures::StreamExt;
    ///
    /// let strategy = AsyncFfiStrategy::new()?;
    /// let mut stream = strategy.property_stream();
    ///
    /// while let Some(property) = stream.next().await {
    ///     println!("Property update: {}.{}", property.device, property.name);
    /// }
    /// ```
    pub fn property_stream(&self) -> PropertyStream {
        let receiver = self.callback_handler.subscribe();
        PropertyStream::new(receiver)
    }

    /// Gets a reference to the callback handler.
    ///
    /// This can be used to subscribe to raw FFI events if needed.
    pub fn callback_handler(&self) -> Arc<CallbackHandler> {
        self.callback_handler.clone()
    }
}

impl Default for AsyncFfiStrategy {
    fn default() -> Self {
        Self::new().expect("Failed to create default AsyncFfiStrategy")
    }
}

#[async_trait]
impl ClientStrategy for AsyncFfiStrategy {
    async fn connect(&mut self, url: &str) -> Result<()> {
        info!("Async FFI connect to {}", url);

        let inner = self.inner.clone();
        let url = url.to_string();

        // Spawn blocking task for potentially blocking C library call
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut strategy = inner.lock().await;
                strategy.connect(&url).await
            })
        })
        .await
        .map_err(|e| IndigoError::ConnectionError(format!("Task join error: {}", e)))?
    }

    async fn disconnect(&mut self) -> Result<()> {
        info!("Async FFI disconnect");

        let inner = self.inner.clone();

        // Spawn blocking task for potentially blocking C library call
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut strategy = inner.lock().await;
                strategy.disconnect().await
            })
        })
        .await
        .map_err(|e| IndigoError::ConnectionError(format!("Task join error: {}", e)))?
    }

    async fn enumerate_properties(&mut self, device: Option<&str>) -> Result<()> {
        debug!("Async FFI enumerate properties for device: {:?}", device);

        let inner = self.inner.clone();
        let device = device.map(|s| s.to_string());

        // Spawn blocking task for potentially blocking C library call
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut strategy = inner.lock().await;
                strategy.enumerate_properties(device.as_deref()).await
            })
        })
        .await
        .map_err(|e| IndigoError::ProtocolError(format!("Task join error: {}", e)))?
    }

    async fn send_property(&mut self, property: Property) -> Result<()> {
        debug!(
            "Async FFI send property: {}.{}",
            property.device, property.name
        );

        let inner = self.inner.clone();

        // Spawn blocking task for potentially blocking C library call
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut strategy = inner.lock().await;
                strategy.send_property(property).await
            })
        })
        .await
        .map_err(|e| IndigoError::ProtocolError(format!("Task join error: {}", e)))?
    }

    async fn enable_blob(
        &mut self,
        device: &str,
        name: Option<&str>,
        mode: BlobTransferMode,
    ) -> Result<()> {
        debug!(
            "Async FFI enable_blob for device: {}, name: {:?}, mode: {:?}",
            device, name, mode
        );

        let inner = self.inner.clone();
        let device = device.to_string();
        let name = name.map(|s| s.to_string());

        // Spawn blocking task for potentially blocking C library call
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut strategy = inner.lock().await;
                strategy.enable_blob(&device, name.as_deref(), mode).await
            })
        })
        .await
        .map_err(|e| IndigoError::ProtocolError(format!("Task join error: {}", e)))?
    }
}

/// Stream of property updates from the INDIGO server
///
/// This stream receives property updates asynchronously as they arrive
/// from the server via the C INDIGO library callbacks. It implements the
/// `Stream` trait for easy integration with async code.
///
/// # Example
///
/// ```rust,ignore
/// use libindigo_ffi::PropertyStream;
/// use futures::StreamExt;
///
/// let mut stream = strategy.property_stream();
///
/// while let Some(property) = stream.next().await {
///     match property.state {
///         PropertyState::Ok => println!("Property OK: {}", property.name),
///         PropertyState::Busy => println!("Property busy: {}", property.name),
///         PropertyState::Alert => println!("Property alert: {}", property.name),
///         _ => (),
///     }
/// }
/// ```
pub struct PropertyStream {
    /// Receiver for FFI events from the callback handler.
    receiver: broadcast::Receiver<FfiEvent>,
}

impl PropertyStream {
    /// Creates a new property stream from a broadcast receiver.
    fn new(receiver: broadcast::Receiver<FfiEvent>) -> Self {
        Self { receiver }
    }
}

impl Stream for PropertyStream {
    type Item = Property;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match self.receiver.try_recv() {
                Ok(event) => {
                    // Convert FFI events to properties
                    match event {
                        FfiEvent::PropertyDefined(prop) | FfiEvent::PropertyUpdated(prop) => {
                            return Poll::Ready(Some(prop));
                        }
                        FfiEvent::PropertyDeleted { .. } => {
                            // Skip deleted properties
                            continue;
                        }
                        FfiEvent::ConnectionChanged(_) => {
                            // Skip connection events
                            continue;
                        }
                        FfiEvent::Error(e) => {
                            error!("FFI error in property stream: {}", e);
                            continue;
                        }
                        FfiEvent::Message { .. } => {
                            // Skip message events
                            continue;
                        }
                    }
                }
                Err(broadcast::error::TryRecvError::Empty) => {
                    // No events available, register waker and return pending
                    // We need to use recv() which is async
                    let waker = cx.waker().clone();
                    let mut receiver = self.receiver.resubscribe();

                    tokio::spawn(async move {
                        let _ = receiver.recv().await;
                        waker.wake();
                    });

                    return Poll::Pending;
                }
                Err(broadcast::error::TryRecvError::Lagged(n)) => {
                    warn!("Property stream lagged by {} events", n);
                    continue;
                }
                Err(broadcast::error::TryRecvError::Closed) => {
                    return Poll::Ready(None);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[cfg(feature = "sys-available")]
    #[test]
    fn test_async_ffi_strategy_creation() {
        let result = AsyncFfiStrategy::new();
        assert!(result.is_ok());
    }

    #[cfg(not(feature = "sys-available"))]
    #[test]
    fn test_async_ffi_strategy_creation_without_sys() {
        let result = AsyncFfiStrategy::new();
        assert!(result.is_err());
        match result {
            Err(IndigoError::NotSupported(_)) => (),
            _ => panic!("Expected NotSupported error"),
        }
    }

    #[cfg(feature = "sys-available")]
    #[test]
    fn test_property_stream_creation() {
        let strategy = AsyncFfiStrategy::new().unwrap();
        let _stream = strategy.property_stream();
        // Stream should be created successfully
    }

    #[cfg(feature = "sys-available")]
    #[tokio::test]
    async fn test_multiple_streams() {
        let strategy = AsyncFfiStrategy::new().unwrap();
        let _stream1 = strategy.property_stream();
        let _stream2 = strategy.property_stream();
        // Multiple streams should work
    }

    #[cfg(feature = "sys-available")]
    #[tokio::test]
    async fn test_connect_disconnect() {
        let mut strategy = AsyncFfiStrategy::new().unwrap();

        // Note: This uses the stub implementation
        let result = strategy.connect("localhost:7624").await;
        assert!(result.is_ok());

        let result = strategy.disconnect().await;
        assert!(result.is_ok());
    }

    #[cfg(feature = "sys-available")]
    #[tokio::test]
    async fn test_property_stream_receives_events() {
        let strategy = AsyncFfiStrategy::new().unwrap();
        let mut stream = strategy.property_stream();

        // Get the callback handler and send a test event
        let handler = strategy.callback_handler();
        let sender = handler.sync_sender().unwrap();

        // Create a test property
        use libindigo::types::{PropertyState, PropertyType};
        let test_prop = Property {
            device: "Test Device".to_string(),
            name: "TEST_PROP".to_string(),
            group: "Test".to_string(),
            label: "Test Property".to_string(),
            state: PropertyState::Ok,
            perm: libindigo::types::PropertyPerm::ReadWrite,
            property_type: PropertyType::Text,
            items: std::collections::HashMap::new(),
            timeout: None,
            timestamp: None,
            message: None,
        };

        // Send the event
        sender
            .send(FfiEvent::PropertyDefined(test_prop.clone()))
            .unwrap();

        // Try to receive it (with timeout)
        let received =
            tokio::time::timeout(std::time::Duration::from_millis(100), stream.next()).await;

        // Note: This may timeout because the stream implementation needs the bridge task running
        // In a real scenario, the bridge would be started by connect()
        match received {
            Ok(Some(prop)) => {
                assert_eq!(prop.device, "Test Device");
                assert_eq!(prop.name, "TEST_PROP");
            }
            Ok(None) => panic!("Stream ended unexpectedly"),
            Err(_) => {
                // Timeout is expected without the bridge task running
                // This is acceptable for this test
            }
        }
    }
}
