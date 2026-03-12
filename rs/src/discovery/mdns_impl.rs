//! mDNS implementation for server discovery using mdns-sd crate.
//!
//! This module provides a pure Rust implementation using the mdns-sd crate.

use super::{DiscoveredServer, DiscoveryConfig, DiscoveryError, DiscoveryEvent, ServerDiscovery};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{mpsc, Mutex};
use tokio::time::timeout;

/// Discovers INDIGO servers using mDNS (one-shot).
pub(crate) async fn discover_servers(
    config: DiscoveryConfig,
) -> Result<Vec<DiscoveredServer>, Box<dyn std::error::Error + Send + Sync>> {
    // Create mDNS service daemon
    let mdns = mdns_sd::ServiceDaemon::new().map_err(|e| {
        DiscoveryError::InitializationFailed(format!("Failed to create mDNS daemon: {}", e))
    })?;

    // Browse for INDIGO services
    let service_type = config.get_service_type();
    let receiver = mdns.browse(service_type).map_err(|e| {
        DiscoveryError::InitializationFailed(format!("Failed to browse services: {}", e))
    })?;

    let mut servers = HashMap::new();
    let discovery_timeout = config.get_timeout();

    // Collect servers for the configured timeout
    let _result = timeout(discovery_timeout, async {
        while let Ok(event) = receiver.recv_async().await {
            match event {
                mdns_sd::ServiceEvent::ServiceResolved(info) => {
                    // Convert mdns-sd service info to DiscoveredServer
                    if let Some(server) = convert_service_info(info) {
                        if config.apply_filter(&server) {
                            servers.insert(server.id.clone(), server);
                        }
                    }
                }
                mdns_sd::ServiceEvent::ServiceRemoved(_, fullname) => {
                    servers.remove(&fullname);
                }
                _ => {}
            }
        }
    })
    .await;

    // Shutdown the mDNS daemon
    mdns.shutdown()
        .map_err(|e| DiscoveryError::DiscoveryFailed(format!("Failed to shutdown mDNS: {}", e)))?;

    Ok(servers.into_values().collect())
}

/// Starts continuous server discovery using mDNS.
pub(crate) async fn start_continuous_discovery(
    config: DiscoveryConfig,
) -> Result<ServerDiscovery, Box<dyn std::error::Error + Send + Sync>> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let servers = Arc::new(Mutex::new(HashMap::new()));
    let servers_clone = servers.clone();

    // Create mDNS service daemon
    let mdns = mdns_sd::ServiceDaemon::new().map_err(|e| {
        DiscoveryError::InitializationFailed(format!("Failed to create mDNS daemon: {}", e))
    })?;

    // Browse for INDIGO services
    let service_type = config.get_service_type().to_string();
    let receiver = mdns.browse(&service_type).map_err(|e| {
        DiscoveryError::InitializationFailed(format!("Failed to browse services: {}", e))
    })?;

    let initial_timeout = config.get_timeout();

    // Spawn background task to handle mDNS events
    let task = tokio::spawn(async move {
        let mut initial_discovery = true;
        let discovery_deadline = tokio::time::Instant::now() + initial_timeout;

        loop {
            // Check if initial discovery period is over
            if initial_discovery && tokio::time::Instant::now() >= discovery_deadline {
                initial_discovery = false;
                let _ = event_tx.send(DiscoveryEvent::DiscoveryComplete);
            }

            // Receive mDNS events with a timeout to check discovery deadline
            let timeout_duration = if initial_discovery {
                discovery_deadline.saturating_duration_since(tokio::time::Instant::now())
            } else {
                std::time::Duration::from_secs(1)
            };

            match tokio::time::timeout(timeout_duration, receiver.recv_async()).await {
                Ok(Ok(event)) => match event {
                    mdns_sd::ServiceEvent::ServiceResolved(info) => {
                        if let Some(server) = convert_service_info(info) {
                            if config.apply_filter(&server) {
                                let mut servers = servers_clone.lock().await;
                                let is_new = !servers.contains_key(&server.id);
                                servers.insert(server.id.clone(), server.clone());
                                drop(servers);

                                let event = if is_new {
                                    DiscoveryEvent::ServerAdded(server)
                                } else {
                                    DiscoveryEvent::ServerUpdated(server)
                                };
                                let _ = event_tx.send(event);
                            }
                        }
                    }
                    mdns_sd::ServiceEvent::ServiceRemoved(_, fullname) => {
                        let mut servers = servers_clone.lock().await;
                        if servers.remove(&fullname).is_some() {
                            drop(servers);
                            let _ = event_tx.send(DiscoveryEvent::ServerRemoved(fullname));
                        }
                    }
                    _ => {}
                },
                Ok(Err(e)) => {
                    let _ = event_tx.send(DiscoveryEvent::Error(format!("mDNS error: {}", e)));
                    break;
                }
                Err(_) => {
                    // Timeout - continue loop
                }
            }
        }

        // Cleanup
        let _ = mdns.shutdown();
    });

    Ok(ServerDiscovery::new(event_rx, task, servers))
}

/// Converts mdns-sd ServiceInfo to DiscoveredServer.
fn convert_service_info(info: mdns_sd::ServiceInfo) -> Option<DiscoveredServer> {
    // Get the primary hostname and strip trailing dot (mDNS FQDN format)
    let host = info.get_hostname().trim_end_matches('.').to_string();

    // Get port
    let port = info.get_port();

    // Get all IP addresses
    let addresses: Vec<IpAddr> = info.get_addresses().iter().cloned().collect();

    // Get TXT records
    let txt_records: HashMap<String, String> = info
        .get_properties()
        .iter()
        .map(|prop| {
            let key = prop.key().to_string();
            let value = prop.val_str().to_string();
            (key, value)
        })
        .collect();

    // Use fullname as ID
    let id = info.get_fullname().to_string();

    // Get service name (without domain)
    let name = info
        .get_fullname()
        .split('.')
        .next()
        .unwrap_or(&id)
        .to_string();

    Some(DiscoveredServer {
        id,
        name,
        host,
        port,
        addresses,
        txt_records,
        discovered_at: SystemTime::now(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_discover_servers_compiles() {
        let config = DiscoveryConfig::new().timeout(Duration::from_millis(100));

        // This test just verifies the code compiles
        // Actual discovery requires a running INDIGO server
        let result = discover_servers(config).await;
        // Result may be Ok or Err depending on network/platform
        let _ = result;
    }

    #[tokio::test]
    async fn test_continuous_discovery_compiles() {
        let config = DiscoveryConfig::continuous().timeout(Duration::from_millis(100));

        // This test just verifies the code compiles
        let result = start_continuous_discovery(config).await;
        // Result may be Ok or Err depending on network/platform
        let _ = result;
    }
}
