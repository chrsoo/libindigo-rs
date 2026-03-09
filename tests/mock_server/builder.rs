//! Fluent builder API for the mock INDIGO server.

use super::device::MockDevice;
use super::server::{MockIndigoServer, ServerConfig};
use libindigo::error::Result;
use std::time::Duration;

/// Fluent builder for MockIndigoServer
pub struct MockServerBuilder {
    config: ServerConfig,
    devices: Vec<MockDevice>,
}

impl MockServerBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: ServerConfig::default(),
            devices: Vec::new(),
        }
    }

    /// Set bind address
    pub fn bind(mut self, addr: impl Into<String>) -> Self {
        self.config.bind_addr = addr.into();
        self
    }

    /// Set maximum connections
    pub fn max_connections(mut self, max: usize) -> Self {
        self.config.max_connections = max;
        self
    }

    /// Enable property streaming with interval
    pub fn with_streaming(mut self, interval: Duration) -> Self {
        self.config.update_interval = Some(interval);
        self
    }

    /// Add a mock device
    pub fn with_device(mut self, device: MockDevice) -> Self {
        self.devices.push(device);
        self
    }

    /// Add a preset device (CCD simulator)
    pub fn with_ccd_simulator(self) -> Self {
        self.with_device(super::presets::ccd_simulator())
    }

    /// Add a preset device (Mount simulator)
    pub fn with_mount_simulator(self) -> Self {
        self.with_device(super::presets::mount_simulator())
    }

    /// Enable verbose logging
    pub fn verbose(mut self) -> Self {
        self.config.verbose = true;
        self
    }

    /// Build and start the server
    pub async fn build(self) -> Result<MockIndigoServer> {
        MockIndigoServer::new(self.config, self.devices).await
    }
}

impl Default for MockServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_builder_basic() {
        let server = MockServerBuilder::new()
            .bind("127.0.0.1:0")
            .max_connections(5)
            .build()
            .await
            .unwrap();

        assert!(server.addr().port() > 0);
        server.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_builder_with_devices() {
        let server = MockServerBuilder::new()
            .with_ccd_simulator()
            .with_mount_simulator()
            .build()
            .await
            .unwrap();

        let devices = server.list_devices().await;
        assert_eq!(devices.len(), 2);
        assert!(devices.contains(&"CCD Simulator".to_string()));
        assert!(devices.contains(&"Mount Simulator".to_string()));

        server.shutdown().await.unwrap();
    }
}
