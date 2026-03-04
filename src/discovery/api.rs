//! Server discovery API implementation.

use super::{DiscoveredServer, DiscoveryConfig, DiscoveryEvent, DiscoveryMode};
use crate::error::Result;
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(feature = "auto")]
use tokio::sync::Mutex;
#[cfg(feature = "auto")]
use tokio::sync::mpsc;
#[cfg(feature = "auto")]
use tokio::task::JoinHandle;

/// Main server discovery API.
///
/// This is the primary interface for discovering INDIGO servers on the local network.
///
/// # Example: One-Shot Discovery
///
/// ```ignore
/// use libindigo::discovery::{DiscoveryConfig, ServerDiscoveryApi};
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
    /// use libindigo::discovery::{DiscoveryConfig, ServerDiscoveryApi};
    /// use std::time::Duration;
    ///
    /// let config = DiscoveryConfig::new()
    ///     .timeout(Duration::from_secs(5));
    ///
    /// let servers = ServerDiscoveryApi::discover(config).await?;
    /// println!("Found {} servers", servers.len());
    /// ```
    pub async fn discover(config: DiscoveryConfig) -> Result<Vec<DiscoveredServer>> {
        Self::discover_impl(config).await
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
    /// use libindigo::discovery::{DiscoveryConfig, DiscoveryEvent, ServerDiscoveryApi};
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
    pub async fn start_continuous(config: DiscoveryConfig) -> Result<ServerDiscovery> {
        Self::start_continuous_impl(config).await
    }

    #[cfg(feature = "auto")]
    async fn discover_impl(config: DiscoveryConfig) -> Result<Vec<DiscoveredServer>> {
        super::zeroconf_impl::discover_servers(config).await
    }

    #[cfg(not(feature = "auto"))]
    async fn discover_impl(_config: DiscoveryConfig) -> Result<Vec<DiscoveredServer>> {
        Err(crate::error::IndigoError::NotSupported(
            "Server discovery requires the 'auto' feature flag. \
             Enable it in Cargo.toml: features = [\"auto\"]".to_string()
        ))
    }

    #[cfg(feature = "auto")]
    async fn start_continuous_impl(config: DiscoveryConfig) -> Result<ServerDiscovery> {
        super::zeroconf_impl::start_continuous_discovery(config).await
    }

    #[cfg(not(feature = "auto"))]
    async fn start_continuous_impl(_config: DiscoveryConfig) -> Result<ServerDiscovery> {
        Err(crate::error::IndigoError::NotSupported(
            "Server discovery requires the 'auto' feature flag. \
             Enable it in Cargo.toml: features = [\"auto\"]".to_string()
        ))
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
    #[cfg(feature = "auto")]
    rx: mpsc::UnboundedReceiver<DiscoveryEvent>,
    #[cfg(feature = "auto")]
    task: JoinHandle<()>,
    #[cfg(feature = "auto")]
    servers: Arc<Mutex<HashMap<String, DiscoveredServer>>>,

    #[cfg(not(feature = "auto"))]
    _phantom: std::marker::PhantomData<()>,
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
    #[cfg(feature = "auto")]
    pub async fn next_event(&mut self) -> Option<DiscoveryEvent> {
        self.rx.recv().await
    }

    #[cfg(not(feature = "auto"))]
    pub async fn next_event(&mut self) -> Option<DiscoveryEvent> {
        None
    }

    /// Returns the current list of discovered servers.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let servers = discovery.servers();
    /// println!("Currently {} servers online", servers.len());
    /// ```
    #[cfg(feature = "auto")]
    pub fn servers(&self) -> Vec<DiscoveredServer> {
        let servers = self.servers.blocking_lock();
        servers.values().cloned().collect()
    }

    #[cfg(not(feature = "auto"))]
    pub fn servers(&self) -> Vec<DiscoveredServer> {
        Vec::new()
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
    #[cfg(feature = "auto")]
    pub async fn stop(self) -> Result<()> {
        self.task.abort();
        let _ = self.task.await;
        Ok(())
    }

    #[cfg(not(feature = "auto"))]
    pub async fn stop(self) -> Result<()> {
        Ok(())
    }
}

#[cfg(feature = "auto")]
impl ServerDiscovery {
    pub(crate) fn new(
        rx: mpsc::UnboundedReceiver<DiscoveryEvent>,
        task: JoinHandle<()>,
        servers: Arc<Mutex<HashMap<String, DiscoveredServer>>>,
    ) -> Self {
        Self { rx, task, servers }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(not(feature = "auto"))]
    async fn test_discovery_without_feature() {
        let result = ServerDiscoveryApi::discover(DiscoveryConfig::new()).await;
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(matches!(e, crate::error::IndigoError::NotSupported(_)));
        }
    }

    #[tokio::test]
    #[cfg(not(feature = "auto"))]
    async fn test_continuous_discovery_without_feature() {
        let result = ServerDiscoveryApi::start_continuous(
            DiscoveryConfig::continuous()
        ).await;
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(matches!(e, crate::error::IndigoError::NotSupported(_)));
        }
    }
}
