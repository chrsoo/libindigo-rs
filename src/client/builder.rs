//! Client builder for ergonomic client construction.
//!
//! This module provides a fluent builder API for constructing INDIGO clients
//! with different strategies and configurations.
//!
//! # Example
//!
//! ```ignore
//! use libindigo::client::ClientBuilder;
//!
//! #[tokio::main]
//! async fn main() -> libindigo::Result<()> {
//!     let client = ClientBuilder::new()
//!         .with_async_ffi_strategy()
//!         .build()?;
//!
//!     // Use the client...
//!     Ok(())
//! }
//! ```

use crate::client::ClientStrategy;
use crate::error::{IndigoError, Result};

#[cfg(all(feature = "ffi-strategy", feature = "async"))]
use crate::strategies::AsyncFfiStrategy;

#[cfg(feature = "ffi-strategy")]
use crate::strategies::FfiClientStrategy;

#[cfg(feature = "rs-strategy")]
use crate::strategies::RsClientStrategy;

/// Builder for constructing INDIGO clients.
///
/// The builder provides a fluent API for configuring and creating clients
/// with different strategies and options.
///
/// # Example
///
/// ```ignore
/// use libindigo::client::ClientBuilder;
///
/// # #[tokio::main]
/// # async fn main() -> libindigo::Result<()> {
/// // Create a client with async FFI strategy
/// let client = ClientBuilder::new()
///     .with_async_ffi_strategy()
///     .build()?;
///
/// // Create a client with synchronous FFI strategy
/// let client = ClientBuilder::new()
///     .with_ffi_strategy()
///     .build()?;
/// # Ok(())
/// # }
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

    /// Configures the client to use the async FFI strategy.
    ///
    /// This strategy wraps synchronous FFI calls in `tokio::task::spawn_blocking`
    /// for non-blocking operation.
    ///
    /// # Availability
    ///
    /// This method is only available when both the `ffi-strategy` and `async`
    /// features are enabled.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use libindigo::client::ClientBuilder;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> libindigo::Result<()> {
    /// let client = ClientBuilder::new()
    ///     .with_async_ffi_strategy()
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(all(feature = "ffi-strategy", feature = "async"))]
    pub fn with_async_ffi_strategy(mut self) -> Self {
        self.strategy = Some(Box::new(AsyncFfiStrategy::new()));
        self
    }

    /// Configures the client to use the synchronous FFI strategy.
    ///
    /// This strategy directly calls the C INDIGO library via FFI.
    ///
    /// # Availability
    ///
    /// This method is only available when the `ffi-strategy` feature is enabled.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use libindigo::client::ClientBuilder;
    ///
    /// let client = ClientBuilder::new()
    ///     .with_ffi_strategy()
    ///     .build()?;
    /// ```
    #[cfg(feature = "ffi-strategy")]
    pub fn with_ffi_strategy(mut self) -> Self {
        self.strategy = Some(Box::new(FfiClientStrategy::new()));
        self
    }

    /// Configures the client to use the pure Rust strategy.
    ///
    /// This strategy implements the INDIGO protocol entirely in Rust without FFI.
    ///
    /// # Availability
    ///
    /// This method is only available when the `rs-strategy` feature is enabled.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use libindigo::client::ClientBuilder;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> libindigo::Result<()> {
    /// let client = ClientBuilder::new()
    ///     .with_rs_strategy()
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "rs-strategy")]
    pub fn with_rs_strategy(mut self) -> Self {
        self.strategy = Some(Box::new(RsClientStrategy::new()));
        self
    }

    /// Configures the client with a custom strategy.
    ///
    /// This allows using a custom implementation of the [`ClientStrategy`] trait.
    ///
    /// # Arguments
    ///
    /// * `strategy` - A boxed strategy implementation
    ///
    /// # Example
    ///
    /// ```ignore
    /// use libindigo::client::{ClientBuilder, ClientStrategy};
    ///
    /// struct MyStrategy;
    /// // ... implement ClientStrategy for MyStrategy ...
    ///
    /// let client = ClientBuilder::new()
    ///     .with_strategy(Box::new(MyStrategy))
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
    /// # #[tokio::main]
    /// # async fn main() -> libindigo::Result<()> {
    /// let client = ClientBuilder::new()
    ///     .with_async_ffi_strategy()
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self) -> Result<Client> {
        let strategy = self.strategy.ok_or_else(|| {
            IndigoError::InvalidState(
                "No strategy configured. Call one of the with_*_strategy() methods.".to_string(),
            )
        })?;

        Ok(Client::new(strategy))
    }
}

impl Default for ClientBuilder {
    /// Creates a new client builder with default settings.
    ///
    /// This is equivalent to calling [`ClientBuilder::new()`].
    fn default() -> Self {
        Self::new()
    }
}

/// INDIGO client for interacting with INDIGO servers.
///
/// The client uses a strategy pattern to support different implementations
/// (FFI-based, pure Rust, etc.). Use [`ClientBuilder`] to construct a client.
///
/// # Example
///
/// ```ignore
/// use libindigo::client::ClientBuilder;
/// use futures::StreamExt;
///
/// #[tokio::main]
/// async fn main() -> libindigo::Result<()> {
///     let mut client = ClientBuilder::new()
///         .with_async_ffi_strategy()
///         .build()?;
///
///     // Connect to server
///     client.connect("localhost:7624").await?;
///
///     // Enumerate properties
///     client.enumerate_properties(None).await?;
///
///     // Disconnect
///     client.disconnect().await?;
///
///     Ok(())
/// }
/// ```
pub struct Client {
    strategy: Box<dyn ClientStrategy>,
}

impl Client {
    /// Creates a new client with the given strategy.
    ///
    /// Most users should use [`ClientBuilder`] instead of calling this directly.
    ///
    /// # Arguments
    ///
    /// * `strategy` - The strategy implementation to use
    ///
    /// # Example
    ///
    /// ```ignore
    /// use libindigo::client::Client;
    /// use libindigo::strategies::AsyncFfiStrategy;
    ///
    /// let client = Client::new(Box::new(AsyncFfiStrategy::new()));
    /// ```
    pub fn new(strategy: Box<dyn ClientStrategy>) -> Self {
        Client { strategy }
    }

    /// Connects to an INDIGO server at the specified URL.
    ///
    /// # Arguments
    ///
    /// * `url` - Server URL in the format "host:port" (e.g., "localhost:7624")
    ///
    /// # Errors
    ///
    /// Returns an error if the connection fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use libindigo::client::ClientBuilder;
    /// # #[tokio::main]
    /// # async fn main() -> libindigo::Result<()> {
    /// let mut client = ClientBuilder::new()
    ///     .with_async_ffi_strategy()
    ///     .build()?;
    ///
    /// client.connect("localhost:7624").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(&mut self, url: &str) -> Result<()> {
        self.strategy.connect(url).await
    }

    /// Disconnects from the INDIGO server.
    ///
    /// # Errors
    ///
    /// Returns an error if disconnection fails or if not currently connected.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use libindigo::client::ClientBuilder;
    /// # #[tokio::main]
    /// # async fn main() -> libindigo::Result<()> {
    /// let mut client = ClientBuilder::new()
    ///     .with_async_ffi_strategy()
    ///     .build()?;
    ///
    /// client.connect("localhost:7624").await?;
    /// client.disconnect().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn disconnect(&mut self) -> Result<()> {
        self.strategy.disconnect().await
    }

    /// Requests enumeration of properties from the server.
    ///
    /// # Arguments
    ///
    /// * `device` - Optional device name to enumerate properties for.
    ///              If `None`, enumerates properties for all devices.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use libindigo::client::ClientBuilder;
    /// # #[tokio::main]
    /// # async fn main() -> libindigo::Result<()> {
    /// let mut client = ClientBuilder::new()
    ///     .with_async_ffi_strategy()
    ///     .build()?;
    ///
    /// client.connect("localhost:7624").await?;
    ///
    /// // Enumerate all properties
    /// client.enumerate_properties(None).await?;
    ///
    /// // Enumerate properties for a specific device
    /// client.enumerate_properties(Some("CCD Simulator")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn enumerate_properties(&mut self, device: Option<&str>) -> Result<()> {
        self.strategy.enumerate_properties(device).await
    }

    /// Sends a property update to the server.
    ///
    /// # Arguments
    ///
    /// * `property` - The property to send
    ///
    /// # Errors
    ///
    /// Returns an error if sending fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use libindigo::client::ClientBuilder;
    /// # use libindigo::types::{Property, PropertyType};
    /// # #[tokio::main]
    /// # async fn main() -> libindigo::Result<()> {
    /// let mut client = ClientBuilder::new()
    ///     .with_async_ffi_strategy()
    ///     .build()?;
    ///
    /// client.connect("localhost:7624").await?;
    ///
    /// let property = Property::builder()
    ///     .device("CCD Simulator")
    ///     .name("CONNECTION")
    ///     .property_type(PropertyType::Switch)
    ///     .build();
    ///
    /// client.send_property(property).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_property(&mut self, property: crate::types::Property) -> Result<()> {
        self.strategy.send_property(property).await
    }

    /// Returns a reference to the underlying strategy.
    ///
    /// This can be used to access strategy-specific functionality.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use libindigo::client::ClientBuilder;
    /// # #[tokio::main]
    /// # async fn main() -> libindigo::Result<()> {
    /// let client = ClientBuilder::new()
    ///     .with_async_ffi_strategy()
    ///     .build()?;
    ///
    /// let strategy = client.strategy();
    /// # Ok(())
    /// # }
    /// ```
    pub fn strategy(&self) -> &dyn ClientStrategy {
        self.strategy.as_ref()
    }

    /// Returns a mutable reference to the underlying strategy.
    ///
    /// This can be used to access strategy-specific functionality.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use libindigo::client::ClientBuilder;
    /// # #[tokio::main]
    /// # async fn main() -> libindigo::Result<()> {
    /// let mut client = ClientBuilder::new()
    ///     .with_async_ffi_strategy()
    ///     .build()?;
    ///
    /// let strategy = client.strategy_mut();
    /// # Ok(())
    /// # }
    /// ```
    pub fn strategy_mut(&mut self) -> &mut dyn ClientStrategy {
        self.strategy.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_without_strategy_fails() {
        let result = ClientBuilder::new().build();
        assert!(result.is_err());
    }

    #[cfg(all(feature = "ffi-strategy", feature = "async"))]
    #[test]
    fn test_builder_with_async_ffi_strategy() {
        let result = ClientBuilder::new().with_async_ffi_strategy().build();
        assert!(result.is_ok());
    }

    #[cfg(feature = "ffi-strategy")]
    #[test]
    fn test_builder_with_ffi_strategy() {
        let result = ClientBuilder::new().with_ffi_strategy().build();
        assert!(result.is_ok());
    }

    #[cfg(all(feature = "ffi-strategy", feature = "async"))]
    #[tokio::test]
    async fn test_client_connect_invalid_url() {
        let mut client = ClientBuilder::new()
            .with_async_ffi_strategy()
            .build()
            .unwrap();

        let result = client.connect("invalid").await;
        assert!(result.is_err());
    }
}
