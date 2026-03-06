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
        ClientBuilder { strategy: None }
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
        let strategy = self
            .strategy
            .ok_or_else(|| IndigoError::InvalidParameter("No strategy configured".to_string()))?;

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
