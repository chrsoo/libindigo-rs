//! Monitoring types for INDIGO server availability tracking.
//!
//! Provides a two-level monitoring system:
//! 1. Host monitoring: ICMP ping with TCP connect fallback
//! 2. Server monitoring: TCP handshake to INDIGO server port

use std::net::SocketAddr;
use std::time::Duration;

/// Server availability status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvailabilityStatus {
    /// Host is reachable and INDIGO server is responding
    Available,
    /// Host is reachable but server is not responding, or high latency
    Degraded,
    /// Host is unreachable
    Unavailable,
}

impl std::fmt::Display for AvailabilityStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AvailabilityStatus::Available => write!(f, "Available"),
            AvailabilityStatus::Degraded => write!(f, "Degraded"),
            AvailabilityStatus::Unavailable => write!(f, "Unavailable"),
        }
    }
}

impl Default for AvailabilityStatus {
    fn default() -> Self {
        AvailabilityStatus::Unavailable
    }
}

/// Events emitted by the monitoring system (internal).
///
/// These are low-level events used internally by the monitoring implementation.
/// The Client API exposes only high-level `ClientEvent` variants.
#[derive(Debug, Clone)]
pub enum MonitoringEvent {
    /// Server status has changed
    StatusChanged {
        previous: AvailabilityStatus,
        current: AvailabilityStatus,
    },
    /// A ping result was received (low-level, hidden from Client API)
    PingResult {
        success: bool,
        duration: Option<Duration>,
    },
    /// A server handshake result (low-level, hidden from Client API)
    HandshakeResult { success: bool },
}

/// Client-level events for server monitoring.
///
/// These are the high-level events exposed through the Client API.
/// Only status changes are published; low-level ping/handshake results are logged but not emitted.
#[derive(Debug, Clone)]
pub enum ClientEvent {
    /// Server is fully available and responding normally.
    ServerAvailable,
    /// Server is in degraded state (host reachable but server not responding properly, or high latency).
    ServerDegraded,
    /// Server is unreachable (host not responding).
    ServerUnavailable,
}

impl ClientEvent {
    /// Convert an AvailabilityStatus to a ClientEvent.
    pub fn from_status(status: AvailabilityStatus) -> Self {
        match status {
            AvailabilityStatus::Available => ClientEvent::ServerAvailable,
            AvailabilityStatus::Degraded => ClientEvent::ServerDegraded,
            AvailabilityStatus::Unavailable => ClientEvent::ServerUnavailable,
        }
    }
}

/// Configuration for the monitoring system.
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// Target server address
    pub server_addr: SocketAddr,
    /// Interval between ping checks. Default: 2 seconds.
    pub ping_interval: Duration,
    /// Maximum acceptable response time. Default: 1 second.
    pub response_time_threshold: Duration,
    /// Number of samples in the rolling window. Default: 5.
    pub window_size: usize,
    /// Whether to use ICMP ping (auto-disabled for localhost). Default: true.
    pub use_icmp: bool,
    /// TCP connection timeout. Default: 3 seconds.
    pub connection_timeout: Duration,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            server_addr: "127.0.0.1:7624".parse().unwrap(),
            ping_interval: Duration::from_secs(2),
            response_time_threshold: Duration::from_secs(1),
            window_size: 5,
            use_icmp: true,
            connection_timeout: Duration::from_secs(3),
        }
    }
}

impl MonitoringConfig {
    /// Create a new monitoring config for the given server address.
    /// Automatically disables ICMP for localhost addresses.
    pub fn new(server_addr: SocketAddr) -> Self {
        let is_localhost = Self::is_localhost(&server_addr);
        Self {
            server_addr,
            use_icmp: !is_localhost,
            ..Default::default()
        }
    }

    /// Check if the address is a localhost address (127.0.0.1, ::1).
    pub fn is_localhost(addr: &SocketAddr) -> bool {
        match addr {
            SocketAddr::V4(v4) => v4.ip().is_loopback(),
            SocketAddr::V6(v6) => v6.ip().is_loopback(),
        }
    }

    /// Set the ping interval.
    pub fn with_ping_interval(mut self, interval: Duration) -> Self {
        self.ping_interval = interval;
        self
    }

    /// Set the response time threshold.
    pub fn with_response_time_threshold(mut self, threshold: Duration) -> Self {
        self.response_time_threshold = threshold;
        self
    }

    /// Set the rolling window size.
    pub fn with_window_size(mut self, size: usize) -> Self {
        self.window_size = size;
        self
    }

    /// Set the TCP connection timeout.
    pub fn with_connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// Force enable or disable ICMP.
    pub fn with_icmp(mut self, enabled: bool) -> Self {
        self.use_icmp = enabled;
        self
    }
}
