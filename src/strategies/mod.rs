//! Strategy implementations for client and device operations.
//!
//! This module contains the concrete implementations of the strategy traits:
//!
//! - **FFI Strategy** (`ffi` module): Uses FFI bindings to the C INDIGO library
//! - **Rust Strategy** (`rs` module): Pure Rust implementation of the INDIGO protocol
//!
//! The strategy to use can be selected at compile time via feature flags or at
//! runtime by choosing which strategy implementation to instantiate.

// FFI strategy - available when ffi-strategy feature is enabled
#[cfg(feature = "ffi-strategy")]
pub mod ffi;

// Pure Rust strategy - available when rs-strategy feature is enabled
#[cfg(feature = "rs-strategy")]
pub mod rs;

// Re-export strategy implementations
#[cfg(feature = "ffi-strategy")]
pub use ffi::FfiClientStrategy;

#[cfg(feature = "rs-strategy")]
pub use rs::RsClientStrategy;

// TODO: Phase 2 - Implement FfiClientStrategy
// TODO: Phase 3 - Implement RsClientStrategy
// TODO: Phase 4 - Implement device strategies
