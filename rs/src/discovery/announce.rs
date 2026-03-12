//! Service announcement implementation for INDIGO servers.
//!
//! This module provides functionality for announcing INDIGO services on the local
//! network via mDNS, allowing clients to discover them automatically.

use super::{DiscoveryError, ServiceAnnouncement};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Handle for an active service announcement.
///
/// The service will be announced on the network as long as this handle exists.
/// When dropped, the service announcement is automatically removed.
///
/// # Example
///
/// ```ignore
/// let handle = ServerDiscoveryApi::announce(announcement).await?;
///
/// // Service is now visible on the network
/// // ...
///
/// // Stop announcing
/// handle.stop().await?;
/// ```
pub struct AnnouncementHandle {
    mdns: Arc<Mutex<Option<mdns_sd::ServiceDaemon>>>,
    fullname: String,
}

impl AnnouncementHandle {
    /// Creates a new announcement handle.
    pub(crate) fn new(mdns: mdns_sd::ServiceDaemon, fullname: String) -> Self {
        Self {
            mdns: Arc::new(Mutex::new(Some(mdns))),
            fullname,
        }
    }

    /// Returns the full service name being announced.
    pub fn fullname(&self) -> &str {
        &self.fullname
    }

    /// Stops the service announcement.
    ///
    /// After calling this method, the service will no longer be visible on the network.
    pub async fn stop(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut mdns_guard = self.mdns.lock().await;
        if let Some(mdns) = mdns_guard.take() {
            mdns.shutdown().map_err(|e| {
                DiscoveryError::RegistrationFailed(format!("Shutdown failed: {}", e))
            })?;
        }
        Ok(())
    }
}

impl Drop for AnnouncementHandle {
    fn drop(&mut self) {
        // Best-effort cleanup - we can't await in Drop
        if let Some(mdns) = self.mdns.blocking_lock().take() {
            let _ = mdns.shutdown();
        }
    }
}

/// Announces an INDIGO service on the local network.
///
/// This function registers the service with mDNS, making it discoverable by clients
/// on the local network.
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
/// use libindigo_rs::discovery::{ServiceAnnouncement, announce_service};
///
/// let announcement = ServiceAnnouncement::new("My INDIGO Server", 7624)
///     .with_property("version", "2.0");
///
/// let handle = announce_service(announcement).await?;
///
/// // Service is now discoverable
/// // Keep handle alive as long as you want the service announced
/// ```
pub async fn announce_service(
    announcement: ServiceAnnouncement,
) -> Result<AnnouncementHandle, Box<dyn std::error::Error + Send + Sync>> {
    // Create mDNS service daemon
    let mdns = mdns_sd::ServiceDaemon::new().map_err(|e| {
        DiscoveryError::InitializationFailed(format!("Failed to create mDNS daemon: {}", e))
    })?;

    // Service type for INDIGO
    let service_type = "_indigo._tcp.local.";

    // Get hostname for the service
    let hostname = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "localhost".to_string());

    // Register the service
    let service_info = mdns_sd::ServiceInfo::new(
        service_type,
        &announcement.name,
        &hostname,
        (), // Use default IP addresses
        announcement.port,
        if announcement.properties.is_empty() {
            None
        } else {
            Some(announcement.properties.clone())
        },
    )
    .map_err(|e| {
        DiscoveryError::RegistrationFailed(format!("Failed to create service info: {}", e))
    })?;

    let fullname = service_info.get_fullname().to_string();

    mdns.register(service_info).map_err(|e| {
        DiscoveryError::RegistrationFailed(format!("Failed to register service: {}", e))
    })?;

    Ok(AnnouncementHandle::new(mdns, fullname))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_announce_service_compiles() {
        let announcement = ServiceAnnouncement::new("Test", 7624);

        // This test just verifies the code compiles
        // Actual announcement requires network access
        let result = announce_service(announcement).await;
        let _ = result;
    }
}
