//! Server discovery API for INDIGO servers using ZeroConf/Bonjour.
//!
//! This module provides automatic discovery of INDIGO servers on the local network
//! using mDNS/DNS-SD (ZeroConf/Bonjour). It supports both one-shot discovery and
//! continuous monitoring for server changes.
//!
//! # Feature Flag
//!
//! This module is only available when the `auto` feature is enabled.
//!
//! # Example: One-Shot Discovery
//!
//! ```ignore
//! use libindigo::discovery::{DiscoveryConfig, ServerDiscoveryApi};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> libindigo::error::Result<()> {
//!     let config = DiscoveryConfig::new()
//!         .timeout(Duration::from_secs(5));
//!
//!     let servers = ServerDiscoveryApi::discover(config).await?;
//!
//!     for server in servers {
//!         println!("Found: {} at {}", server.name, server.url());
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Example: Continuous Discovery
//!
//! ```ignore
//! use libindigo::discovery::{DiscoveryConfig, DiscoveryEvent, ServerDiscoveryApi};
//!
//! #[tokio::main]
//! async fn main() -> libindigo::error::Result<()> {
//!     let config = DiscoveryConfig::continuous();
//!     let mut discovery = ServerDiscoveryApi::start_continuous(config).await?;
//!
//!     while let Some(event) = discovery.next_event().await {
//!         match event {
//!             DiscoveryEvent::ServerAdded(server) => {
//!                 println!("New server: {}", server.name);
//!             }
//!             DiscoveryEvent::ServerRemoved(id) => {
//!                 println!("Server removed: {}", id);
//!             }
//!             _ => {}
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, SystemTime};

mod api;
mod error;

#[cfg(feature = "auto")]
mod zeroconf_impl;

pub use api::{ServerDiscovery, ServerDiscoveryApi};
pub use error::DiscoveryError;

/// Information about a discovered INDIGO server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredServer {
    /// Unique identifier for this server instance.
    pub id: String,

    /// Service name (e.g., "INDIGO Server @ hostname").
    pub name: String,

    /// Hostname or primary IP address.
    pub host: String,

    /// TCP port number.
    pub port: u16,

    /// All IP addresses associated with this server.
    pub addresses: Vec<IpAddr>,

    /// TXT record metadata from mDNS.
    pub txt_records: HashMap<String, String>,

    /// Timestamp when this server was discovered.
    pub discovered_at: SystemTime,
}

impl DiscoveredServer {
    /// Returns the connection URL for this server.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let url = server.url();
    /// client.connect(&url).await?;
    /// ```
    pub fn url(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Returns the full service identifier.
    pub fn service_id(&self) -> String {
        self.id.clone()
    }
}

/// Discovery mode configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryMode {
    /// One-shot discovery: collect servers for a duration, then stop.
    OneShot,

    /// Continuous discovery: keep monitoring for server changes.
    Continuous,
}

/// Configuration for server discovery.
///
/// Use the builder pattern to configure discovery behavior.
///
/// # Example
///
/// ```ignore
/// use libindigo::discovery::DiscoveryConfig;
/// use std::time::Duration;
///
/// let config = DiscoveryConfig::new()
///     .timeout(Duration::from_secs(10))
///     .filter(|server| server.name.contains("Observatory"));
/// ```
pub struct DiscoveryConfig {
    timeout: Duration,
    service_type: String,
    filter: Option<Box<dyn Fn(&DiscoveredServer) -> bool + Send + Sync>>,
    mode: DiscoveryMode,
}

impl DiscoveryConfig {
    /// Creates a new discovery configuration with default settings.
    ///
    /// Default settings:
    /// - Timeout: 5 seconds
    /// - Service type: `_indigo._tcp.local.`
    /// - Mode: OneShot
    /// - No filter
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            service_type: "_indigo._tcp.local.".to_string(),
            filter: None,
            mode: DiscoveryMode::OneShot,
        }
    }

    /// Creates a configuration for continuous discovery.
    pub fn continuous() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            service_type: "_indigo._tcp.local.".to_string(),
            filter: None,
            mode: DiscoveryMode::Continuous,
        }
    }

    /// Sets the discovery timeout.
    ///
    /// For one-shot discovery, this is how long to wait for servers.
    /// For continuous discovery, this is the initial discovery period.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the mDNS service type to discover.
    ///
    /// Default is `_indigo._tcp.local.` for INDIGO servers.
    pub fn service_type(mut self, service_type: impl Into<String>) -> Self {
        self.service_type = service_type.into();
        self
    }

    /// Sets a filter function to select specific servers.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = DiscoveryConfig::new()
    ///     .filter(|server| server.name.contains("Telescope"));
    /// ```
    pub fn filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&DiscoveredServer) -> bool + Send + Sync + 'static,
    {
        self.filter = Some(Box::new(filter));
        self
    }

    /// Sets the discovery mode.
    pub fn mode(mut self, mode: DiscoveryMode) -> Self {
        self.mode = mode;
        self
    }

    /// Returns the configured timeout.
    pub fn get_timeout(&self) -> Duration {
        self.timeout
    }

    /// Returns the configured service type.
    pub fn get_service_type(&self) -> &str {
        &self.service_type
    }

    /// Returns the configured mode.
    pub fn get_mode(&self) -> DiscoveryMode {
        self.mode
    }

    /// Applies the filter to a server, returning true if it should be included.
    pub(crate) fn apply_filter(&self, server: &DiscoveredServer) -> bool {
        match &self.filter {
            Some(f) => f(server),
            None => true,
        }
    }
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for DiscoveryConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiscoveryConfig")
            .field("timeout", &self.timeout)
            .field("service_type", &self.service_type)
            .field("filter", &self.filter.as_ref().map(|_| "Some(...)"))
            .field("mode", &self.mode)
            .finish()
    }
}

/// Events emitted during continuous discovery.
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    /// A new server was discovered.
    ServerAdded(DiscoveredServer),

    /// A server was removed (went offline).
    ServerRemoved(String),

    /// A server's information was updated.
    ServerUpdated(DiscoveredServer),

    /// Initial discovery phase completed.
    DiscoveryComplete,

    /// An error occurred during discovery.
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovered_server_url() {
        let server = DiscoveredServer {
            id: "test-id".to_string(),
            name: "Test Server".to_string(),
            host: "192.168.1.100".to_string(),
            port: 7624,
            addresses: vec![],
            txt_records: HashMap::new(),
            discovered_at: SystemTime::now(),
        };

        assert_eq!(server.url(), "192.168.1.100:7624");
    }

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::new();
        assert_eq!(config.get_timeout(), Duration::from_secs(5));
        assert_eq!(config.get_service_type(), "_indigo._tcp.local.");
        assert_eq!(config.get_mode(), DiscoveryMode::OneShot);
    }

    #[test]
    fn test_discovery_config_builder() {
        let config = DiscoveryConfig::new()
            .timeout(Duration::from_secs(10))
            .service_type("_test._tcp.local.")
            .mode(DiscoveryMode::Continuous);

        assert_eq!(config.get_timeout(), Duration::from_secs(10));
        assert_eq!(config.get_service_type(), "_test._tcp.local.");
        assert_eq!(config.get_mode(), DiscoveryMode::Continuous);
    }

    #[test]
    fn test_discovery_config_filter() {
        let config = DiscoveryConfig::new().filter(|server| server.name.contains("Test"));

        let server1 = DiscoveredServer {
            id: "1".to_string(),
            name: "Test Server".to_string(),
            host: "localhost".to_string(),
            port: 7624,
            addresses: vec![],
            txt_records: HashMap::new(),
            discovered_at: SystemTime::now(),
        };

        let server2 = DiscoveredServer {
            id: "2".to_string(),
            name: "Other Server".to_string(),
            host: "localhost".to_string(),
            port: 7625,
            addresses: vec![],
            txt_records: HashMap::new(),
            discovered_at: SystemTime::now(),
        };

        assert!(config.apply_filter(&server1));
        assert!(!config.apply_filter(&server2));
    }
}
