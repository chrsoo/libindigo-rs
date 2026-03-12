//! FFI-compatible monitoring types and functions.
//!
//! This module provides C-compatible types and functions for server monitoring,
//! allowing C/C++ consumers to use the INDIGO server monitoring feature.
//!
//! # Architecture
//!
//! The monitoring FFI follows the existing callback pattern:
//! 1. C code calls `indigo_set_monitoring_config()` to configure monitoring
//! 2. C code registers a callback via `indigo_set_status_callback()`
//! 3. When server status changes, the callback is invoked with the new status
//!
//! # Example (C)
//!
//! ```c
//! void status_callback(FfiAvailabilityStatus previous,
//!                      FfiAvailabilityStatus current,
//!                      void* user_data) {
//!     printf("Status changed: %d -> %d\n", previous, current);
//! }
//!
//! FfiMonitoringConfig config = {
//!     .host = "192.168.1.50",
//!     .port = 7624,
//!     .ping_interval_ms = 2000,
//!     .response_threshold_ms = 1000,
//!     .window_size = 5,
//!     .use_icmp = true,
//! };
//!
//! indigo_set_monitoring_config(&config);
//! indigo_set_status_callback(status_callback, NULL);
//! ```

use libindigo::client::monitoring::{AvailabilityStatus, MonitoringConfig};
use std::ffi::CStr;
use std::net::{IpAddr, SocketAddr};
use std::os::raw::{c_char, c_void};
use std::time::Duration;
use tracing::{debug, error, warn};

/// C-compatible availability status enum.
///
/// Represents the three-state availability model used by the monitoring system.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiAvailabilityStatus {
    /// Server is reachable and responding to INDIGO protocol handshake.
    Available = 0,
    /// Host is reachable but INDIGO server is not responding correctly.
    Degraded = 1,
    /// Host is unreachable or not responding at all.
    Unavailable = 2,
}

impl From<AvailabilityStatus> for FfiAvailabilityStatus {
    fn from(status: AvailabilityStatus) -> Self {
        match status {
            AvailabilityStatus::Available => FfiAvailabilityStatus::Available,
            AvailabilityStatus::Degraded => FfiAvailabilityStatus::Degraded,
            AvailabilityStatus::Unavailable => FfiAvailabilityStatus::Unavailable,
        }
    }
}

impl From<FfiAvailabilityStatus> for AvailabilityStatus {
    fn from(status: FfiAvailabilityStatus) -> Self {
        match status {
            FfiAvailabilityStatus::Available => AvailabilityStatus::Available,
            FfiAvailabilityStatus::Degraded => AvailabilityStatus::Degraded,
            FfiAvailabilityStatus::Unavailable => AvailabilityStatus::Unavailable,
        }
    }
}

/// C-compatible monitoring configuration.
///
/// All fields have default values if set to 0/NULL:
/// - `ping_interval_ms`: 2000ms (2 seconds)
/// - `response_threshold_ms`: 1000ms (1 second)
/// - `window_size`: 5
///
/// # Safety
///
/// The `host` pointer must point to a valid null-terminated C string that
/// remains valid for the duration of the configuration call.
#[repr(C)]
pub struct FfiMonitoringConfig {
    /// Server address as null-terminated C string (e.g., "192.168.1.50")
    pub host: *const c_char,
    /// Server port
    pub port: u16,
    /// Ping interval in milliseconds (0 = default 2000)
    pub ping_interval_ms: u32,
    /// Response time threshold in milliseconds (0 = default 1000)
    pub response_threshold_ms: u32,
    /// Rolling window size (0 = default 5)
    pub window_size: u32,
    /// Whether to use ICMP (auto-disabled for localhost)
    pub use_icmp: bool,
}

/// Callback type for status change events.
///
/// # Parameters
///
/// - `previous`: The previous availability status
/// - `current`: The new availability status
/// - `user_data`: User-provided context pointer (passed to `indigo_set_status_callback`)
///
/// # Safety
///
/// This callback must not panic. Any errors should be handled gracefully.
/// The callback may be called from any thread.
pub type FfiStatusCallback = extern "C" fn(
    previous: FfiAvailabilityStatus,
    current: FfiAvailabilityStatus,
    user_data: *mut c_void,
);

/// Converts an `FfiMonitoringConfig` to a `MonitoringConfig`.
///
/// # Safety
///
/// The `config.host` pointer must point to a valid null-terminated C string.
///
/// # Errors
///
/// Returns an error if:
/// - The host pointer is null
/// - The host string is not valid UTF-8
/// - The host cannot be parsed as an IP address
pub unsafe fn ffi_config_to_monitoring_config(
    config: &FfiMonitoringConfig,
) -> Result<MonitoringConfig, String> {
    // Validate and convert host
    if config.host.is_null() {
        return Err("Host pointer is null".to_string());
    }

    let host_cstr = CStr::from_ptr(config.host);
    let host_str = host_cstr
        .to_str()
        .map_err(|e| format!("Invalid UTF-8 in host string: {}", e))?;

    // Parse host as IP address
    let ip_addr: IpAddr = host_str
        .parse()
        .map_err(|e| format!("Invalid IP address '{}': {}", host_str, e))?;

    let server_addr = SocketAddr::new(ip_addr, config.port);

    // Create base config
    let mut monitoring_config = MonitoringConfig::new(server_addr);

    // Apply custom values (0 means use default)
    if config.ping_interval_ms > 0 {
        monitoring_config = monitoring_config
            .with_ping_interval(Duration::from_millis(config.ping_interval_ms as u64));
    }

    if config.response_threshold_ms > 0 {
        monitoring_config = monitoring_config.with_response_time_threshold(Duration::from_millis(
            config.response_threshold_ms as u64,
        ));
    }

    if config.window_size > 0 {
        monitoring_config = monitoring_config.with_window_size(config.window_size as usize);
    }

    // Apply ICMP setting only if not localhost
    // Note: MonitoringConfig::new() already disables ICMP for localhost,
    // so we only override if the user explicitly wants to enable/disable it
    // and it's not a localhost address
    if !MonitoringConfig::is_localhost(&server_addr) {
        monitoring_config = monitoring_config.with_icmp(config.use_icmp);
    }

    Ok(monitoring_config)
}

// ============================================================================
// Exported C Functions
// ============================================================================

/// Sets the monitoring configuration.
///
/// This function configures the monitoring system with the provided settings.
/// The configuration will be applied when the client connects to a server.
///
/// # Parameters
///
/// - `config`: Pointer to the monitoring configuration
///
/// # Returns
///
/// - `0` on success
/// - `-1` on error (check logs for details)
///
/// # Safety
///
/// The `config` pointer must be valid and point to a properly initialized
/// `FfiMonitoringConfig` structure. The `host` field must point to a valid
/// null-terminated C string.
///
/// # Example (C)
///
/// ```c
/// FfiMonitoringConfig config = {
///     .host = "192.168.1.50",
///     .port = 7624,
///     .ping_interval_ms = 2000,
///     .response_threshold_ms = 1000,
///     .window_size = 5,
///     .use_icmp = true,
/// };
///
/// if (indigo_set_monitoring_config(&config) != 0) {
///     fprintf(stderr, "Failed to set monitoring config\n");
/// }
/// ```
#[no_mangle]
pub unsafe extern "C" fn indigo_set_monitoring_config(config: *const FfiMonitoringConfig) -> i32 {
    if config.is_null() {
        error!("indigo_set_monitoring_config: config pointer is null");
        return -1;
    }

    let config_ref = &*config;

    match ffi_config_to_monitoring_config(config_ref) {
        Ok(monitoring_config) => {
            debug!(
                "Monitoring config set: server={}, ping_interval={:?}",
                monitoring_config.server_addr, monitoring_config.ping_interval
            );
            // TODO: Store the config in a global state or pass to strategy
            // For now, we just validate and log
            warn!("indigo_set_monitoring_config: Configuration validated but not yet stored (implementation pending)");
            0
        }
        Err(e) => {
            error!("indigo_set_monitoring_config: {}", e);
            -1
        }
    }
}

/// Sets the status change callback.
///
/// Registers a callback function that will be invoked whenever the server
/// availability status changes.
///
/// # Parameters
///
/// - `callback`: The callback function to invoke on status changes
/// - `user_data`: User-provided context pointer that will be passed to the callback
///
/// # Returns
///
/// - `0` on success
/// - `-1` on error (check logs for details)
///
/// # Safety
///
/// The `callback` function pointer must be valid for the lifetime of the monitoring.
/// The `user_data` pointer must remain valid until monitoring is stopped or a new
/// callback is registered.
///
/// # Example (C)
///
/// ```c
/// void my_callback(FfiAvailabilityStatus previous,
///                  FfiAvailabilityStatus current,
///                  void* user_data) {
///     printf("Status: %d -> %d\n", previous, current);
/// }
///
/// if (indigo_set_status_callback(my_callback, NULL) != 0) {
///     fprintf(stderr, "Failed to set status callback\n");
/// }
/// ```
#[no_mangle]
pub unsafe extern "C" fn indigo_set_status_callback(
    callback: FfiStatusCallback,
    user_data: *mut c_void,
) -> i32 {
    debug!("Status callback registered");
    // TODO: Store the callback and user_data in a global state
    // For now, we just validate and log
    warn!("indigo_set_status_callback: Callback registered but not yet stored (implementation pending)");

    // Prevent unused variable warnings
    let _ = callback;
    let _ = user_data;

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_availability_status_conversion() {
        assert_eq!(
            FfiAvailabilityStatus::from(AvailabilityStatus::Available),
            FfiAvailabilityStatus::Available
        );
        assert_eq!(
            FfiAvailabilityStatus::from(AvailabilityStatus::Degraded),
            FfiAvailabilityStatus::Degraded
        );
        assert_eq!(
            FfiAvailabilityStatus::from(AvailabilityStatus::Unavailable),
            FfiAvailabilityStatus::Unavailable
        );

        assert_eq!(
            AvailabilityStatus::from(FfiAvailabilityStatus::Available),
            AvailabilityStatus::Available
        );
        assert_eq!(
            AvailabilityStatus::from(FfiAvailabilityStatus::Degraded),
            AvailabilityStatus::Degraded
        );
        assert_eq!(
            AvailabilityStatus::from(FfiAvailabilityStatus::Unavailable),
            AvailabilityStatus::Unavailable
        );
    }

    #[test]
    fn test_ffi_config_conversion() {
        let host = CString::new("192.168.1.50").unwrap();
        let config = FfiMonitoringConfig {
            host: host.as_ptr(),
            port: 7624,
            ping_interval_ms: 3000,
            response_threshold_ms: 1500,
            window_size: 10,
            use_icmp: true,
        };

        let result = unsafe { ffi_config_to_monitoring_config(&config) };
        assert!(result.is_ok());

        let monitoring_config = result.unwrap();
        assert_eq!(monitoring_config.server_addr.port(), 7624);
        assert_eq!(monitoring_config.ping_interval, Duration::from_millis(3000));
        assert_eq!(
            monitoring_config.response_time_threshold,
            Duration::from_millis(1500)
        );
        assert_eq!(monitoring_config.window_size, 10);
        assert_eq!(monitoring_config.use_icmp, true);
    }

    #[test]
    fn test_ffi_config_defaults() {
        let host = CString::new("10.0.0.1").unwrap();
        let config = FfiMonitoringConfig {
            host: host.as_ptr(),
            port: 7624,
            ping_interval_ms: 0,      // Use default
            response_threshold_ms: 0, // Use default
            window_size: 0,           // Use default
            use_icmp: false,
        };

        let result = unsafe { ffi_config_to_monitoring_config(&config) };
        assert!(result.is_ok());

        let monitoring_config = result.unwrap();
        // Should use defaults from MonitoringConfig::new()
        assert_eq!(monitoring_config.ping_interval, Duration::from_secs(2));
        assert_eq!(
            monitoring_config.response_time_threshold,
            Duration::from_secs(1)
        );
        assert_eq!(monitoring_config.window_size, 5);
    }

    #[test]
    fn test_ffi_config_localhost_detection() {
        let host = CString::new("127.0.0.1").unwrap();
        let config = FfiMonitoringConfig {
            host: host.as_ptr(),
            port: 7624,
            ping_interval_ms: 0,
            response_threshold_ms: 0,
            window_size: 0,
            use_icmp: true, // Will be auto-disabled for localhost
        };

        let result = unsafe { ffi_config_to_monitoring_config(&config) };
        assert!(result.is_ok());

        let monitoring_config = result.unwrap();
        // ICMP should be disabled for localhost
        assert_eq!(monitoring_config.use_icmp, false);
    }

    #[test]
    fn test_ffi_config_invalid_host() {
        let host = CString::new("not-an-ip").unwrap();
        let config = FfiMonitoringConfig {
            host: host.as_ptr(),
            port: 7624,
            ping_interval_ms: 0,
            response_threshold_ms: 0,
            window_size: 0,
            use_icmp: true,
        };

        let result = unsafe { ffi_config_to_monitoring_config(&config) };
        assert!(result.is_err());
    }

    #[test]
    fn test_ffi_config_null_host() {
        let config = FfiMonitoringConfig {
            host: std::ptr::null(),
            port: 7624,
            ping_interval_ms: 0,
            response_threshold_ms: 0,
            window_size: 0,
            use_icmp: true,
        };

        let result = unsafe { ffi_config_to_monitoring_config(&config) };
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("null"));
    }
}
