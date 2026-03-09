//! Main mock INDIGO server implementation.

use super::device::DeviceRegistry;
use super::property::{MockProperty, PropertyUpdate};
use super::subscription::SubscriptionManager;
use libindigo::error::{IndigoError, Result};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;

/// Pure Rust mock INDIGO server for testing.
///
/// This server implements the INDIGO JSON protocol (version 512) without
/// any FFI dependencies. It's designed for testing the libindigo-rs client.
pub struct MockIndigoServer {
    /// Server configuration
    config: ServerConfig,

    /// TCP listener address
    addr: SocketAddr,

    /// Shared server state
    state: Arc<ServerState>,

    /// Shutdown signal sender
    shutdown_tx: broadcast::Sender<()>,

    /// Server task handle
    task_handle: Option<JoinHandle<Result<()>>>,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Bind address (default: "127.0.0.1:0" for random port)
    pub bind_addr: String,

    /// Protocol version (always 512 for JSON)
    pub protocol_version: u32,

    /// Maximum concurrent connections
    pub max_connections: usize,

    /// Property update interval for streaming (None = no auto-updates)
    pub update_interval: Option<Duration>,

    /// Enable verbose logging
    pub verbose: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:0".to_string(),
            protocol_version: 512,
            max_connections: 10,
            update_interval: None,
            verbose: false,
        }
    }
}

/// Shared server state (thread-safe)
pub struct ServerState {
    /// Device registry
    pub devices: RwLock<DeviceRegistry>,

    /// Active client subscriptions
    pub subscriptions: RwLock<SubscriptionManager>,

    /// Connection counter for debugging
    pub connection_count: AtomicUsize,

    /// Server statistics
    pub stats: RwLock<ServerStats>,
}

/// Server statistics for testing/debugging
#[derive(Debug, Default, Clone)]
pub struct ServerStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub messages_received: usize,
    pub messages_sent: usize,
    pub properties_defined: usize,
    pub properties_updated: usize,
}

impl ServerState {
    /// Create new server state
    pub fn new() -> Self {
        Self {
            devices: RwLock::new(DeviceRegistry::new()),
            subscriptions: RwLock::new(SubscriptionManager::new()),
            connection_count: AtomicUsize::new(0),
            stats: RwLock::new(ServerStats::default()),
        }
    }
}

impl MockIndigoServer {
    /// Create a new mock server with configuration and devices
    pub async fn new(
        config: ServerConfig,
        devices: Vec<super::device::MockDevice>,
    ) -> Result<Self> {
        // Bind TCP listener
        let listener = TcpListener::bind(&config.bind_addr)
            .await
            .map_err(|e| IndigoError::ConnectionError(format!("Failed to bind: {}", e)))?;
        let addr = listener
            .local_addr()
            .map_err(|e| IndigoError::ConnectionError(format!("Failed to get address: {}", e)))?;

        // Create shared state
        let state = Arc::new(ServerState::new());

        // Add devices to registry
        {
            let mut registry = state.devices.write().await;
            for device in devices {
                registry.add_device(device);
            }
        }

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = broadcast::channel(16);

        // Spawn server task
        let state_clone = state.clone();
        let config_clone = config.clone();
        let task_handle = tokio::spawn(async move {
            Self::run_server(listener, state_clone, config_clone, shutdown_rx).await
        });

        Ok(Self {
            config,
            addr,
            state,
            shutdown_tx,
            task_handle: Some(task_handle),
        })
    }

    /// Get the server's listening address
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Get server statistics
    pub async fn stats(&self) -> ServerStats {
        self.state.stats.read().await.clone()
    }

    /// Add a device at runtime
    pub async fn add_device(&self, device: super::device::MockDevice) -> Result<()> {
        let mut registry = self.state.devices.write().await;
        registry.add_device(device);
        Ok(())
    }

    /// Update a property value
    pub async fn update_property(
        &self,
        device: &str,
        property: &str,
        update: PropertyUpdate,
    ) -> Result<()> {
        let mut registry = self.state.devices.write().await;
        registry.update_property(device, property, update)?;

        // Notify subscribers
        if let Some(prop) = registry.get_property(device, property) {
            let subscriptions = self.state.subscriptions.read().await;
            let message = super::handler::property_to_set_message(prop)?;
            subscriptions.notify_property_update(device, property, message);
        }

        Ok(())
    }

    /// Get a property value (for test assertions)
    pub async fn get_property(&self, device: &str, property: &str) -> Option<MockProperty> {
        let registry = self.state.devices.read().await;
        registry.get_property(device, property).cloned()
    }

    /// List all devices
    pub async fn list_devices(&self) -> Vec<String> {
        let registry = self.state.devices.read().await;
        registry
            .list_devices()
            .iter()
            .map(|d| d.name.clone())
            .collect()
    }

    /// Shutdown the server gracefully
    pub async fn shutdown(mut self) -> Result<()> {
        // Send shutdown signal to all tasks
        let _ = self.shutdown_tx.send(());

        // Wait for server task to complete
        if let Some(handle) = self.task_handle.take() {
            handle
                .await
                .map_err(|e| IndigoError::ConnectionError(format!("Task join error: {}", e)))??;
        }

        Ok(())
    }

    /// Internal server loop
    async fn run_server(
        listener: TcpListener,
        state: Arc<ServerState>,
        config: ServerConfig,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<()> {
        // Spawn property updater if configured
        if let Some(interval) = config.update_interval {
            let state_clone = state.clone();
            let shutdown_clone = shutdown_rx.resubscribe();
            tokio::spawn(async move {
                Self::property_updater(state_clone, interval, shutdown_clone).await;
            });
        }

        // Accept connections
        let mut connection_id = 0;

        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, _addr)) => {
                            connection_id += 1;
                            state.connection_count.fetch_add(1, Ordering::SeqCst);

                            // Update stats
                            {
                                let mut stats = state.stats.write().await;
                                stats.total_connections += 1;
                                stats.active_connections += 1;
                            }

                            let conn = super::connection::Connection::new(
                                connection_id,
                                stream,
                                state.clone(),
                                shutdown_rx.resubscribe(),
                            );

                            // Spawn task for this connection
                            let state_clone = state.clone();
                            tokio::spawn(async move {
                                if let Err(e) = conn.handle().await {
                                    eprintln!("Connection {} error: {}", connection_id, e);
                                }
                                // Decrement active connections
                                let mut stats = state_clone.stats.write().await;
                                stats.active_connections = stats.active_connections.saturating_sub(1);
                            });
                        }
                        Err(e) => {
                            eprintln!("Accept error: {}", e);
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Property updater task for simulated streaming
    async fn property_updater(
        state: Arc<ServerState>,
        interval: Duration,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) {
        let mut interval_timer = tokio::time::interval(interval);

        loop {
            tokio::select! {
                _ = interval_timer.tick() => {
                    // Update simulated properties
                    if let Err(e) = Self::update_simulated_properties(&state).await {
                        eprintln!("Property update error: {}", e);
                    }
                }
                _ = shutdown_rx.recv() => {
                    break;
                }
            }
        }
    }

    /// Update simulated properties (e.g., temperature changes)
    async fn update_simulated_properties(state: &ServerState) -> Result<()> {
        use rand::Rng;

        let mut devices = state.devices.write().await;
        let subscriptions = state.subscriptions.read().await;

        // Example: Update CCD temperature
        if let Some(device) = devices.get_device_mut("CCD Simulator") {
            if let Some(property) = device.properties.get_mut("CCD_TEMPERATURE") {
                // Simulate temperature change
                for item in &mut property.items {
                    if item.name == "CCD_TEMPERATURE_VALUE" {
                        if let super::property::PropertyValue::Number(ref mut num) = item.value {
                            let mut rng = rand::thread_rng();
                            num.value += (rng.gen::<f64>() - 0.5) * 0.1;
                            num.value = num.value.clamp(num.min, num.max);
                        }
                    }
                }
                property.state = libindigo_rs::protocol::PropertyState::Ok;

                // Notify subscribers
                if let Ok(message) = super::handler::property_to_set_message(property) {
                    subscriptions.notify_property_update(&property.device, &property.name, message);
                }
            }
        }

        Ok(())
    }
}
