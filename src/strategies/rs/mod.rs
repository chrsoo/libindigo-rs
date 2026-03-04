//! Rust INDIGO Client Strategy
//!
//! This module provides a complete implementation of the INDIGO protocol in Rust,
//! without relying on C FFI bindings to the INDIGO library.
//!
//! # Overview
//!
//! The Rust strategy consists of five main components:
//!
//! - **Protocol** (`protocol` module): XML parsing and serialization of INDIGO messages
//! - **Protocol JSON** (`protocol_json` module): JSON parsing and serialization of INDIGO messages
//! - **Protocol Negotiation** (`protocol_negotiation` module): Protocol negotiation with JSON-first strategy
//! - **Transport** (`transport` module): TCP connection management and data transmission
//! - **Client** (`client` module): ClientStrategy implementation that ties everything together
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │     RsClientStrategy (client)       │
//! │  - Implements ClientStrategy trait  │
//! │  - Manages connection lifecycle     │
//! │  - Handles property cache           │
//! └──────────────┬──────────────────────┘
//!                │
//!       ┌────────┴────────┐
//!       │                 │
//!       ▼                 ▼
//! ┌───────────┐    ┌──────────────┐
//! │ Protocol  │    │  Transport   │
//! │ (protocol)│    │ (transport)  │
//! │ - Parser  │    │ - TcpStream  │
//! │ - Serializer   │ - Buffering  │
//! └───────────┘    └──────────────┘
//! ```
//!
//! # Features
//!
//! - **Zero FFI**: No C dependencies, Rust implementation
//! - **Async**: Built on tokio for efficient async I/O
//! - **Type Safe**: Leverages Rust's type system for protocol correctness
//! - **Efficient**: Minimal allocations, zero-copy parsing where possible
//!
//! # Usage
//!
//! This strategy is available when the `rs-strategy` feature is enabled:
//!
//! ```toml
//! [dependencies]
//! libindigo = { version = "0.1", features = ["rs-strategy"] }
//! ```
//!
//! Example usage:
//!
//! ```rust,ignore
//! use libindigo::client::ClientBuilder;
//! use libindigo::strategies::rs::RsClientStrategy;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let strategy = RsClientStrategy::new();
//!     let mut client = ClientBuilder::new()
//!         .with_strategy(strategy)
//!         .build();
//!
//!     client.connect("localhost:7624").await?;
//!     client.enumerate_properties(None).await?;
//!
//!     // ... work with properties ...
//!
//!     client.disconnect().await?;
//!     Ok(())
//! }
//! ```
//!
//! # Phase 3 Implementation Plan
//!
//! 1. **Protocol Layer** (protocol.rs)
//!    - Define protocol message types
//!    - Implement XML parser using quick-xml
//!    - Implement XML serializer
//!    - Add protocol validation
//!
//! 2. **Transport Layer** (transport.rs)
//!    - Implement TCP connection management
//!    - Add buffered I/O
//!    - Handle connection errors and timeouts
//!    - Implement graceful shutdown
//!
//! 3. **Client Layer** (client.rs)
//!    - Integrate protocol and transport
//!    - Implement ClientStrategy trait
//!    - Add property caching
//!    - Implement event streaming
//!    - Handle protocol state machine

pub mod client;
pub mod protocol;
pub mod protocol_json;
pub mod protocol_negotiation;
pub mod transport;

// Re-export the main client strategy and protocol types
pub use client::RsClientStrategy;
pub use protocol_negotiation::{ProtocolNegotiator, ProtocolType};
