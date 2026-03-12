//! # libindigo - Rust API for INDIGO astronomy clients and devices
//!
//! This crate provides the core API for developing INDIGO astronomy clients and devices.
//! It defines the Service Provider Interface (SPI) through traits and shared types.
//!
//! ## Architecture
//!
//! The libindigo crate is the core of a multi-crate workspace:
//!
//! - **libindigo** (this crate) - Core API with SPI traits and shared types
//! - **libindigo-ffi** - FFI-based strategy using C INDIGO library
//! - **libindigo-rs** - Pure Rust strategy implementation
//! - **libindigo-discovery** - Server discovery via ZeroConf/mDNS
//!
//! ## Core API
//!
//! This crate provides:
//!
//! - [`ClientStrategy`](client::ClientStrategy) - SPI trait for client implementations
//! - [`Client`](client::Client) - Client builder and facade
//! - Core types: [`Property`](types::Property), [`Device`](types::Device), etc.
//! - [`IndigoError`](error::IndigoError) - Error types
//! - INDIGO constants in the [`name`] module
//!
//! ## Example
//!
//! ```ignore
//! use libindigo::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Strategy implementations are provided by separate crates
//!     // See libindigo-ffi or libindigo-rs for concrete implementations
//!     Ok(())
//! }
//! ```

// ============================================================================
// Core API Modules
// ============================================================================

/// Error types for libindigo operations.
pub mod error;

/// Core types for properties, devices, and values.
pub mod types;

/// Client API and strategy trait (SPI).
pub mod client;

/// Device Driver SPI for implementing INDIGO devices in Rust.
pub mod device;

/// Logging configuration for libindigo.
pub mod logging;

// Re-export commonly used types
pub use client::{AvailabilityStatus, ClientStrategy, MonitoringConfig, MonitoringEvent};
pub use error::{IndigoError, Result};
pub use logging::{init_logging, LogConfig, LogLevel};
pub use types::{Device, DeviceInfo};

/// Prelude module for convenient imports.
///
/// Use this module to import the most commonly used types:
///
/// ```ignore
/// use libindigo::prelude::*;
/// ```
pub mod prelude {
    pub use crate::client::{Client, ClientBuilder, ClientStrategy};
    pub use crate::device::{DeviceDriver, DeviceInterface, DriverInfo};
    pub use crate::error::{IndigoError, Result};
    pub use crate::logging::{init_logging, LogConfig, LogLevel};
    pub use crate::types::{
        Device, DeviceInfo, Property, PropertyPerm, PropertyState, PropertyType, PropertyValue,
        SwitchRule, SwitchState,
    };
}

// ============================================================================
// INDIGO Constants
// ============================================================================

/// INDIGO protocol constants and well-known property/item names.
///
/// This module contains constants generated from the INDIGO library headers,
/// providing type-safe access to standard INDIGO property and item names.
///
/// # Example
///
/// ```ignore
/// use libindigo::name;
///
/// assert_eq!(name::INFO_PROPERTY, "INFO");
/// assert_eq!(name::CONNECTION_PROPERTY, "CONNECTION");
/// ```
pub mod name {
    include!("constants.rs");

    #[cfg(test)]
    mod tests {
        use crate::name;

        #[test]
        fn names() {
            assert_eq!(name::INFO_PROPERTY, "INFO");
        }
    }
}

/// INDIGO library version information.
///
/// This module contains version constants extracted from the INDIGO source code
/// at build time. These constants reflect the version of the INDIGO library
/// that was used to generate the bindings.
///
/// # Example
///
/// ```ignore
/// use libindigo::version;
///
/// println!("INDIGO version: {}", version::INDIGO_VERSION);
/// println!("Major: {}, Minor: {}, Build: {}",
///     version::INDIGO_VERSION_MAJOR,
///     version::INDIGO_VERSION_MINOR,
///     version::INDIGO_BUILD);
/// ```
pub mod version {
    include!(concat!(env!("OUT_DIR"), "/version.rs"));
}

#[cfg(test)]
mod tests {
    // Core API tests
}
