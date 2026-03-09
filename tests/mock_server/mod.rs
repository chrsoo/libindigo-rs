//! Pure Rust mock INDIGO server for testing.
//!
//! This module provides a mock INDIGO server implementation that supports
//! the JSON protocol (version 512) without any FFI dependencies.
//!
//! # Example
//!
//! ```ignore
//! use tests::mock_server::MockServerBuilder;
//!
//! let server = MockServerBuilder::new()
//!     .with_ccd_simulator()
//!     .build()
//!     .await?;
//!
//! let addr = server.addr();
//! // Use addr for testing...
//!
//! server.shutdown().await?;
//! ```

mod builder;
mod connection;
mod device;
mod handler;
pub mod presets;
mod property;
mod server;
mod subscription;

// Public API exports
pub use builder::MockServerBuilder;
pub use device::{DeviceMetadata, DeviceRegistry, MockDevice};
pub use property::{
    BlobValue, MockProperty, NumberValue, PropertyItem, PropertyType, PropertyTypeMetadata,
    PropertyUpdate, PropertyValue,
};
pub use server::{MockIndigoServer, ServerConfig, ServerState, ServerStats};
pub use subscription::{ClientSubscription, SubscriptionManager};
