//! Client API for connecting to INDIGO servers.
//!
//! This module provides the main client interface for interacting with INDIGO
//! servers. The client uses a strategy pattern to support both FFI-based and
//! pure Rust implementations.
//!
//! # Example
//!
//! ```ignore
//! use libindigo::client::Client;
//!
//! #[tokio::main]
//! async fn main() -> libindigo::Result<()> {
//!     // Client creation will be implemented in Phase 2
//!     Ok(())
//! }
//! ```

pub mod strategy;

pub use strategy::ClientStrategy;

// TODO: Phase 2 - Implement Client struct and builder
// TODO: Phase 2 - Implement property streams
// TODO: Phase 2 - Implement event handling
