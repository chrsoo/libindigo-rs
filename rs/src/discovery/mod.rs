//! Server discovery API for INDIGO servers using mDNS.
//!
//! This module provides automatic discovery of INDIGO servers on the local network
//! using mDNS/DNS-SD. It supports both one-shot discovery and continuous monitoring
//! for server changes.
//!
//! # Feature Flag
//!
//! This module is only available when the `discovery` feature is enabled.
//!
//! # Example: One-Shot Discovery
//!
//! ```ignore
//! use libindigo_rs::discovery::{DiscoveryConfig, ServerDiscoveryApi};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
//! use libindigo_rs::discovery::{DiscoveryConfig, DiscoveryEvent, ServerDiscoveryApi};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

// Re-export shared types from core
pub use libindigo::discovery::{
    DiscoveredServer, DiscoveryConfig, DiscoveryError, DiscoveryEvent, DiscoveryMode,
    ServiceAnnouncement,
};

// RS-specific implementation modules
mod announce;
mod api;
mod mdns_impl;

// Re-export RS-specific types and functions
pub use announce::{announce_service, AnnouncementHandle};
pub use api::{ServerDiscovery, ServerDiscoveryApi};
