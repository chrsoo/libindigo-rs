# Server Monitoring

## Overview

libindigo provides a comprehensive server monitoring system that tracks INDIGO server availability through two-level health checking:

1. **Host-level monitoring**: ICMP ping with TCP connect fallback
2. **Server-level monitoring**: INDIGO protocol handshake verification

The monitoring system uses a rolling window with configurable thresholds to smooth status transitions and provides event-based status reporting through the Client API.

## Quick Start

### Basic Monitoring Setup

```rust
use libindigo::client::{ClientBuilder, MonitoringConfig};
use std::net::SocketAddr;
use std::time::Duration;

let server_addr: SocketAddr = "192.168.1.50:7624".parse()?;

let client = ClientBuilder::new()
    .with_strategy(strategy)
    .with_monitoring(MonitoringConfig::new(server_addr))
    .build()?;

// Subscribe to status events
if let Some(mut rx) = client.subscribe_status() {
    while let Some(event) = rx.recv().await {
        match event {
            ClientEvent::ServerAvailable => println!("Server is available"),
            ClientEvent::ServerDegraded => println!("Server is degraded"),
            ClientEvent::ServerUnavailable => println!("Server is unavailable"),
        }
    }
}
```

### Custom Configuration

```rust
use libindigo::client::MonitoringConfig;
use std::time::Duration;

let config = MonitoringConfig::new(server_addr)
    .with_ping_interval(Duration::from_secs(3))
    .with_response_time_threshold(Duration::from_millis(500))
    .with_window_size(10)
    .with_connection_timeout(Duration::from_secs(5));

let client = ClientBuilder::new()
    .with_strategy(strategy)
    .with_monitoring(config)
    .build()?;
```

## Architecture

### Two-Level Monitoring

```
┌─────────────────────────────────────────────────────────┐
│                    ServerMonitor                         │
│                                                          │
│  ┌──────────────────┐         ┌──────────────────┐      │
│  │ HeartbeatChecker │         │  ServerChecker   │      │
│  │                  │         │                  │      │
│  │ • ICMP ping      │         │ • TCP handshake  │      │
│  │ • TCP fallback   │         │ • Protocol check │      │
│  │ • Every 2s       │         │ • Every ~10s     │      │
│  └────────┬─────────┘         └────────┬─────────┘      │
│           │                            │                │
│           └────────────┬───────────────┘                │
│                        ▼                                │
│              ┌──────────────────┐                        │
│              │  StatusTracker   │                        │
│              │                  │                        │
│              │ • Rolling window │                        │
│              │ • Thresholds     │                        │
│              │ • State machine  │                        │
│              └────────┬─────────┘                        │
│                       │                                  │
│                       ▼                                  │
│              Status Change Events                        │
└───────────────────────┼──────────────────────────────────┘
                        │
                        ▼
                Client Event Channel
```

### Status Model

The monitoring system uses a three-state availability model:

| Status | Description | Conditions |
|--------|-------------|------------|
| **Available** | Server is fully operational | Host reachable AND server responding AND response times normal |
| **Degraded** | Server is partially operational | Host reachable BUT (server not responding OR slow responses) |
| **Unavailable** | Server is not reachable | Host unreachable |

### State Machine

```
                    ┌──────────────┐
                    │ Unavailable  │ (Initial state)
                    └──────┬───────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        │ Host reachable   │ Host reachable   │
        │ Server responds  │ Server fails     │
        ▼                  ▼                  │
   ┌──────────┐      ┌──────────┐            │
   │Available │◄────►│ Degraded │            │
   └────┬─────┘      └────┬─────┘            │
        │                 │                  │
        │ Host fails      │ Host fails       │
        └─────────────────┴──────────────────┘
                          │
                          ▼
                    ┌──────────────┐
                    │ Unavailable  │
                    └──────────────┘
```

### Status Determination Logic

The status is computed from the rolling window of check results:

```
if recent_pings show >= 3 consecutive failures:
    status = Unavailable

else if recent_pings show >= 3 consecutive successes:
    if no handshake data yet:
        status = Degraded  // Host up, server unknown

    else if recent_handshakes show >= 3 consecutive successes:
        if any recent response times > threshold:
            status = Degraded  // Slow responses
        else:
            status = Available  // Everything good

    else:
        status = Degraded  // Host up, server not responding

else:
    status = previous_status  // Insufficient data
```

## Configuration

### MonitoringConfig

```rust
pub struct MonitoringConfig {
    /// Target server address
    pub server_addr: SocketAddr,

    /// Interval between ping checks (default: 2 seconds)
    pub ping_interval: Duration,

    /// Maximum acceptable response time (default: 1 second)
    pub response_time_threshold: Duration,

    /// Number of samples in rolling window (default: 5)
    pub window_size: usize,

    /// Whether to use ICMP ping (default: true, auto-disabled for localhost)
    pub use_icmp: bool,

    /// TCP connection timeout (default: 3 seconds)
    pub connection_timeout: Duration,
}
```

### Builder Methods

```rust
// Create with defaults
let config = MonitoringConfig::new(server_addr);

// Customize ping interval
let config = MonitoringConfig::new(server_addr)
    .with_ping_interval(Duration::from_secs(5));

// Customize response threshold
let config = MonitoringConfig::new(server_addr)
    .with_response_time_threshold(Duration::from_millis(500));

// Customize window size
let config = MonitoringConfig::new(server_addr)
    .with_window_size(10);

// Customize connection timeout
let config = MonitoringConfig::new(server_addr)
    .with_connection_timeout(Duration::from_secs(5));

// Force ICMP on/off (overrides localhost detection)
let config = MonitoringConfig::new(server_addr)
    .with_icmp(false);  // Force TCP-only

// Chain multiple settings
let config = MonitoringConfig::new(server_addr)
    .with_ping_interval(Duration::from_secs(3))
    .with_response_time_threshold(Duration::from_millis(800))
    .with_window_size(7)
    .with_icmp(false);
```

### Configuration Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `server_addr` | `SocketAddr` | Required | Target server address and port |
| `ping_interval` | `Duration` | 2s | Time between host-level pings |
| `response_time_threshold` | `Duration` | 1s | RTT above this marks response as slow |
| `window_size` | `usize` | 5 | Number of results in rolling window |
| `use_icmp` | `bool` | `true` | Use ICMP when available (auto-disabled for localhost) |
| `connection_timeout` | `Duration` | 3s | Timeout for individual check operations |

### Localhost Auto-Detection

When the target address is a loopback address (`127.0.0.1`, `::1`), ICMP is automatically disabled regardless of the `use_icmp` setting:

```rust
// ICMP will be auto-disabled for localhost
let config = MonitoringConfig::new("127.0.0.1:7624".parse()?);
assert_eq!(config.use_icmp, false);

// ICMP will be enabled for remote hosts
let config = MonitoringConfig::new("192.168.1.50:7624".parse()?);
assert_eq!(config.use_icmp, true);
```

**Rationale**:

- ICMP to localhost always succeeds (no useful signal)
- TCP connect is more meaningful for local server monitoring
- Avoids unnecessary privilege requirements

## Using with ClientBuilder

### Enable Monitoring

```rust
use libindigo::client::{ClientBuilder, MonitoringConfig};

let client = ClientBuilder::new()
    .with_strategy(strategy)
    .with_monitoring(MonitoringConfig::new(server_addr))
    .build()?;
```

### Subscribe to Status Events

```rust
use libindigo::client::ClientEvent;

if let Some(mut rx) = client.subscribe_status() {
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                ClientEvent::ServerAvailable => {
                    println!("✓ Server is available");
                }
                ClientEvent::ServerDegraded => {
                    println!("⚠ Server is degraded");
                }
                ClientEvent::ServerUnavailable => {
                    println!("✗ Server is unavailable");
                }
            }
        }
    });
}
```

### Multiple Subscribers

Multiple tasks can subscribe to status events:

```rust
// Subscriber 1: Log to console
let mut rx1 = client.subscribe_status().unwrap();
tokio::spawn(async move {
    while let Some(event) = rx1.recv().await {
        println!("Status: {:?}", event);
    }
});

// Subscriber 2: Update UI
let mut rx2 = client.subscribe_status().unwrap();
tokio::spawn(async move {
    while let Some(event) = rx2.recv().await {
        update_ui(event);
    }
});
```

## Monitoring Events

### ClientEvent Enum

```rust
pub enum ClientEvent {
    /// Server is fully available and responding normally
    ServerAvailable,

    /// Server is in degraded state
    /// (host reachable but server not responding properly, or high latency)
    ServerDegraded,

    /// Server is unreachable (host not responding)
    ServerUnavailable,
}
```

### Event Delivery

- Events are delivered via `tokio::sync::mpsc::UnboundedReceiver`
- Only status **changes** are emitted (not every check result)
- Events are broadcast to all subscribers
- Low-level ping/handshake results are logged but not emitted as events

## Monitoring Checks

### Host-Level Checks (HeartbeatChecker)

Performed every `ping_interval` (default: 2 seconds):

1. **ICMP Ping** (if enabled and not localhost):
   - Sends ICMP Echo Request
   - Measures round-trip time
   - Falls back to TCP on failure

2. **TCP Connect** (fallback or primary):
   - Attempts TCP connection to server port
   - Measures connection time
   - Used when ICMP unavailable or disabled

**Logging**:

- `TRACE`: Every ping request/response with RTT
- `DEBUG`: Ping failures with reason
- `WARN`: ICMP unavailable, falling back to TCP

### Server-Level Checks (ServerChecker)

Performed approximately every 5 pings (~10 seconds with default settings):

1. **TCP Handshake**:
   - Opens TCP connection to INDIGO server port
   - Reads initial greeting
   - Verifies INDIGO protocol markers

2. **Protocol Verification**:
   - Checks for `<getProperties` (XML) or `"getProperties"` (JSON)
   - Validates server is responding with INDIGO protocol

**Logging**:

- `TRACE`: Every handshake attempt
- `DEBUG`: Handshake failures with reason

### Status Tracking (StatusTracker)

Maintains rolling windows of check results:

- **Ping Window**: Last N ping results
- **Handshake Window**: Last N handshake results
- **Window Size**: Configurable (default: 5)

**Status Computation**:

- Examines last 3 results in each window
- Requires 3 consecutive successes/failures for transitions
- Considers response times against threshold

**Logging**:

- `TRACE`: Window state changes
- `INFO`: Status transitions

## Logging Integration

Monitoring uses the [tracing framework](logging.md) with specific log levels:

### Recommended Log Levels

```bash
# Production: Only status changes
RUST_LOG=libindigo_rs::monitoring=info

# Troubleshooting: Include check failures
RUST_LOG=libindigo_rs::monitoring=debug

# Deep debugging: All ping/handshake details
RUST_LOG=libindigo_rs::monitoring=trace
```

### Log Output Examples

```
# INFO level - Status changes only
2026-03-12T06:45:00.000Z  INFO libindigo_rs::monitoring::monitor: Starting monitoring for 192.168.1.50:7624 (ping_interval=2s, window_size=5)
2026-03-12T06:45:15.123Z  INFO libindigo_rs::monitoring::monitor: Status changed from Unavailable to Degraded
2026-03-12T06:45:30.456Z  INFO libindigo_rs::monitoring::monitor: Status changed from Degraded to Available

# DEBUG level - Includes check failures
2026-03-12T06:45:05.789Z DEBUG libindigo_rs::monitoring::heartbeat: Heartbeat check failed for 192.168.1.50: connection refused
2026-03-12T06:45:10.012Z DEBUG libindigo_rs::monitoring::server_check: Server handshake failed for 192.168.1.50:7624: timeout after 3s

# TRACE level - All details
2026-03-12T06:45:02.345Z TRACE libindigo_rs::monitoring::heartbeat: ICMP ping request sent to 192.168.1.50
2026-03-12T06:45:02.357Z TRACE libindigo_rs::monitoring::heartbeat: ICMP ping response received from 192.168.1.50: rtt=12ms
2026-03-12T06:45:12.678Z TRACE libindigo_rs::monitoring::server_check: TCP handshake attempt to 192.168.1.50:7624
2026-03-12T06:45:12.690Z TRACE libindigo_rs::monitoring::status: Window state changed: Unavailable -> Degraded
```

## FFI Usage (C/C++)

The monitoring system is available to C/C++ consumers through the FFI interface.

### C Types

```c
// Availability status enum
typedef enum {
    FfiAvailabilityStatus_Available = 0,
    FfiAvailabilityStatus_Degraded = 1,
    FfiAvailabilityStatus_Unavailable = 2,
} FfiAvailabilityStatus;

// Monitoring configuration
typedef struct {
    const char* host;              // Server IP address (e.g., "192.168.1.50")
    uint16_t port;                 // Server port (e.g., 7624)
    uint32_t ping_interval_ms;     // Ping interval in ms (0 = default 2000)
    uint32_t response_threshold_ms; // Response threshold in ms (0 = default 1000)
    uint32_t window_size;          // Window size (0 = default 5)
    bool use_icmp;                 // Use ICMP ping
} FfiMonitoringConfig;

// Status callback type
typedef void (*FfiStatusCallback)(
    FfiAvailabilityStatus previous,
    FfiAvailabilityStatus current,
    void* user_data
);
```

### C API Functions

```c
// Set monitoring configuration
int indigo_set_monitoring_config(const FfiMonitoringConfig* config);

// Register status callback
int indigo_set_status_callback(FfiStatusCallback callback, void* user_data);
```

### C Example

```c
#include <stdio.h>
#include "libindigo_ffi.h"

void status_callback(FfiAvailabilityStatus previous,
                     FfiAvailabilityStatus current,
                     void* user_data) {
    const char* status_names[] = {"Available", "Degraded", "Unavailable"};
    printf("Status changed: %s -> %s\n",
           status_names[previous],
           status_names[current]);
}

int main() {
    // Configure monitoring
    FfiMonitoringConfig config = {
        .host = "192.168.1.50",
        .port = 7624,
        .ping_interval_ms = 2000,
        .response_threshold_ms = 1000,
        .window_size = 5,
        .use_icmp = true,
    };

    if (indigo_set_monitoring_config(&config) != 0) {
        fprintf(stderr, "Failed to set monitoring config\n");
        return 1;
    }

    // Register callback
    if (indigo_set_status_callback(status_callback, NULL) != 0) {
        fprintf(stderr, "Failed to set status callback\n");
        return 1;
    }

    // ... rest of application

    return 0;
}
```

### C++ Example

```cpp
#include <iostream>
#include "libindigo_ffi.h"

class ServerMonitor {
public:
    ServerMonitor(const std::string& host, uint16_t port) {
        FfiMonitoringConfig config = {
            .host = host.c_str(),
            .port = port,
            .ping_interval_ms = 2000,
            .response_threshold_ms = 1000,
            .window_size = 5,
            .use_icmp = true,
        };

        indigo_set_monitoring_config(&config);
        indigo_set_status_callback(status_callback_wrapper, this);
    }

    void on_status_change(FfiAvailabilityStatus previous,
                         FfiAvailabilityStatus current) {
        std::cout << "Status: " << status_name(previous)
                  << " -> " << status_name(current) << std::endl;
    }

private:
    static void status_callback_wrapper(FfiAvailabilityStatus previous,
                                       FfiAvailabilityStatus current,
                                       void* user_data) {
        auto* monitor = static_cast<ServerMonitor*>(user_data);
        monitor->on_status_change(previous, current);
    }

    static const char* status_name(FfiAvailabilityStatus status) {
        switch (status) {
            case FfiAvailabilityStatus_Available: return "Available";
            case FfiAvailabilityStatus_Degraded: return "Degraded";
            case FfiAvailabilityStatus_Unavailable: return "Unavailable";
            default: return "Unknown";
        }
    }
};
```

## Examples

### Basic Monitoring

```rust
use libindigo::client::{ClientBuilder, ClientEvent, MonitoringConfig};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_addr: SocketAddr = "192.168.1.50:7624".parse()?;

    let client = ClientBuilder::new()
        .with_strategy(strategy)
        .with_monitoring(MonitoringConfig::new(server_addr))
        .build()?;

    if let Some(mut rx) = client.subscribe_status() {
        while let Some(event) = rx.recv().await {
            match event {
                ClientEvent::ServerAvailable => println!("✓ Server available"),
                ClientEvent::ServerDegraded => println!("⚠ Server degraded"),
                ClientEvent::ServerUnavailable => println!("✗ Server unavailable"),
            }
        }
    }

    Ok(())
}
```

### Custom Configuration

```rust
use libindigo::client::{ClientBuilder, MonitoringConfig};
use std::time::Duration;

let config = MonitoringConfig::new(server_addr)
    .with_ping_interval(Duration::from_secs(3))
    .with_response_time_threshold(Duration::from_millis(500))
    .with_window_size(10)
    .with_connection_timeout(Duration::from_secs(5));

let client = ClientBuilder::new()
    .with_strategy(strategy)
    .with_monitoring(config)
    .build()?;
```

### TCP-Only Monitoring

```rust
// Force TCP-only (no ICMP)
let config = MonitoringConfig::new(server_addr)
    .with_icmp(false);

let client = ClientBuilder::new()
    .with_strategy(strategy)
    .with_monitoring(config)
    .build()?;
```

### Monitoring with Logging

```rust
use libindigo::client::ClientBuilder;
use libindigo::logging::{LogConfig, LogLevel};

let client = ClientBuilder::new()
    .with_strategy(strategy)
    .with_logging(
        LogConfig::default()
            .with_level(LogLevel::Debug)
    )
    .with_monitoring(MonitoringConfig::new(server_addr))
    .build()?;
```

## Troubleshooting

### No Status Events

**Problem**: Not receiving status change events.

**Solutions**:

1. Verify monitoring is enabled via `with_monitoring()`
2. Check that `subscribe_status()` returns `Some(rx)`
3. Ensure the receiver is being polled (`.recv().await`)
4. Check logs for monitoring initialization

### ICMP Permission Denied

**Problem**: ICMP ping fails with permission errors.

**Solutions**:

1. Run with elevated privileges (not recommended for production)
2. Use TCP-only mode: `.with_icmp(false)`
3. On Linux, grant `CAP_NET_RAW` capability
4. Monitoring will automatically fall back to TCP

### Slow Status Transitions

**Problem**: Status changes take too long to detect.

**Solutions**:

1. Reduce `ping_interval` for faster checks
2. Reduce `window_size` for quicker transitions
3. Note: Smaller windows are more sensitive to transient failures

### False Degraded Status

**Problem**: Server shows as degraded when it's actually available.

**Solutions**:

1. Increase `response_time_threshold` if network is slow
2. Increase `connection_timeout` for slow networks
3. Check server logs for actual performance issues
4. Verify network latency with `ping` command

### Monitoring Not Starting

**Problem**: Monitoring doesn't seem to be running.

**Solutions**:

1. Verify `monitoring` feature is enabled in `Cargo.toml`
2. Check that strategy supports monitoring (RS strategy does)
3. Enable debug logging: `RUST_LOG=libindigo_rs::monitoring=debug`
4. Check for errors in `ClientBuilder::build()`

## Performance Considerations

### Network Traffic

Default configuration generates:

- **Ping checks**: Every 2 seconds (~30 checks/minute)
- **Handshake checks**: Every ~10 seconds (~6 checks/minute)
- **Total**: ~36 checks/minute per monitored server

To reduce traffic:

```rust
let config = MonitoringConfig::new(server_addr)
    .with_ping_interval(Duration::from_secs(5));  // Reduce frequency
```

### CPU Usage

Monitoring uses minimal CPU:

- Background tokio task
- Async I/O (no blocking)
- Efficient rolling window

### Memory Usage

Per monitored server:

- ~1KB for configuration
- ~100 bytes per window entry
- Default: ~500 bytes for windows (5 entries each)

## API Reference

### Types

- [`MonitoringConfig`](../src/client/monitoring.rs:83) - Monitoring configuration
- [`AvailabilityStatus`](../src/client/monitoring.rs:12) - Server availability status
- [`ClientEvent`](../src/client/monitoring.rs:62) - Client-level monitoring events
- [`ServerMonitor`](../rs/src/monitoring/monitor.rs:15) - Monitoring orchestrator (internal)

### Functions

- [`MonitoringConfig::new()`](../src/client/monitoring.rs:115) - Create configuration
- [`MonitoringConfig::with_ping_interval()`](../src/client/monitoring.rs:133) - Set ping interval
- [`MonitoringConfig::with_response_time_threshold()`](../src/client/monitoring.rs:139) - Set response threshold
- [`MonitoringConfig::with_window_size()`](../src/client/monitoring.rs:145) - Set window size
- [`MonitoringConfig::with_connection_timeout()`](../src/client/monitoring.rs:151) - Set timeout
- [`MonitoringConfig::with_icmp()`](../src/client/monitoring.rs:157) - Enable/disable ICMP
- [`Client::subscribe_status()`](../src/client/builder.rs:264) - Subscribe to status events

### FFI Functions

- [`indigo_set_monitoring_config()`](../ffi/src/monitoring.rs:224) - Set monitoring config (C)
- [`indigo_set_status_callback()`](../ffi/src/monitoring.rs:285) - Register callback (C)

## See Also

- [Logging Documentation](logging.md) - Configure logging for monitoring
- [Client Builder API](../src/client/builder.rs) - Client construction
- [Architecture Plan](../plans/server-monitoring-architecture.md) - Detailed architecture
- [Client Strategies](architecture/client-strategies.md) - Strategy implementations

---

**Last Updated**: 2026-03-12
**Version**: 0.2.0
