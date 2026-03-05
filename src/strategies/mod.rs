//! Strategy implementations for client and device operations.
//!
//! This module contains the concrete implementations of the strategy traits:
//!
//! - **FFI Strategy** (`ffi` module): Uses FFI bindings to the C INDIGO library (synchronous)
//! - **Async FFI Strategy** (`async_ffi` module): Async wrapper around FFI bindings
//! - **Rust Strategy** (`rs` module): Rust implementation of the INDIGO protocol
//!
//! The strategy to use can be selected at compile time via feature flags or at
//! runtime by choosing which strategy implementation to instantiate.

// FFI strategy - available when ffi-strategy feature is enabled
#[cfg(feature = "ffi")]
pub mod ffi;

// Async FFI strategy - available when ffi-strategy and async features are enabled
#[cfg(all(feature = "ffi", feature = "async"))]
pub mod async_ffi;

// Rust strategy - available when rs-strategy feature is enabled
#[cfg(feature = "rs")]
pub mod rs;

// Re-export strategy implementations
#[cfg(feature = "ffi")]
pub use ffi::FfiClientStrategy;

#[cfg(all(feature = "ffi", feature = "async"))]
pub use async_ffi::{AsyncFfiStrategy, PropertyStream};

#[cfg(feature = "rs")]
pub use rs::RsClientStrategy;

// TODO: Phase 4 - Implement device strategies
