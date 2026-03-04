//! Client API for connecting to INDIGO servers.
//!
//! This module provides the main client interface for interacting with INDIGO
//! servers. The client uses a strategy pattern to support both FFI-based and
//! pure Rust implementations.
//!
//! # Example
//!
//! ```ignore
//! use libindigo::client::ClientBuilder;
//! use futures::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> libindigo::Result<()> {
//!     // Create a client with async FFI strategy
//!     let mut client = ClientBuilder::new()
//!         .with_async_ffi_strategy()
//!         .build()?;
//!
//!     // Connect to server
//!     client.connect("localhost:7624").await?;
//!
//!     // Enumerate properties
//!     client.enumerate_properties(None).await?;
//!
//!     // Disconnect
//!     client.disconnect().await?;
//!
//!     Ok(())
//! }
//! ```

pub mod builder;
pub mod strategy;

pub use builder::{Client, ClientBuilder};
pub use strategy::ClientStrategy;
