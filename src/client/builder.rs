//! Client builder for ergonomic client construction.
//!
//! This module provides a fluent builder API for constructing INDIGO clients
//! with different strategies and configurations.
//!
//! # Example
//!
//! ```ignore
//! use libindigo::client::{ClientBuilder, ClientStrategy};
//!
//! // Strategy implementations are provided by libindigo-rs or libindigo-ffi
//! let strategy: Box<dyn ClientStrategy> = // ... get from implementation crate
//! let client = ClientBuilder::new()
//!     .with_strategy(strategy)
//!     .build()?;
//! ```

use crate::client::ClientStrategy;
use crate::error::{IndigoError, Result};
use crate::logging::LogConfig;

#[cfg(feature = "monitoring")]
use crate::client::monitoring::{ClientEvent, MonitoringConfig};
#[cfg(feature = "monitoring")]
use tokio::sync::mpsc;

/// Builder for constructing INDIGO clients.
///
/// The builder provides a fluent API for configuring and creating clients
/// with different strategies. Strategy implementations are provided by
/// separate crates (`libindigo-rs` for pure Rust, `libindigo-ffi` for FFI-based).
///
/// # Example
///
/// ```ignore
/// use libindigo::client::ClientBuilder;
///
/// let client = ClientBuilder::new()
///     .with_strategy(my_strategy)
///     .build()?;
/// ```
pub struct ClientBuilder {
    strategy: Option<Box<dyn ClientStrategy>>,
    log_config: Option<LogConfig>,
    #[cfg(feature = "monitoring")]
    monitoring_config: Option<MonitoringConfig>,
}

impl ClientBuilder {
    /// Creates a new client builder.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use libindigo::client::ClientBuilder;
    ///
    /// let builder = ClientBuilder::new();
    /// ```
    pub fn new() -> Self {
        ClientBuilder {
            strategy: None,
            log_config: None,
            #[cfg(feature = "monitoring")]
            monitoring_config: None,
        }
    }

    /// Sets the strategy implementation for the client.
    ///
    /// The strategy determines how the client communicates with INDIGO servers.
    /// Strategy implementations are provided by:
    /// - `libindigo-rs` - Pure Rust implementation
    /// - `libindigo-ffi` - FFI-based implementation using C INDIGO library
    ///
    /// # Arguments
    ///
    /// * `strategy` - A boxed strategy implementation
    ///
    /// # Example
    ///
    /// ```ignore
    /// use libindigo::client::ClientBuilder;
    ///
    /// let strategy = // ... get from implementation crate
    /// let client = ClientBuilder::new()
    ///     .with_strategy(strategy)
    ///     .build()?;
    /// ```
    pub fn with_strategy(mut self, strategy: Box<dyn ClientStrategy>) -> Self {
        self.strategy = Some(strategy);
        self
    }

    /// Configures logging for the client.
    ///
    /// When set, logging will be initialized when the client is built.
    /// If logging initialization fails, the build will return an error.
    ///
    /// # Arguments
    ///
    /// * `config` - The logging configuration to use
    ///
    /// # Example
    ///
    /// ```ignore
    /// use libindigo::client::ClientBuilder;
    /// use libindigo::logging::{LogConfig, LogLevel};
    ///
    /// let client = ClientBuilder::new()
    ///     .with_strategy(strategy)
    ///     .with_logging(LogConfig::default().with_level(LogLevel::Debug))
    ///     .build()?;
    /// ```
    pub fn with_logging(mut self, config: LogConfig) -> Self {
        self.log_config = Some(config);
        self
    }

    /// Enables server monitoring with the given configuration.
    ///
    /// When monitoring is enabled, the client will track server availability
    /// and emit status change events that can be subscribed to via
    /// [`Client::subscribe_status()`].
    ///
    /// # Arguments
    ///
    /// * `config` - The monitoring configuration to use
    ///
    /// # Example
    ///
    /// ```ignore
    /// use libindigo::client::{ClientBuilder, MonitoringConfig};
    /// use std::time::Duration;
    ///
    /// let client = ClientBuilder::new()
    ///     .with_strategy(strategy)
    ///     .with_monitoring(
    ///         MonitoringConfig::new("192.168.1.50:7624".parse().unwrap())
    ///             .with_ping_interval(Duration::from_secs(3))
    ///             .with_window_size(5)
    ///     )
    ///     .build()?;
    /// ```
    #[cfg(feature = "monitoring")]
    pub fn with_monitoring(mut self, config: MonitoringConfig) -> Self {
        self.monitoring_config = Some(config);
        self
    }

    /// Builds the client with the configured strategy.
    ///
    /// # Errors
    ///
    /// Returns an error if no strategy has been configured.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use libindigo::client::ClientBuilder;
    ///
    /// let client = ClientBuilder::new()
    ///     .with_strategy(my_strategy)
    ///     .build()?;
    /// ```
    pub fn build(self) -> Result<Client> {
        #[cfg(feature = "monitoring")]
        let mut strategy = self
            .strategy
            .ok_or_else(|| IndigoError::InvalidParameter("No strategy configured".to_string()))?;

        #[cfg(not(feature = "monitoring"))]
        let strategy = self
            .strategy
            .ok_or_else(|| IndigoError::InvalidParameter("No strategy configured".to_string()))?;

        // Initialize logging if configured
        if let Some(config) = &self.log_config {
            crate::logging::init_logging(config).map_err(|e| {
                IndigoError::InvalidParameter(format!("Failed to initialize logging: {}", e))
            })?;
        }

        // Pass monitoring config to strategy if provided
        #[cfg(feature = "monitoring")]
        if let Some(config) = self.monitoring_config {
            strategy.set_monitoring_config(config);
        }

        Ok(Client { strategy })
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// An INDIGO client that communicates with INDIGO servers.
///
/// The client is constructed using [`ClientBuilder`] and uses a strategy
/// implementation to handle the actual communication.
///
/// # Example
///
/// ```ignore
/// use libindigo::client::{Client, ClientBuilder};
///
/// let client = ClientBuilder::new()
///     .with_strategy(my_strategy)
///     .build()?;
/// ```
pub struct Client {
    strategy: Box<dyn ClientStrategy>,
}

impl Client {
    /// Returns a reference to the client's strategy.
    ///
    /// This allows direct access to strategy-specific functionality.
    pub fn strategy(&self) -> &dyn ClientStrategy {
        &*self.strategy
    }

    /// Returns a mutable reference to the client's strategy.
    ///
    /// This allows direct access to strategy-specific functionality.
    pub fn strategy_mut(&mut self) -> &mut dyn ClientStrategy {
        &mut *self.strategy
    }

    /// Subscribes to server status events.
    ///
    /// Returns a receiver for monitoring status change events. Each event indicates
    /// a change in server availability (Available, Degraded, or Unavailable).
    ///
    /// # Returns
    ///
    /// An `UnboundedReceiver` that will receive `ClientEvent` notifications when
    /// the server status changes. Returns `None` if monitoring is not enabled or
    /// not supported by the strategy.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use libindigo::client::{ClientBuilder, ClientEvent};
    ///
    /// let mut client = ClientBuilder::new()
    ///     .with_strategy(strategy)
    ///     .with_monitoring(MonitoringConfig::new(server_addr))
    ///     .build()?;
    ///
    /// if let Some(mut rx) = client.subscribe_status() {
    ///     while let Some(event) = rx.recv().await {
    ///         match event {
    ///             ClientEvent::ServerAvailable => println!("Server is available"),
    ///             ClientEvent::ServerDegraded => println!("Server is degraded"),
    ///             ClientEvent::ServerUnavailable => println!("Server is unavailable"),
    ///         }
    ///     }
    /// }
    /// ```
    #[cfg(feature = "monitoring")]
    pub fn subscribe_status(&self) -> Option<mpsc::UnboundedReceiver<ClientEvent>> {
        self.strategy.subscribe_status()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_requires_strategy() {
        let result = ClientBuilder::new().build();
        assert!(result.is_err());
        assert!(matches!(result, Err(IndigoError::InvalidParameter(_))));
    }
}
