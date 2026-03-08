//! # libindigo-rs
//!
//! **⚠️ Important**: This crate is named `libindigo-rs` in Cargo.toml, but you must
//! import it as `libindigo_rs` (with underscore) in your code:
//!
//! ```rust
//! use libindigo_rs::{Client, ClientBuilder, RsClientStrategy};
//! ```
//!
//! Rust automatically converts hyphens to underscores in crate names. Using `libindigo::`
//! instead of `libindigo_rs::` will cause "unresolved module" errors.
//!
//! ---
//!
//! Pure Rust implementation of the INDIGO protocol
//!
//! This crate provides a complete implementation of the INDIGO astronomy protocol
//! in pure Rust, with ZERO C dependencies. It's designed to be a drop-in replacement
//! for FFI-based implementations while offering better safety, portability, and
//! integration with Rust's async ecosystem.
//!
//! # Features
//!
//! - **Zero FFI**: No C dependencies, pure Rust implementation
//! - **Async-first**: Built on tokio for efficient async I/O
//! - **Type-safe**: Leverages Rust's type system for protocol correctness
//! - **Protocol Support**: Both XML and JSON protocols with auto-negotiation
//! - **Cross-platform**: Works on any platform supported by Rust and tokio
//!
//! # Architecture
//!
//! The crate is organized into several layers:
//!
//! - **Core API** (re-exported from `libindigo`): Types, traits, and error handling
//! - **Protocol Layer**: XML and JSON parsing/serialization
//! - **Transport Layer**: TCP connection management
//! - **Client Layer**: High-level client implementation
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use libindigo_rs::{Client, ClientBuilder, RsClientStrategy};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a client with the Rust strategy
//!     let strategy = RsClientStrategy::new();
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
//! # Protocol Negotiation
//!
//! The client automatically negotiates the best protocol with the server:
//!
//! ```rust,ignore
//! use libindigo_rs::{RsClientStrategy, ProtocolType, ProtocolNegotiator};
//!
//! // JSON-first with XML fallback (default)
//! let strategy = RsClientStrategy::new();
//!
//! // JSON-only (no fallback)
//! let negotiator = ProtocolNegotiator::json_only();
//! let strategy = RsClientStrategy::with_protocol_negotiator(negotiator);
//!
//! // XML-only
//! let negotiator = ProtocolNegotiator::xml_only();
//! let strategy = RsClientStrategy::with_protocol_negotiator(negotiator);
//! ```
//!
//! # Feature Flags
//!
//! - `client` (default): Enable client functionality
//! - `device`: Enable device driver support (future)
//! - `discovery`: Enable mDNS server discovery (pure Rust, no FFI)

// Re-export core API from libindigo
pub use libindigo::{
    // Client types
    client::{Client, ClientBuilder},
    // Error handling
    error::{IndigoError, Result},
    // Core types
    types::{
        Device, DeviceInfo, LightState, Property, PropertyItem, PropertyPerm, PropertyState,
        PropertyType, PropertyValue, SwitchState,
    },
};

// Re-export the name module (INDIGO constants)
pub use libindigo::name;

// Internal modules
mod client;
mod protocol;
mod protocol_json;
mod protocol_negotiation;
mod transport;

// Optional discovery module (pure Rust mDNS)
#[cfg(feature = "discovery")]
pub mod discovery;

// Export the RS strategy implementation
pub use client::RsClientStrategy;

// Export protocol negotiation types for advanced users
pub use protocol_negotiation::{ProtocolNegotiator, ProtocolType};

// Note: Protocol and transport modules are kept internal as they are
// implementation details. Users should interact through RsClientStrategy.
