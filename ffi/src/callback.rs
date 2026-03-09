//! C callback handlers for INDIGO property updates.
//!
//! Translates C-style callbacks from the INDIGO library into Rust async channels
//! using the pattern: C callback → sync mpsc → async broadcast channel.
//!
//! # Architecture
//!
//! The C INDIGO library calls callbacks from its own threads. We cannot use
//! tokio channels directly in these callbacks because they may not be called
//! from within a tokio runtime context. Instead, we use:
//!
//! 1. **C callback** → Calls Rust function with raw pointers
//! 2. **Rust wrapper** → Converts C types to Rust, sends to `std::sync::mpsc`
//! 3. **Bridge task** → Forwards from sync mpsc to `tokio::sync::broadcast`
//! 4. **Consumers** → Subscribe to broadcast channel for property updates

use crate::conversion::property_from_c;
use libindigo::types::Property;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, warn};

#[cfg(feature = "async")]
use tokio::sync::broadcast;

/// Events received from the C INDIGO library via callbacks.
#[derive(Debug, Clone)]
pub enum FfiEvent {
    /// A property was defined (initial property definition from server).
    PropertyDefined(Property),

    /// A property was updated (property value changed).
    PropertyUpdated(Property),

    /// A property was deleted.
    PropertyDeleted {
        /// Device name.
        device: String,
        /// Property name.
        name: String,
    },

    /// Connection state changed.
    ConnectionChanged(bool),

    /// An error occurred in the C library.
    Error(String),

    /// A message was received from the server.
    Message {
        /// Device name (if applicable).
        device: Option<String>,
        /// Message text.
        message: String,
    },
}

/// Manages the callback bridge between C INDIGO and Rust.
///
/// This handler sets up the bridge from C callbacks through sync channels
/// to async broadcast channels that can be consumed by async Rust code.
///
/// # Example
///
/// ```ignore
/// use libindigo_ffi::callback::CallbackHandler;
///
/// let handler = CallbackHandler::new();
/// let mut receiver = handler.subscribe();
///
/// // Start the bridge task
/// tokio::spawn(async move {
///     handler.start().await.unwrap();
/// });
///
/// // Receive events
/// while let Ok(event) = receiver.recv().await {
///     match event {
///         FfiEvent::PropertyDefined(prop) => {
///             println!("Property defined: {}.{}", prop.device, prop.name);
///         }
///         _ => {}
///     }
/// }
/// ```
pub struct CallbackHandler {
    /// Sender for the sync mpsc channel (used by C callbacks).
    sync_tx: Arc<Mutex<Option<mpsc::Sender<FfiEvent>>>>,

    /// Receiver for the sync mpsc channel (consumed by bridge task).
    sync_rx: Arc<Mutex<Option<mpsc::Receiver<FfiEvent>>>>,

    /// Broadcast channel for async consumers (only with async feature).
    #[cfg(feature = "async")]
    broadcast_tx: broadcast::Sender<FfiEvent>,
}

impl CallbackHandler {
    /// Creates a new callback handler.
    ///
    /// The handler is created with a broadcast channel that can support
    /// multiple subscribers. The channel has a capacity of 1000 events.
    #[cfg(feature = "async")]
    pub fn new() -> Self {
        let (sync_tx, sync_rx) = mpsc::channel();
        let (broadcast_tx, _) = broadcast::channel(1000);

        Self {
            sync_tx: Arc::new(Mutex::new(Some(sync_tx))),
            sync_rx: Arc::new(Mutex::new(Some(sync_rx))),
            broadcast_tx,
        }
    }

    /// Creates a new callback handler (non-async version).
    #[cfg(not(feature = "async"))]
    pub fn new() -> Self {
        let (sync_tx, sync_rx) = mpsc::channel();

        Self {
            sync_tx: Arc::new(Mutex::new(Some(sync_tx))),
            sync_rx: Arc::new(Mutex::new(Some(sync_rx))),
        }
    }

    /// Gets a receiver for FFI events.
    ///
    /// This can be called multiple times to create multiple consumers.
    /// Each consumer will receive all events.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let handler = CallbackHandler::new();
    /// let mut rx1 = handler.subscribe();
    /// let mut rx2 = handler.subscribe();
    /// // Both receivers will get all events
    /// ```
    #[cfg(feature = "async")]
    pub fn subscribe(&self) -> broadcast::Receiver<FfiEvent> {
        self.broadcast_tx.subscribe()
    }

    /// Gets the sync sender for use by C callbacks.
    ///
    /// This is used internally by the FFI layer to send events from C callbacks.
    /// Also exposed for testing purposes.
    #[cfg(any(test, feature = "async"))]
    pub fn sync_sender(&self) -> Option<mpsc::Sender<FfiEvent>> {
        self.sync_tx.lock().unwrap().clone()
    }

    /// Gets the sync sender for use by C callbacks (internal only).
    #[cfg(not(any(test, feature = "async")))]
    pub(crate) fn sync_sender(&self) -> Option<mpsc::Sender<FfiEvent>> {
        self.sync_tx.lock().unwrap().clone()
    }

    /// Starts the bridge task that forwards from sync mpsc to async broadcast.
    ///
    /// This should be called once and will run until the sync sender is dropped
    /// (which happens when the FFI client disconnects).
    ///
    /// # Errors
    ///
    /// Returns an error if the bridge task cannot be started or if the
    /// sync receiver has already been taken.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let handler = CallbackHandler::new();
    /// tokio::spawn(async move {
    ///     if let Err(e) = handler.start().await {
    ///         eprintln!("Bridge task failed: {}", e);
    ///     }
    /// });
    /// ```
    #[cfg(feature = "async")]
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let sync_rx = self
            .sync_rx
            .lock()
            .unwrap()
            .take()
            .ok_or("Sync receiver already taken")?;

        let broadcast_tx = self.broadcast_tx.clone();

        // Run the bridge in a blocking task since we're receiving from a sync channel
        tokio::task::spawn_blocking(move || {
            debug!("FFI callback bridge task started");

            while let Ok(event) = sync_rx.recv() {
                debug!("Forwarding FFI event: {:?}", event);

                // Forward to broadcast channel
                if let Err(e) = broadcast_tx.send(event) {
                    // This is not necessarily an error - it just means no one is listening
                    debug!("No subscribers for FFI event: {}", e);
                }
            }

            debug!("FFI callback bridge task stopped");
        });

        Ok(())
    }
}

impl Default for CallbackHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Context passed to C callbacks containing the sync sender.
///
/// This struct is passed as a void pointer to C callbacks and must be
/// reconstructed to access the sender.
#[repr(C)]
pub(crate) struct CallbackContext {
    sender: mpsc::Sender<FfiEvent>,
}

impl CallbackContext {
    /// Creates a new callback context.
    pub(crate) fn new(sender: mpsc::Sender<FfiEvent>) -> Self {
        Self { sender }
    }

    /// Sends an event through the callback context.
    pub(crate) fn send(&self, event: FfiEvent) {
        if let Err(e) = self.sender.send(event) {
            error!("Failed to send FFI event: {}", e);
        }
    }
}

// ============================================================================
// C Callback Functions
// ============================================================================
//
// These functions are called by the C INDIGO library. They must be marked
// as `extern "C"` and must not panic. All errors should be logged and
// handled gracefully.

/// C callback for property definition.
///
/// # Safety
///
/// This function is called by C code and must handle all errors gracefully.
/// The `client` and `property` pointers must be valid for the duration of the call.
#[cfg(feature = "sys-available")]
#[no_mangle]
pub unsafe extern "C" fn indigo_define_property_callback(
    _client: *mut libindigo_sys::indigo_client,
    _device: *mut libindigo_sys::indigo_device,
    property: *mut libindigo_sys::indigo_property,
    _message: *const std::os::raw::c_char,
) {
    // Extract context from client user data
    // For now, we'll need to store the context globally or in the client structure
    // This is a simplified version - full implementation would need proper context management

    if property.is_null() {
        warn!("Null property pointer in define callback");
        return;
    }

    match property_from_c(property) {
        Ok(prop) => {
            debug!("Property defined: {}.{}", prop.device, prop.name);
            // TODO: Send to callback handler
            // This requires storing the callback context in a way accessible from C
        }
        Err(e) => {
            error!("Failed to convert property in define callback: {}", e);
        }
    }
}

/// C callback for property update.
///
/// # Safety
///
/// This function is called by C code and must handle all errors gracefully.
/// The `client` and `property` pointers must be valid for the duration of the call.
#[cfg(feature = "sys-available")]
#[no_mangle]
pub unsafe extern "C" fn indigo_update_property_callback(
    _client: *mut libindigo_sys::indigo_client,
    _device: *mut libindigo_sys::indigo_device,
    property: *mut libindigo_sys::indigo_property,
    _message: *const std::os::raw::c_char,
) {
    if property.is_null() {
        warn!("Null property pointer in update callback");
        return;
    }

    match property_from_c(property) {
        Ok(prop) => {
            debug!("Property updated: {}.{}", prop.device, prop.name);
            // TODO: Send to callback handler
        }
        Err(e) => {
            error!("Failed to convert property in update callback: {}", e);
        }
    }
}

/// C callback for property deletion.
///
/// # Safety
///
/// This function is called by C code and must handle all errors gracefully.
/// The `client` and `property` pointers must be valid for the duration of the call.
#[cfg(feature = "sys-available")]
#[no_mangle]
pub unsafe extern "C" fn indigo_delete_property_callback(
    _client: *mut libindigo_sys::indigo_client,
    _device: *mut libindigo_sys::indigo_device,
    property: *mut libindigo_sys::indigo_property,
    _message: *const std::os::raw::c_char,
) {
    if property.is_null() {
        warn!("Null property pointer in delete callback");
        return;
    }

    match property_from_c(property) {
        Ok(prop) => {
            debug!("Property deleted: {}.{}", prop.device, prop.name);
            // TODO: Send to callback handler
        }
        Err(e) => {
            error!("Failed to convert property in delete callback: {}", e);
        }
    }
}

/// C callback for messages.
///
/// # Safety
///
/// This function is called by C code and must handle all errors gracefully.
#[cfg(feature = "sys-available")]
#[no_mangle]
pub unsafe extern "C" fn indigo_message_callback(
    _client: *mut libindigo_sys::indigo_client,
    _device: *mut libindigo_sys::indigo_device,
    _message: *const std::os::raw::c_char,
) {
    // TODO: Implement message callback
    debug!("Message callback received");
}

// Stub implementations when sys crate is not available
#[cfg(not(feature = "sys-available"))]
pub unsafe extern "C" fn indigo_define_property_callback() {
    // No-op stub
}

#[cfg(not(feature = "sys-available"))]
pub unsafe extern "C" fn indigo_update_property_callback() {
    // No-op stub
}

#[cfg(not(feature = "sys-available"))]
pub unsafe extern "C" fn indigo_delete_property_callback() {
    // No-op stub
}

#[cfg(not(feature = "sys-available"))]
pub unsafe extern "C" fn indigo_message_callback() {
    // No-op stub
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "async")]
    #[test]
    fn test_callback_handler_creation() {
        let handler = CallbackHandler::new();
        let _rx = handler.subscribe();
        // Handler should be created successfully
    }

    #[cfg(feature = "async")]
    #[test]
    fn test_multiple_subscribers() {
        let handler = CallbackHandler::new();
        let _rx1 = handler.subscribe();
        let _rx2 = handler.subscribe();
        let _rx3 = handler.subscribe();
        // Multiple subscribers should work
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_event_forwarding() {
        let handler = CallbackHandler::new();
        let mut rx = handler.subscribe();

        // Get the sync sender
        let sender = handler.sync_sender().unwrap();

        // Start the bridge task
        let bridge_handle = tokio::spawn(async move {
            handler.start().await.unwrap();
        });

        // Send a test event
        let test_event = FfiEvent::ConnectionChanged(true);
        sender.send(test_event).unwrap();

        // Receive the event
        let received = tokio::time::timeout(std::time::Duration::from_millis(100), rx.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Failed to receive event");

        match received {
            FfiEvent::ConnectionChanged(true) => (),
            _ => panic!("Unexpected event received"),
        }

        // Clean up
        drop(sender);
        let _ = tokio::time::timeout(std::time::Duration::from_millis(100), bridge_handle).await;
    }

    #[test]
    fn test_callback_context() {
        let (tx, rx) = mpsc::channel();
        let ctx = CallbackContext::new(tx);

        ctx.send(FfiEvent::ConnectionChanged(false));

        let event = rx.recv().unwrap();
        match event {
            FfiEvent::ConnectionChanged(false) => (),
            _ => panic!("Unexpected event"),
        }
    }
}
