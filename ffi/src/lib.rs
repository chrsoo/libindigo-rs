//! FFI-based implementation of the INDIGO protocol using C library
//!
//! This crate provides an implementation of the INDIGO astronomy protocol
//! using FFI bindings to the official C INDIGO library. It offers maximum
//! compatibility with the reference implementation while providing a safe
//! Rust API.
//!
//! # Features
//!
//! - **C Library Integration**: Uses the official INDIGO C library via FFI
//! - **Maximum Compatibility**: Guaranteed compatibility with INDIGO servers
//! - **Safe API**: Wraps unsafe FFI calls in a safe Rust interface
//! - **Optional Async**: Async wrapper available with the `async` feature
//!
//! # Architecture
//!
//! The crate is organized into several layers:
//!
//! - **Core API** (re-exported from `libindigo`): Types, traits, and error handling
//! - **FFI Layer**: Safe wrappers around C INDIGO library calls
//! - **Client Layer**: Implementation of `ClientStrategy` trait
//! - **Async Layer** (optional): Async wrapper for non-blocking operations
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use libindigo_ffi::{Client, ClientBuilder, FfiClientStrategy};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a client with the FFI strategy
//!     let strategy = FfiClientStrategy::new()?;
//!     let mut client = ClientBuilder::new()
//!         .with_strategy(strategy)
//!         .build();
//!
//!     // Connect to an INDIGO server
//!     client.connect("localhost:7624").await?;
//!
//!     // Enumerate all properties
//!     client.enumerate_properties(None).await?;
//!
//!     // Work with properties...
//!
//!     // Disconnect
//!     client.disconnect().await?;
//!     Ok(())
//! }
//! ```
//!
//! # Async Support
//!
//! Enable the `async` feature for async wrapper support:
//!
//! ```toml
//! [dependencies]
//! libindigo-ffi = { version = "0.3", features = ["async"] }
//! ```
//!
//! ```rust,ignore
//! use libindigo_ffi::{AsyncFfiStrategy, PropertyStream};
//!
//! let strategy = AsyncFfiStrategy::new()?;
//! // Use with Client as normal
//! ```
//!
//! # Feature Flags
//!
//! - `client` (default): Enable client functionality
//! - `device`: Enable device driver support (future)
//! - `async`: Enable async wrapper for non-blocking operations

// Re-export core API from libindigo
pub use libindigo::{
    // Client types
    client::{Client, ClientBuilder},
    // Error handling
    error::{IndigoError, Result},
    // Core types
    types::{
        Device, DeviceInfo, Property, PropertyPerm, PropertyState, PropertyType, PropertyValue,
        SwitchRule, SwitchState,
    },
};

// Re-export the name module (INDIGO constants)
pub use libindigo::name;

// Internal modules
mod ffi;

// Export the FFI strategy implementation
pub use ffi::FfiClientStrategy;

// Async FFI wrapper (optional)
#[cfg(feature = "async")]
mod async_ffi;
#[cfg(feature = "async")]
pub use async_ffi::{AsyncFfiStrategy, PropertyStream};

// Note: The FFI module is kept mostly internal as it contains unsafe code.
// Users should interact through FfiClientStrategy which provides a safe API.
