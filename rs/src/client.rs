//! Rust INDIGO Client Implementation
//!
//! This module implements the `ClientStrategy` trait using Rust,
//! without relying on C FFI bindings to the INDIGO library.
//!
//! # Overview
//!
//! The Rust client provides:
//!
//! - **Protocol Implementation**: Full INDIGO XML protocol in Rust
//! - **TCP Communication**: Direct TCP connection to INDIGO servers
//! - **Async Operations**: Fully async using tokio
//! - **Type Safety**: Leverages Rust's type system for protocol correctness
//!
//! # Architecture
//!
//! The client is composed of three main components:
//!
//! 1. **Transport Layer** (`transport` module): Handles TCP connections
//! 2. **Protocol Layer** (`protocol` module): Parses and serializes XML messages
//! 3. **Client Logic** (this module): Implements the ClientStrategy trait
//!
//! # Usage
//!
//! ```rust,ignore
//! use libindigo::client::ClientBuilder;
//! use libindigo::strategies::rs::RsClientStrategy;
//!
//! let strategy = RsClientStrategy::new();
//! let client = ClientBuilder::new()
//!     .with_strategy(strategy)
//!     .build();
//!
//! client.connect("localhost:7624").await?;
//! ```

use async_trait::async_trait;
use libindigo::client::strategy::ClientStrategy;
use libindigo::error::{IndigoError, Result};
use libindigo::types::property::PropertyItem;
use libindigo::types::value::{LightState, PropertyValue, SwitchState as DomainSwitchState};
use libindigo::types::{BlobTransferMode, Property, PropertyPerm, PropertyState, PropertyType};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;

#[cfg(feature = "monitoring")]
use crate::monitoring::ServerMonitor;
#[cfg(feature = "monitoring")]
use libindigo::client::monitoring::{ClientEvent, MonitoringConfig, MonitoringEvent};

use crate::protocol::{
    decode_blob, encode_blob, BLOBEnable, EnableBLOB, GetProperties, NewBLOBVector,
    NewNumberVector, NewSwitchVector, NewTextVector, NewVectorAttributes, OneBLOB, OneNumber,
    OneSwitch, OneText, ProtocolMessage, SwitchState as ProtocolSwitchState,
};
use crate::protocol_negotiation::{ProtocolNegotiator, ProtocolType};
use crate::transport::{Transport, WriteTransport};

/// Rust client strategy implementation.
///
/// This strategy implements the INDIGO protocol entirely in Rust,
/// communicating directly with INDIGO servers via TCP without
/// using C FFI bindings.
///
/// # Thread Safety
///
/// The client uses `Arc<Mutex<>>` for internal state to allow safe
/// concurrent access from multiple async tasks.
pub struct RsClientStrategy {
    /// Shared internal state.
    state: Arc<Mutex<ClientState>>,
}

/// Internal client state.
struct ClientState {
    /// Write half of TCP transport (for sending messages).
    write_transport: Option<WriteTransport>,
    /// Channel sender for property updates.
    property_tx: Option<mpsc::UnboundedSender<Property>>,
    /// Channel receiver for property updates (moved to stream).
    /// DEPRECATED: Use property_subscribers instead.
    property_rx: Option<mpsc::UnboundedReceiver<Property>>,
    /// List of property subscribers for multi-subscriber pattern.
    property_subscribers: Vec<mpsc::UnboundedSender<Property>>,
    /// Background task handle for receiving messages.
    background_task: Option<JoinHandle<()>>,
    /// Connection state flag.
    connected: bool,
    /// Negotiated protocol type.
    protocol: ProtocolType,
    /// Protocol negotiator for establishing protocol.
    negotiator: ProtocolNegotiator,
    /// Monitoring configuration (when monitoring feature is enabled).
    #[cfg(feature = "monitoring")]
    monitoring_config: Option<MonitoringConfig>,
    /// Server monitor instance (when monitoring is active).
    #[cfg(feature = "monitoring")]
    monitoring_handle: Option<ServerMonitor>,
    /// List of monitoring event subscribers.
    #[cfg(feature = "monitoring")]
    monitoring_subscribers: Vec<mpsc::UnboundedSender<ClientEvent>>,
}

impl RsClientStrategy {
    /// Creates a new Rust client strategy.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let strategy = RsClientStrategy::new();
    /// ```
    pub fn new() -> Self {
        Self::with_protocol_negotiator(ProtocolNegotiator::default())
    }

    /// Creates a new Rust client strategy with a custom protocol negotiator.
    ///
    /// # Arguments
    ///
    /// * `negotiator` - The protocol negotiator to use
    ///
    /// # Example
    ///
    /// ```ignore
    /// use libindigo::strategies::rs::protocol_negotiation::ProtocolNegotiator;
    ///
    /// let negotiator = ProtocolNegotiator::json_only();
    /// let strategy = RsClientStrategy::with_protocol_negotiator(negotiator);
    /// ```
    pub fn with_protocol_negotiator(negotiator: ProtocolNegotiator) -> Self {
        RsClientStrategy {
            state: Arc::new(Mutex::new(ClientState {
                write_transport: None,
                property_tx: None,
                property_rx: None,
                property_subscribers: Vec::new(),
                background_task: None,
                connected: false,
                protocol: ProtocolType::default(),
                negotiator,
                #[cfg(feature = "monitoring")]
                monitoring_config: None,
                #[cfg(feature = "monitoring")]
                monitoring_handle: None,
                #[cfg(feature = "monitoring")]
                monitoring_subscribers: Vec::new(),
            })),
        }
    }

    /// Sets the protocol preference for this client.
    ///
    /// This should be called before connecting to the server.
    ///
    /// # Arguments
    ///
    /// * `protocol` - The preferred protocol type
    /// * `fallback_enabled` - Whether to enable fallback to alternate protocol
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut strategy = RsClientStrategy::new();
    /// strategy.set_protocol_preference(ProtocolType::Json, true).await;
    /// ```
    pub async fn set_protocol_preference(
        &mut self,
        protocol: ProtocolType,
        fallback_enabled: bool,
    ) {
        let mut state = self.state.lock().await;
        state.negotiator = ProtocolNegotiator::new(protocol, fallback_enabled);
    }

    /// Gets the currently negotiated protocol type.
    ///
    /// Returns the protocol that was successfully negotiated with the server.
    /// Before connection, this returns the default protocol (JSON).
    pub async fn protocol(&self) -> ProtocolType {
        let state = self.state.lock().await;
        state.protocol
    }

    /// Subscribes to property updates from the server.
    ///
    /// This method creates a new receiver channel for property updates, allowing
    /// multiple subscribers to receive property updates simultaneously. Each call
    /// to this method returns a new independent receiver.
    ///
    /// # Returns
    ///
    /// A new `UnboundedReceiver<Property>` that will receive all property updates
    /// from the server.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use libindigo::client::ClientBuilder;
    /// use libindigo::strategies::rs::RsClientStrategy;
    ///
    /// let mut strategy = RsClientStrategy::new();
    /// let client = ClientBuilder::new()
    ///     .with_strategy(strategy)
    ///     .build();
    ///
    /// client.connect("localhost:7624").await?;
    ///
    /// // Create multiple subscribers
    /// let mut subscriber1 = strategy.subscribe_properties().await;
    /// let mut subscriber2 = strategy.subscribe_properties().await;
    ///
    /// // Both subscribers will receive all property updates
    /// tokio::spawn(async move {
    ///     while let Some(property) = subscriber1.recv().await {
    ///         println!("Subscriber 1: {:?}", property);
    ///     }
    /// });
    ///
    /// tokio::spawn(async move {
    ///     while let Some(property) = subscriber2.recv().await {
    ///         println!("Subscriber 2: {:?}", property);
    ///     }
    /// });
    /// ```
    ///
    /// # Notes
    ///
    /// - Each subscriber receives a clone of every property update
    /// - Subscribers are automatically removed when their receiver is dropped
    /// - This method can be called multiple times to create multiple subscribers
    /// - Property updates are broadcast to all active subscribers
    pub async fn subscribe_properties(&self) -> mpsc::UnboundedReceiver<Property> {
        let mut state = self.state.lock().await;
        let (tx, rx) = mpsc::unbounded_channel();
        state.property_subscribers.push(tx);
        rx
    }

    /// Starts the background task for receiving messages from the server.
    ///
    /// This task continuously reads messages from the transport, converts them
    /// to domain `Property` types, and broadcasts them to all subscribers.
    ///
    /// # Arguments
    ///
    /// * `read_transport` - The read half of the transport (owned by this task)
    /// * `state` - Shared client state for broadcasting properties
    async fn start_receiver_task(
        mut read_transport: crate::transport::ReadTransport,
        state: Arc<Mutex<ClientState>>,
    ) -> Result<()> {
        // Create channel for property updates (for backward compatibility)
        let (tx, rx) = mpsc::unbounded_channel();

        // Clone the state Arc for the background task
        let task_state = Arc::clone(&state);

        // Spawn background task
        let handle = tokio::spawn(async move {
            tracing::info!("Background receiver task started");
            let mut message_count = 0;

            loop {
                // Receive message from transport (no need to take/put back!)
                match read_transport.receive_message().await {
                    Ok(msg) => {
                        message_count += 1;
                        tracing::debug!("Received message #{}: {:?}", message_count, msg);

                        // Convert protocol message to property
                        match Self::convert_to_property(msg) {
                            Some(property) => {
                                tracing::debug!(
                                    "Converted to property: device={}, name={}, items={}",
                                    property.device,
                                    property.name,
                                    property.items.len()
                                );

                                // Broadcast property update to all subscribers
                                let mut state = task_state.lock().await;

                                // Send to legacy channel (backward compatibility)
                                if let Some(ref tx) = state.property_tx {
                                    let _ = tx.send(property.clone());
                                }

                                // Broadcast to all subscribers
                                // Remove disconnected subscribers while iterating
                                let subscriber_count = state.property_subscribers.len();
                                state
                                    .property_subscribers
                                    .retain(|subscriber| subscriber.send(property.clone()).is_ok());

                                let retained_count = state.property_subscribers.len();
                                if retained_count < subscriber_count {
                                    tracing::debug!(
                                        "Removed {} disconnected subscribers",
                                        subscriber_count - retained_count
                                    );
                                }
                            }
                            None => {
                                tracing::trace!(
                                    "Message did not convert to property (control message)"
                                );
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Error receiving message: {}", e);

                        // Check if this is a connection error
                        if matches!(e, IndigoError::ConnectionError(_)) {
                            // Connection closed, exit task
                            tracing::info!("Connection closed, exiting receiver task");
                            let mut state = task_state.lock().await;
                            state.connected = false;
                            break;
                        }
                        // For other errors, continue trying
                        tracing::debug!("Non-fatal error, continuing to receive");
                    }
                }
            }

            tracing::info!(
                "Background receiver task exited after {} messages",
                message_count
            );
        });

        // Store channel and task handle in state
        let mut state = state.lock().await;
        state.property_tx = Some(tx);
        state.property_rx = Some(rx);
        state.background_task = Some(handle);

        Ok(())
    }

    /// Converts a protocol message to a domain `Property` type.
    ///
    /// Returns `None` for messages that don't represent properties
    /// (e.g., control messages, status messages).
    fn convert_to_property(msg: ProtocolMessage) -> Option<Property> {
        match msg {
            // Definition messages
            ProtocolMessage::DefTextVector(v) => {
                let mut items = HashMap::new();
                for elem in v.elements {
                    items.insert(
                        elem.name.clone(),
                        PropertyItem {
                            name: elem.name,
                            label: elem.label,
                            value: PropertyValue::Text(elem.value),
                        },
                    );
                }
                Some(Property {
                    device: v.attrs.device,
                    name: v.attrs.name,
                    group: v.attrs.group,
                    label: v.attrs.label,
                    state: v.attrs.state,
                    perm: v.perm,
                    property_type: PropertyType::Text,
                    items,
                    timeout: v.attrs.timeout,
                    timestamp: v.attrs.timestamp,
                    message: v.attrs.message,
                })
            }
            ProtocolMessage::DefNumberVector(v) => {
                let mut items = HashMap::new();
                for elem in v.elements {
                    items.insert(
                        elem.name.clone(),
                        PropertyItem {
                            name: elem.name,
                            label: elem.label,
                            value: PropertyValue::Number {
                                value: elem.value,
                                min: elem.min,
                                max: elem.max,
                                step: elem.step,
                                format: elem.format,
                            },
                        },
                    );
                }
                Some(Property {
                    device: v.attrs.device,
                    name: v.attrs.name,
                    group: v.attrs.group,
                    label: v.attrs.label,
                    state: v.attrs.state,
                    perm: v.perm,
                    property_type: PropertyType::Number,
                    items,
                    timeout: v.attrs.timeout,
                    timestamp: v.attrs.timestamp,
                    message: v.attrs.message,
                })
            }
            ProtocolMessage::DefSwitchVector(v) => {
                let mut items = HashMap::new();
                for elem in v.elements {
                    let state = match elem.value {
                        ProtocolSwitchState::Off => DomainSwitchState::Off,
                        ProtocolSwitchState::On => DomainSwitchState::On,
                    };
                    items.insert(
                        elem.name.clone(),
                        PropertyItem {
                            name: elem.name,
                            label: elem.label,
                            value: PropertyValue::Switch { state },
                        },
                    );
                }
                Some(Property {
                    device: v.attrs.device,
                    name: v.attrs.name,
                    group: v.attrs.group,
                    label: v.attrs.label,
                    state: v.attrs.state,
                    perm: v.perm,
                    property_type: PropertyType::Switch,
                    items,
                    timeout: v.attrs.timeout,
                    timestamp: v.attrs.timestamp,
                    message: v.attrs.message,
                })
            }
            ProtocolMessage::DefLightVector(v) => {
                let mut items = HashMap::new();
                for elem in v.elements {
                    let state = match elem.value {
                        PropertyState::Idle => LightState::Idle,
                        PropertyState::Ok => LightState::Ok,
                        PropertyState::Busy => LightState::Busy,
                        PropertyState::Alert => LightState::Alert,
                    };
                    items.insert(
                        elem.name.clone(),
                        PropertyItem {
                            name: elem.name,
                            label: elem.label,
                            value: PropertyValue::Light { state },
                        },
                    );
                }
                Some(Property {
                    device: v.attrs.device,
                    name: v.attrs.name,
                    group: v.attrs.group,
                    label: v.attrs.label,
                    state: v.attrs.state,
                    perm: PropertyPerm::ReadOnly, // Lights are always read-only
                    property_type: PropertyType::Light,
                    items,
                    timeout: v.attrs.timeout,
                    timestamp: v.attrs.timestamp,
                    message: v.attrs.message,
                })
            }
            ProtocolMessage::DefBLOBVector(v) => {
                let mut items = HashMap::new();
                for elem in v.elements {
                    items.insert(
                        elem.name.clone(),
                        PropertyItem {
                            name: elem.name,
                            label: elem.label,
                            value: PropertyValue::Blob {
                                data: Vec::new(),
                                format: String::new(),
                                size: 0,
                            },
                        },
                    );
                }
                Some(Property {
                    device: v.attrs.device,
                    name: v.attrs.name,
                    group: v.attrs.group,
                    label: v.attrs.label,
                    state: v.attrs.state,
                    perm: v.perm,
                    property_type: PropertyType::Blob,
                    items,
                    timeout: v.attrs.timeout,
                    timestamp: v.attrs.timestamp,
                    message: v.attrs.message,
                })
            }

            // Set messages (property updates)
            ProtocolMessage::SetTextVector(v) => {
                let mut items = HashMap::new();
                for elem in v.elements {
                    items.insert(
                        elem.name.clone(),
                        PropertyItem {
                            name: elem.name,
                            label: String::new(),
                            value: PropertyValue::Text(elem.value),
                        },
                    );
                }
                Some(Property {
                    device: v.attrs.device,
                    name: v.attrs.name,
                    group: String::new(),
                    label: String::new(),
                    state: v.attrs.state.unwrap_or(PropertyState::Idle),
                    perm: PropertyPerm::ReadWrite,
                    property_type: PropertyType::Text,
                    items,
                    timeout: v.attrs.timeout,
                    timestamp: v.attrs.timestamp,
                    message: v.attrs.message,
                })
            }
            ProtocolMessage::SetNumberVector(v) => {
                let mut items = HashMap::new();
                for elem in v.elements {
                    items.insert(
                        elem.name.clone(),
                        PropertyItem {
                            name: elem.name,
                            label: String::new(),
                            value: PropertyValue::Number {
                                value: elem.value,
                                min: f64::MIN,
                                max: f64::MAX,
                                step: 0.0,
                                format: "%.2f".to_string(),
                            },
                        },
                    );
                }
                Some(Property {
                    device: v.attrs.device,
                    name: v.attrs.name,
                    group: String::new(),
                    label: String::new(),
                    state: v.attrs.state.unwrap_or(PropertyState::Idle),
                    perm: PropertyPerm::ReadWrite,
                    property_type: PropertyType::Number,
                    items,
                    timeout: v.attrs.timeout,
                    timestamp: v.attrs.timestamp,
                    message: v.attrs.message,
                })
            }
            ProtocolMessage::SetSwitchVector(v) => {
                let mut items = HashMap::new();
                for elem in v.elements {
                    let state = match elem.value {
                        ProtocolSwitchState::Off => DomainSwitchState::Off,
                        ProtocolSwitchState::On => DomainSwitchState::On,
                    };
                    items.insert(
                        elem.name.clone(),
                        PropertyItem {
                            name: elem.name,
                            label: String::new(),
                            value: PropertyValue::Switch { state },
                        },
                    );
                }
                Some(Property {
                    device: v.attrs.device,
                    name: v.attrs.name,
                    group: String::new(),
                    label: String::new(),
                    state: v.attrs.state.unwrap_or(PropertyState::Idle),
                    perm: PropertyPerm::ReadWrite,
                    property_type: PropertyType::Switch,
                    items,
                    timeout: v.attrs.timeout,
                    timestamp: v.attrs.timestamp,
                    message: v.attrs.message,
                })
            }
            ProtocolMessage::SetLightVector(v) => {
                let mut items = HashMap::new();
                for elem in v.elements {
                    let state = match elem.value {
                        PropertyState::Idle => LightState::Idle,
                        PropertyState::Ok => LightState::Ok,
                        PropertyState::Busy => LightState::Busy,
                        PropertyState::Alert => LightState::Alert,
                    };
                    items.insert(
                        elem.name.clone(),
                        PropertyItem {
                            name: elem.name,
                            label: String::new(),
                            value: PropertyValue::Light { state },
                        },
                    );
                }
                Some(Property {
                    device: v.attrs.device,
                    name: v.attrs.name,
                    group: String::new(),
                    label: String::new(),
                    state: v.attrs.state.unwrap_or(PropertyState::Idle),
                    perm: PropertyPerm::ReadOnly,
                    property_type: PropertyType::Light,
                    items,
                    timeout: v.attrs.timeout,
                    timestamp: v.attrs.timestamp,
                    message: v.attrs.message,
                })
            }
            ProtocolMessage::SetBLOBVector(v) => {
                let mut items = HashMap::new();
                for elem in v.elements {
                    // Decode base64 BLOB data
                    let data = match decode_blob(&elem.value) {
                        Ok(decoded) => decoded,
                        Err(e) => {
                            tracing::warn!(
                                "Failed to decode BLOB {}: {}. Using empty data.",
                                elem.name,
                                e
                            );
                            Vec::new()
                        }
                    };

                    items.insert(
                        elem.name.clone(),
                        PropertyItem {
                            name: elem.name,
                            label: String::new(),
                            value: PropertyValue::Blob {
                                data,
                                format: elem.format,
                                size: elem.size,
                            },
                        },
                    );
                }
                Some(Property {
                    device: v.attrs.device,
                    name: v.attrs.name,
                    group: String::new(),
                    label: String::new(),
                    state: v.attrs.state.unwrap_or(PropertyState::Idle),
                    perm: PropertyPerm::ReadWrite,
                    property_type: PropertyType::Blob,
                    items,
                    timeout: v.attrs.timeout,
                    timestamp: v.attrs.timestamp,
                    message: v.attrs.message,
                })
            }

            // Control messages don't represent properties
            ProtocolMessage::GetProperties(_)
            | ProtocolMessage::EnableBLOB(_)
            | ProtocolMessage::Message(_)
            | ProtocolMessage::DelProperty(_)
            | ProtocolMessage::NewTextVector(_)
            | ProtocolMessage::NewNumberVector(_)
            | ProtocolMessage::NewSwitchVector(_)
            | ProtocolMessage::NewBLOBVector(_) => None,
        }
    }

    /// Converts a domain `Property` to a protocol message for sending.
    ///
    /// This creates a `newXXXVector` message appropriate for the property type.
    fn convert_from_property(prop: Property) -> Result<ProtocolMessage> {
        let attrs = NewVectorAttributes {
            device: prop.device,
            name: prop.name,
            timestamp: prop.timestamp,
        };

        match prop.property_type {
            PropertyType::Text => {
                let mut elements = Vec::new();
                for (_, item) in prop.items {
                    if let PropertyValue::Text(value) = item.value {
                        elements.push(OneText {
                            name: item.name,
                            value,
                        });
                    }
                }
                Ok(ProtocolMessage::NewTextVector(NewTextVector {
                    attrs,
                    elements,
                }))
            }
            PropertyType::Number => {
                let mut elements = Vec::new();
                for (_, item) in prop.items {
                    if let PropertyValue::Number { value, .. } = item.value {
                        elements.push(OneNumber {
                            name: item.name,
                            value,
                        });
                    }
                }
                Ok(ProtocolMessage::NewNumberVector(NewNumberVector {
                    attrs,
                    elements,
                }))
            }
            PropertyType::Switch => {
                let mut elements = Vec::new();
                for (_, item) in prop.items {
                    if let PropertyValue::Switch { state } = item.value {
                        let protocol_state = match state {
                            DomainSwitchState::Off => ProtocolSwitchState::Off,
                            DomainSwitchState::On => ProtocolSwitchState::On,
                        };
                        elements.push(OneSwitch {
                            name: item.name,
                            value: protocol_state,
                        });
                    }
                }
                Ok(ProtocolMessage::NewSwitchVector(NewSwitchVector {
                    attrs,
                    elements,
                }))
            }
            PropertyType::Light => Err(IndigoError::InvalidParameter(
                "Cannot send Light properties (read-only)".to_string(),
            )),
            PropertyType::Blob => {
                // Convert BLOB property to newBLOBVector message
                let mut elements = Vec::new();
                for (name, item) in &prop.items {
                    if let PropertyValue::Blob { data, format, size } = &item.value {
                        // Encode binary data as base64
                        let encoded = encode_blob(data);
                        elements.push(OneBLOB {
                            name: name.clone(),
                            size: *size,
                            format: format.clone(),
                            value: encoded,
                        });
                    }
                }
                Ok(ProtocolMessage::NewBLOBVector(NewBLOBVector {
                    attrs,
                    elements,
                }))
            }
        }
    }

    /// Returns a receiver for property updates.
    ///
    /// **DEPRECATED**: This method uses `take()` which makes the receiver unavailable
    /// for subsequent calls, breaking property streaming for multiple subscribers.
    /// Use [`subscribe_properties()`](Self::subscribe_properties) instead, which supports
    /// multiple concurrent subscribers.
    ///
    /// This method is kept for backward compatibility but will be removed in a future version.
    ///
    /// # Migration
    ///
    /// Replace:
    /// ```ignore
    /// let mut rx = strategy.property_receiver().await.unwrap();
    /// ```
    ///
    /// With:
    /// ```ignore
    /// let mut rx = strategy.subscribe_properties().await;
    /// ```
    #[deprecated(
        since = "0.1.0",
        note = "Use subscribe_properties() instead for multi-subscriber support"
    )]
    pub async fn property_receiver(&self) -> Option<mpsc::UnboundedReceiver<Property>> {
        let mut state = self.state.lock().await;
        state.property_rx.take()
    }

    /// Negotiates protocol with the server.
    ///
    /// This is an internal method that performs the protocol negotiation handshake.
    /// It tries the preferred protocol first, and falls back to XML if needed.
    ///
    /// # Arguments
    ///
    /// * `transport` - The transport to use for negotiation
    /// * `negotiator` - The protocol negotiator with preferences
    ///
    /// # Returns
    ///
    /// The successfully negotiated protocol type.
    async fn negotiate_protocol(
        &self,
        transport: &mut Transport,
        negotiator: &ProtocolNegotiator,
    ) -> Result<ProtocolType> {
        // Use the negotiator to establish protocol
        negotiator.negotiate(transport).await
    }

    /// Starts monitoring with the given configuration.
    ///
    /// This spawns a background task that monitors server availability and
    /// forwards status change events to all subscribers.
    #[cfg(feature = "monitoring")]
    async fn start_monitoring(&self, config: MonitoringConfig) -> Result<()> {
        let monitor = ServerMonitor::new(config.clone());
        let mut event_rx = monitor.start().await;

        // Store the monitor handle
        let mut state = self.state.lock().await;
        state.monitoring_handle = Some(monitor);
        drop(state);

        // Spawn a task to forward monitoring events to subscribers
        let state_clone = Arc::clone(&self.state);
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                // Only forward StatusChanged events, filter out low-level events
                if let MonitoringEvent::StatusChanged {
                    previous: _,
                    current,
                } = event
                {
                    let client_event = ClientEvent::from_status(current);

                    // Broadcast to all subscribers
                    let state = state_clone.lock().await;
                    let mut dead_subscribers = Vec::new();

                    for (idx, subscriber) in state.monitoring_subscribers.iter().enumerate() {
                        if subscriber.send(client_event.clone()).is_err() {
                            dead_subscribers.push(idx);
                        }
                    }

                    // Clean up dead subscribers
                    drop(state);
                    if !dead_subscribers.is_empty() {
                        let mut state = state_clone.lock().await;
                        for idx in dead_subscribers.iter().rev() {
                            state.monitoring_subscribers.remove(*idx);
                        }
                    }
                }
            }
        });

        tracing::info!("Started server monitoring for {}", config.server_addr);
        Ok(())
    }
}

impl Default for RsClientStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ClientStrategy for RsClientStrategy {
    /// Connects to an INDIGO server.
    ///
    /// # Arguments
    ///
    /// * `url` - Server URL in format "host:port" (e.g., "localhost:7624")
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Already connected
    /// - Connection fails
    /// - URL format is invalid
    ///
    /// # Example
    ///
    /// ```ignore
    /// strategy.connect("localhost:7624").await?;
    /// ```
    async fn connect(&mut self, url: &str) -> Result<()> {
        let state = self.state.lock().await;

        if state.connected {
            return Err(IndigoError::InvalidState("Already connected".to_string()));
        }

        // Create and connect transport
        let mut transport = Transport::connect(url).await?;

        // Negotiate protocol with server
        let negotiator = state.negotiator.clone();

        #[cfg(feature = "monitoring")]
        let monitoring_config = state.monitoring_config.clone();

        drop(state); // Drop lock before async negotiation

        let protocol = self.negotiate_protocol(&mut transport, &negotiator).await?;

        // Set protocol on transport
        transport.set_protocol(protocol);

        tracing::info!("Connected to {} using protocol {:?}", url, protocol);

        // Split transport into read and write halves
        let (read_transport, write_transport) = transport.split()?;

        // Store write transport and protocol in state
        let mut state = self.state.lock().await;
        state.write_transport = Some(write_transport);
        state.protocol = protocol;
        state.connected = true;

        // Drop the lock before starting the receiver task
        drop(state);

        // Start background receiver task with read transport
        Self::start_receiver_task(read_transport, Arc::clone(&self.state)).await?;

        // Send initial getProperties to enumerate all devices
        self.enumerate_properties(None).await?;

        // Start monitoring if configured
        #[cfg(feature = "monitoring")]
        if let Some(config) = monitoring_config {
            self.start_monitoring(config).await?;
        }

        Ok(())
    }

    /// Disconnects from the INDIGO server.
    ///
    /// Stops the background receiver task and closes the TCP connection.
    ///
    /// # Errors
    ///
    /// Returns an error if not currently connected.
    async fn disconnect(&mut self) -> Result<()> {
        let mut state = self.state.lock().await;

        if !state.connected {
            return Err(IndigoError::InvalidState("Not connected".to_string()));
        }

        // Stop monitoring if active
        #[cfg(feature = "monitoring")]
        if let Some(monitor) = state.monitoring_handle.take() {
            monitor.stop().await;
            state.monitoring_subscribers.clear();
            tracing::info!("Stopped server monitoring");
        }

        // Stop background task
        if let Some(handle) = state.background_task.take() {
            handle.abort();
        }

        // Drop write transport (this will close the connection)
        state.write_transport = None;

        // Clear channels
        state.property_tx = None;
        state.property_rx = None;
        state.property_subscribers.clear();
        state.connected = false;

        tracing::info!("Disconnected from server");

        Ok(())
    }

    /// Requests enumeration of properties from the server.
    ///
    /// # Arguments
    ///
    /// * `device` - Optional device name to enumerate properties for.
    ///              If `None`, enumerates properties for all devices.
    ///
    /// # Errors
    ///
    /// Returns an error if not connected or if sending fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Enumerate all properties
    /// strategy.enumerate_properties(None).await?;
    ///
    /// // Enumerate properties for specific device
    /// strategy.enumerate_properties(Some("CCD Simulator")).await?;
    /// ```
    async fn enumerate_properties(&mut self, device: Option<&str>) -> Result<()> {
        let mut state = self.state.lock().await;

        if !state.connected {
            return Err(IndigoError::InvalidState("Not connected".to_string()));
        }

        // Create getProperties message
        let msg = ProtocolMessage::GetProperties(GetProperties {
            version: Some("1.7".to_string()),
            device: device.map(|s| s.to_string()),
            name: None,
        });

        // Send via write transport
        if let Some(ref mut write_transport) = state.write_transport {
            tracing::debug!("Sending getProperties for device: {:?}", device);
            write_transport.send_message(&msg).await?;
        } else {
            return Err(IndigoError::InvalidState(
                "Transport not available".to_string(),
            ));
        }

        Ok(())
    }

    /// Sends a property update to the server.
    ///
    /// # Arguments
    ///
    /// * `property` - The property to send
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Not connected
    /// - Property type is not supported for sending (e.g., Light)
    /// - Sending fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let property = Property::builder()
    ///     .device("CCD Simulator")
    ///     .name("CONNECTION")
    ///     .property_type(PropertyType::Switch)
    ///     .item(PropertyItem::new("CONNECT", "Connect", PropertyValue::switch(SwitchState::On)))
    ///     .build()?;
    ///
    /// strategy.send_property(property).await?;
    /// ```
    async fn send_property(&mut self, property: Property) -> Result<()> {
        let mut state = self.state.lock().await;

        if !state.connected {
            return Err(IndigoError::InvalidState("Not connected".to_string()));
        }

        // Convert property to protocol message
        let msg = Self::convert_from_property(property)?;

        // Send via write transport
        if let Some(ref mut write_transport) = state.write_transport {
            write_transport.send_message(&msg).await?;
        } else {
            return Err(IndigoError::InvalidState(
                "Transport not available".to_string(),
            ));
        }

        Ok(())
    }

    /// Enables or configures BLOB transfer mode for a device.
    ///
    /// This sends an `enableBLOB` message to the server to control how BLOBs
    /// are transferred for the specified device.
    ///
    /// # Arguments
    ///
    /// * `device` - The device name to configure BLOB transfer for
    /// * `name` - Optional property name to limit BLOB configuration to a specific property
    /// * `mode` - The BLOB transfer mode to use
    ///
    /// # Errors
    ///
    /// Returns an error if not connected or if sending fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use libindigo::types::BlobTransferMode;
    ///
    /// // Enable BLOB transfer for all properties on CCD Simulator
    /// strategy.enable_blob("CCD Simulator", None, BlobTransferMode::Also).await?;
    ///
    /// // Disable BLOB transfer for a specific property
    /// strategy.enable_blob("CCD Simulator", Some("CCD_IMAGE"), BlobTransferMode::Never).await?;
    /// ```
    async fn enable_blob(
        &mut self,
        device: &str,
        name: Option<&str>,
        mode: BlobTransferMode,
    ) -> Result<()> {
        let mut state = self.state.lock().await;

        if !state.connected {
            return Err(IndigoError::InvalidState("Not connected".to_string()));
        }

        // Convert BlobTransferMode to protocol BLOBEnable
        let blob_enable = match mode {
            BlobTransferMode::Never => BLOBEnable::Never,
            BlobTransferMode::Also => BLOBEnable::Also,
            BlobTransferMode::Only => BLOBEnable::Only,
        };

        // Create enableBLOB message
        let msg = ProtocolMessage::EnableBLOB(EnableBLOB {
            device: device.to_string(),
            name: name.map(|s| s.to_string()),
            value: blob_enable,
        });

        // Send via write transport
        if let Some(ref mut write_transport) = state.write_transport {
            write_transport.send_message(&msg).await?;
        } else {
            return Err(IndigoError::InvalidState(
                "Transport not available".to_string(),
            ));
        }

        Ok(())
    }

    #[cfg(feature = "monitoring")]
    fn set_monitoring_config(&mut self, config: MonitoringConfig) {
        // Store the config in state - it will be used when connect() is called
        if let Ok(mut state) = self.state.try_lock() {
            state.monitoring_config = Some(config);
        }
    }

    #[cfg(feature = "monitoring")]
    fn subscribe_status(&self) -> Option<mpsc::UnboundedReceiver<ClientEvent>> {
        if let Ok(mut state) = self.state.try_lock() {
            let (tx, rx) = mpsc::unbounded_channel();
            state.monitoring_subscribers.push(tx);
            Some(rx)
        } else {
            None
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use libindigo::types::value::SwitchState as DomainSwitchState;

    #[test]
    fn test_convert_def_text_vector() {
        use crate::protocol::{DefText, DefTextVector, VectorAttributes};

        let protocol_msg = ProtocolMessage::DefTextVector(DefTextVector {
            attrs: VectorAttributes {
                device: "CCD Simulator".to_string(),
                name: "DRIVER_INFO".to_string(),
                label: "Driver Info".to_string(),
                group: "General".to_string(),
                state: PropertyState::Idle,
                timeout: None,
                timestamp: None,
                message: None,
            },
            perm: PropertyPerm::ReadOnly,
            elements: vec![DefText {
                name: "DRIVER_NAME".to_string(),
                label: "Name".to_string(),
                value: "CCD Simulator".to_string(),
            }],
        });

        let property = RsClientStrategy::convert_to_property(protocol_msg).unwrap();

        assert_eq!(property.device, "CCD Simulator");
        assert_eq!(property.name, "DRIVER_INFO");
        assert_eq!(property.property_type, PropertyType::Text);
        assert_eq!(property.items.len(), 1);
        assert!(property.items.contains_key("DRIVER_NAME"));
    }

    #[test]
    fn test_convert_def_number_vector() {
        use crate::protocol::{DefNumber, DefNumberVector, VectorAttributes};

        let protocol_msg = ProtocolMessage::DefNumberVector(DefNumberVector {
            attrs: VectorAttributes {
                device: "CCD Simulator".to_string(),
                name: "CCD_EXPOSURE".to_string(),
                label: "Exposure".to_string(),
                group: "Main".to_string(),
                state: PropertyState::Idle,
                timeout: Some(60.0),
                timestamp: None,
                message: None,
            },
            perm: PropertyPerm::ReadWrite,
            elements: vec![DefNumber {
                name: "EXPOSURE".to_string(),
                label: "Duration".to_string(),
                format: "%.2f".to_string(),
                min: 0.0,
                max: 3600.0,
                step: 0.01,
                value: 1.0,
            }],
        });

        let property = RsClientStrategy::convert_to_property(protocol_msg).unwrap();

        assert_eq!(property.device, "CCD Simulator");
        assert_eq!(property.name, "CCD_EXPOSURE");
        assert_eq!(property.property_type, PropertyType::Number);
        assert_eq!(property.timeout, Some(60.0));
    }

    #[test]
    fn test_convert_def_switch_vector() {
        use crate::protocol::{DefSwitch, DefSwitchVector, SwitchRule, VectorAttributes};

        let protocol_msg = ProtocolMessage::DefSwitchVector(DefSwitchVector {
            attrs: VectorAttributes {
                device: "CCD Simulator".to_string(),
                name: "CONNECTION".to_string(),
                label: "Connection".to_string(),
                group: "Main".to_string(),
                state: PropertyState::Idle,
                timeout: None,
                timestamp: None,
                message: None,
            },
            perm: PropertyPerm::ReadWrite,
            rule: SwitchRule::OneOfMany,
            elements: vec![
                DefSwitch {
                    name: "CONNECT".to_string(),
                    label: "Connect".to_string(),
                    value: ProtocolSwitchState::On,
                },
                DefSwitch {
                    name: "DISCONNECT".to_string(),
                    label: "Disconnect".to_string(),
                    value: ProtocolSwitchState::Off,
                },
            ],
        });

        let property = RsClientStrategy::convert_to_property(protocol_msg).unwrap();

        assert_eq!(property.device, "CCD Simulator");
        assert_eq!(property.name, "CONNECTION");
        assert_eq!(property.property_type, PropertyType::Switch);
        assert_eq!(property.items.len(), 2);
    }

    #[test]
    fn test_convert_from_property_text() {
        let mut items = HashMap::new();
        items.insert(
            "TEXT1".to_string(),
            PropertyItem {
                name: "TEXT1".to_string(),
                label: "Text 1".to_string(),
                value: PropertyValue::Text("Hello".to_string()),
            },
        );

        let property = Property {
            device: "Device".to_string(),
            name: "PROPERTY".to_string(),
            group: "Group".to_string(),
            label: "Label".to_string(),
            state: PropertyState::Idle,
            perm: PropertyPerm::ReadWrite,
            property_type: PropertyType::Text,
            items,
            timeout: None,
            timestamp: None,
            message: None,
        };

        let msg = RsClientStrategy::convert_from_property(property).unwrap();

        match msg {
            ProtocolMessage::NewTextVector(v) => {
                assert_eq!(v.attrs.device, "Device");
                assert_eq!(v.attrs.name, "PROPERTY");
                assert_eq!(v.elements.len(), 1);
                assert_eq!(v.elements[0].name, "TEXT1");
                assert_eq!(v.elements[0].value, "Hello");
            }
            _ => panic!("Expected NewTextVector"),
        }
    }

    #[test]
    fn test_convert_from_property_number() {
        let mut items = HashMap::new();
        items.insert(
            "NUM1".to_string(),
            PropertyItem {
                name: "NUM1".to_string(),
                label: "Number 1".to_string(),
                value: PropertyValue::Number {
                    value: 42.5,
                    min: 0.0,
                    max: 100.0,
                    step: 0.1,
                    format: "%.1f".to_string(),
                },
            },
        );

        let property = Property {
            device: "Device".to_string(),
            name: "PROPERTY".to_string(),
            group: "Group".to_string(),
            label: "Label".to_string(),
            state: PropertyState::Idle,
            perm: PropertyPerm::ReadWrite,
            property_type: PropertyType::Number,
            items,
            timeout: None,
            timestamp: None,
            message: None,
        };

        let msg = RsClientStrategy::convert_from_property(property).unwrap();

        match msg {
            ProtocolMessage::NewNumberVector(v) => {
                assert_eq!(v.attrs.device, "Device");
                assert_eq!(v.attrs.name, "PROPERTY");
                assert_eq!(v.elements.len(), 1);
                assert_eq!(v.elements[0].name, "NUM1");
                assert_eq!(v.elements[0].value, 42.5);
            }
            _ => panic!("Expected NewNumberVector"),
        }
    }

    #[test]
    fn test_convert_from_property_switch() {
        let mut items = HashMap::new();
        items.insert(
            "SWITCH1".to_string(),
            PropertyItem {
                name: "SWITCH1".to_string(),
                label: "Switch 1".to_string(),
                value: PropertyValue::Switch {
                    state: DomainSwitchState::On,
                },
            },
        );

        let property = Property {
            device: "Device".to_string(),
            name: "PROPERTY".to_string(),
            group: "Group".to_string(),
            label: "Label".to_string(),
            state: PropertyState::Idle,
            perm: PropertyPerm::ReadWrite,
            property_type: PropertyType::Switch,
            items,
            timeout: None,
            timestamp: None,
            message: None,
        };

        let msg = RsClientStrategy::convert_from_property(property).unwrap();

        match msg {
            ProtocolMessage::NewSwitchVector(v) => {
                assert_eq!(v.attrs.device, "Device");
                assert_eq!(v.attrs.name, "PROPERTY");
                assert_eq!(v.elements.len(), 1);
                assert_eq!(v.elements[0].name, "SWITCH1");
                assert_eq!(v.elements[0].value, ProtocolSwitchState::On);
            }
            _ => panic!("Expected NewSwitchVector"),
        }
    }

    #[test]
    fn test_convert_from_property_light_fails() {
        let mut items = HashMap::new();
        items.insert(
            "LIGHT1".to_string(),
            PropertyItem {
                name: "LIGHT1".to_string(),
                label: "Light 1".to_string(),
                value: PropertyValue::Light {
                    state: LightState::Ok,
                },
            },
        );

        let property = Property {
            device: "Device".to_string(),
            name: "PROPERTY".to_string(),
            group: "Group".to_string(),
            label: "Label".to_string(),
            state: PropertyState::Idle,
            perm: PropertyPerm::ReadOnly,
            property_type: PropertyType::Light,
            items,
            timeout: None,
            timestamp: None,
            message: None,
        };

        let result = RsClientStrategy::convert_from_property(property);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            IndigoError::InvalidParameter(_)
        ));
    }

    #[tokio::test]
    async fn test_new_strategy() {
        let strategy = RsClientStrategy::new();
        let state = strategy.state.lock().await;
        assert!(!state.connected);
        assert!(state.write_transport.is_none());
        assert!(state.property_tx.is_none());
    }

    #[tokio::test]
    async fn test_connect_requires_url() {
        let mut strategy = RsClientStrategy::new();
        // This will fail because there's no server running, but tests the flow
        let result = strategy.connect("invalid:99999").await;
        assert!(result.is_err());
    }
}
