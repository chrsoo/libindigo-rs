//! Server discovery API implementation.

use super::{
    announce_service, AnnouncementHandle, DiscoveredServer, DiscoveryConfig, DiscoveryEvent,
    ServiceAnnouncement,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// Main server discovery API.
///
/// This is the primary interface for discovering INDIGO servers on the local network.
///
/// # Example: One-Shot Discovery
///
/// ```ignore
/// use libindigo_rs::discovery::{DiscoveryConfig, ServerDiscoveryApi};
///
/// let servers = ServerDiscoveryApi::discover(DiscoveryConfig::new()).await?;
/// for server in servers {
///     println!("Found: {}", server.name);
/// }
/// ```
pub struct ServerDiscoveryApi;

impl ServerDiscoveryApi {
    /// Discovers INDIGO servers on the local network (one-shot).
    ///
    /// This method performs a one-shot discovery, collecting servers for the
    /// configured timeout duration and then returning the results.
    ///
    /// # Arguments
    ///
    /// * `config` - Discovery configuration
    ///
    /// # Returns
    ///
    /// A list of discovered servers, or an error if discovery fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use libindigo_rs::discovery::{DiscoveryConfig, ServerDiscoveryApi};
    /// use std::time::Duration;
    ///
    /// let config = DiscoveryConfig::new()
    ///     .timeout(Duration::from_secs(5));
    ///
    /// let servers = ServerDiscoveryApi::discover(config).await?;
    /// println!("Found {} servers", servers.len());
    /// ```
    pub async fn discover(
        config: DiscoveryConfig,
    ) -> Result<Vec<DiscoveredServer>, Box<dyn std::error::Error + Send + Sync>> {
        super::mdns_impl::discover_servers(config).await
    }

    /// Starts continuous server discovery.
    ///
    /// This method starts a background task that continuously monitors for
    /// server changes, emitting events when servers are added, removed, or updated.
    ///
    /// # Arguments
    ///
    /// * `config` - Discovery configuration (should use `DiscoveryMode::Continuous`)
    ///
    /// # Returns
    ///
    /// A [`ServerDiscovery`] handle for receiving events and controlling the discovery.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use libindigo_rs::discovery::{DiscoveryConfig, DiscoveryEvent, ServerDiscoveryApi};
    ///
    /// let mut discovery = ServerDiscoveryApi::start_continuous(
    ///     DiscoveryConfig::continuous()
    /// ).await?;
    ///
    /// while let Some(event) = discovery.next_event().await {
    ///     match event {
    ///         DiscoveryEvent::ServerAdded(server) => {
    ///             println!("New server: {}", server.name);
    ///         }
    ///         _ => {}
    ///     }
    /// }
    /// ```
    pub async fn start_continuous(
        config: DiscoveryConfig,
    ) -> Result<ServerDiscovery, Box<dyn std::error::Error + Send + Sync>> {
        super::mdns_impl::start_continuous_discovery(config).await
    }

    /// Announces an INDIGO service on the local network.
    ///
    /// This method registers the service with mDNS, making it discoverable by clients
    /// on the local network. The service will remain announced as long as the returned
    /// handle exists.
    ///
    /// # Arguments
    ///
    /// * `announcement` - Service configuration including name, port, and properties
    ///
    /// # Returns
    ///
    /// An [`AnnouncementHandle`] that keeps the service announced. When dropped, the
    /// announcement is automatically removed.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use libindigo_rs::discovery::{ServiceAnnouncement, ServerDiscoveryApi};
    ///
    /// let announcement = ServiceAnnouncement::new("My INDIGO Server", 7624)
    ///     .with_property("version", "2.0");
    ///
    /// let handle = ServerDiscoveryApi::announce(announcement).await?;
    ///
    /// // Service is now discoverable
    /// // Keep handle alive as long as you want the service announced
    ///
    /// // Stop announcing
    /// handle.stop().await?;
    /// ```
    pub async fn announce(
        announcement: ServiceAnnouncement,
    ) -> Result<AnnouncementHandle, Box<dyn std::error::Error + Send + Sync>> {
        announce_service(announcement).await
    }
}

/// Handle for continuous server discovery.
///
/// This handle allows you to receive discovery events and control the
/// continuous discovery process.
///
/// # Example
///
/// ```ignore
/// let mut discovery = ServerDiscoveryApi::start_continuous(config).await?;
///
/// // Receive events
/// while let Some(event) = discovery.next_event().await {
///     // Handle event
/// }
///
/// // Get current list of servers
/// let servers = discovery.servers();
///
/// // Stop discovery
/// discovery.stop().await?;
/// ```
pub struct ServerDiscovery {
    rx: mpsc::UnboundedReceiver<DiscoveryEvent>,
    task: JoinHandle<()>,
    servers: Arc<Mutex<HashMap<String, DiscoveredServer>>>,
}

impl ServerDiscovery {
    /// Receives the next discovery event.
    ///
    /// Returns `None` when the discovery has been stopped or an error occurred.
    ///
    /// # Example
    ///
    /// ```ignore
    /// while let Some(event) = discovery.next_event().await {
    ///     match event {
    ///         DiscoveryEvent::ServerAdded(server) => {
    ///             println!("New server: {}", server.name);
    ///         }
    ///         DiscoveryEvent::ServerRemoved(id) => {
    ///             println!("Server removed: {}", id);
    ///         }
    ///         _ => {}
    ///     }
    /// }
    /// ```
    pub async fn next_event(&mut self) -> Option<DiscoveryEvent> {
        self.rx.recv().await
    }

    /// Returns the current list of discovered servers.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let servers = discovery.servers();
    /// println!("Currently {} servers online", servers.len());
    /// ```
    pub fn servers(&self) -> Vec<DiscoveredServer> {
        let servers = self.servers.blocking_lock();
        servers.values().cloned().collect()
    }

    /// Stops the continuous discovery.
    ///
    /// This will abort the background discovery task and close the event channel.
    ///
    /// # Example
    ///
    /// ```ignore
    /// discovery.stop().await?;
    /// ```
    pub async fn stop(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.task.abort();
        let _ = self.task.await;
        Ok(())
    }
}

impl ServerDiscovery {
    pub(crate) fn new(
        rx: mpsc::UnboundedReceiver<DiscoveryEvent>,
        task: JoinHandle<()>,
        servers: Arc<Mutex<HashMap<String, DiscoveredServer>>>,
    ) -> Self {
        Self { rx, task, servers }
    }
}
