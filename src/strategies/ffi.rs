//! FFI-based strategy implementation.
//!
//! This module provides a client strategy that uses FFI bindings to the
//! C INDIGO library via `libindigo-sys`.
//!
//! # Phase 2 Implementation
//!
//! This is a placeholder for Phase 2 implementation. The FFI strategy will:
//!
//! - Wrap synchronous FFI calls in `tokio::task::spawn_blocking`
//! - Convert C callbacks to async streams using channels
//! - Manage memory safety around FFI boundaries
//! - Handle string conversions between Rust and C

use crate::client::ClientStrategy;
use crate::error::{IndigoError, Result};
use crate::types::Property;
use async_trait::async_trait;

/// FFI-based client strategy.
///
/// This strategy uses the C INDIGO library via FFI bindings.
///
/// # TODO: Phase 2
///
/// - Implement connection management
/// - Set up callback handlers
/// - Create property event streams
/// - Handle memory management
pub struct FfiClientStrategy {
    // TODO: Phase 2 - Add fields for FFI client state
    // inner: Arc<Mutex<FfiClientInner>>,
}

impl FfiClientStrategy {
    /// Creates a new FFI client strategy.
    ///
    /// # TODO: Phase 2
    ///
    /// Initialize FFI client structures and set up callbacks.
    #[allow(dead_code)]
    pub fn new() -> Self {
        FfiClientStrategy {
            // TODO: Phase 2 - Initialize
        }
    }
}

#[async_trait]
impl ClientStrategy for FfiClientStrategy {
    async fn connect(&mut self, _url: &str) -> Result<()> {
        // TODO: Phase 2 - Implement FFI connection
        Err(IndigoError::NotSupported(
            "FFI strategy not yet implemented (Phase 2)".to_string(),
        ))
    }

    async fn disconnect(&mut self) -> Result<()> {
        // TODO: Phase 2 - Implement FFI disconnection
        Err(IndigoError::NotSupported(
            "FFI strategy not yet implemented (Phase 2)".to_string(),
        ))
    }

    async fn enumerate_properties(&mut self, _device: Option<&str>) -> Result<()> {
        // TODO: Phase 2 - Implement property enumeration
        Err(IndigoError::NotSupported(
            "FFI strategy not yet implemented (Phase 2)".to_string(),
        ))
    }

    async fn send_property(&mut self, _property: Property) -> Result<()> {
        // TODO: Phase 2 - Implement property sending
        Err(IndigoError::NotSupported(
            "FFI strategy not yet implemented (Phase 2)".to_string(),
        ))
    }
}
