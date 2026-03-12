//! INDIGO server monitoring implementation.
//!
//! Provides two-level monitoring:
//! 1. Host availability via ICMP ping (with TCP fallback)
//! 2. Server availability via TCP handshake

#[cfg(feature = "monitoring")]
mod heartbeat;
#[cfg(feature = "monitoring")]
mod monitor;
#[cfg(feature = "monitoring")]
mod server_check;
#[cfg(feature = "monitoring")]
mod status;

#[cfg(feature = "monitoring")]
pub use monitor::ServerMonitor;
