# Mission Control CLI — Monitoring Migration Plan

## Overview

This plan describes how to migrate Mission Control CLI's custom server monitoring implementation to use the `libindigo` monitoring feature. The goal is to eliminate duplicated monitoring code from Mission Control while preserving the existing `AppEvent`-driven architecture and UI/reconnection behavior.

**Target**: Replace `run_heartbeat` and related types in Mission Control with `libindigo`'s `ClientEvent` subscription.

**Not tracked in GitHub issues** — this plan is intended for the Mission Control Roo instance to execute directly.

---

## 1. Event Mapping

### 1.1 Status Events

| Mission Control Event | libindigo Event | Notes |
|---|---|---|
| `AppEvent::HeartbeatAvailable` | `ClientEvent::ServerAvailable` | Host reachable AND server responding AND response times normal |
| `AppEvent::HeartbeatDegraded` | `ClientEvent::ServerDegraded` | Host reachable BUT server not responding OR slow responses |
| `AppEvent::HeartbeatUnavailable` | `ClientEvent::ServerUnavailable` | Host unreachable |

### 1.2 Status Model Comparison

Both systems use the same three-state model:

| Status | Mission Control Definition | libindigo Definition | Compatible? |
|---|---|---|---|
| Available | 5 consecutive successful pings + response < 1s | 3+ recent pings succeed + handshake succeeds + RTT < threshold | ✅ Yes |
| Degraded | Mixed results or high latency | Host reachable but server not responding or slow | ✅ Yes |
| Unavailable | 5 consecutive failures | 3+ recent pings fail | ✅ Yes |

### 1.3 Key Difference: Two-Level Monitoring

libindigo provides **explicit two-level monitoring** that Mission Control currently implements implicitly:

1. **Host-level** — `HeartbeatChecker`: ICMP ping with TCP connect fallback (every 2s by default)
2. **Server-level** — `ServerChecker`: TCP handshake to INDIGO port, verifies protocol greeting (every ~10s by default)

The `StatusTracker` combines both into a single `AvailabilityStatus`. Only high-level `ClientEvent` status changes are emitted — individual ping/handshake results are logged but not exposed as events.

### 1.4 What Mission Control Loses

Individual `PingResult::Success(Duration)` and `PingResult::Failed` events are **not** emitted by libindigo. If Mission Control uses individual ping results for anything (e.g., latency display), that data is only available through `tracing` logs, not events. Verify whether Mission Control needs per-ping data before proceeding.

---

## 2. libindigo Monitoring API Reference

This section provides the complete API surface so the Mission Control instance does not need to read libindigo sources.

### 2.1 Feature Flag

Enable in `Cargo.toml`:

```toml
[dependencies]
libindigo = { version = "0.3.3", features = ["monitoring"] }
```

This activates:

- `libindigo/monitoring` → `libindigo-rs/monitoring` → pulls in `surge-ping` and `socket2`

### 2.2 Core Types

#### `MonitoringConfig`

```rust
use std::net::SocketAddr;
use std::time::Duration;

pub struct MonitoringConfig {
    pub server_addr: SocketAddr,
    pub ping_interval: Duration,          // Default: 2s
    pub response_time_threshold: Duration, // Default: 1s
    pub window_size: usize,                // Default: 5
    pub use_icmp: bool,                    // Default: true, auto-disabled for localhost
    pub connection_timeout: Duration,      // Default: 3s
}

impl MonitoringConfig {
    // Create with defaults for the given address
    // Auto-disables ICMP for localhost (127.0.0.1, ::1)
    pub fn new(server_addr: SocketAddr) -> Self;

    // Builder methods
    pub fn with_ping_interval(self, interval: Duration) -> Self;
    pub fn with_response_time_threshold(self, threshold: Duration) -> Self;
    pub fn with_window_size(self, size: usize) -> Self;
    pub fn with_connection_timeout(self, timeout: Duration) -> Self;
    pub fn with_icmp(self, enabled: bool) -> Self;
}
```

#### `AvailabilityStatus`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvailabilityStatus {
    Available,
    Degraded,
    Unavailable,
}

impl Default for AvailabilityStatus {
    fn default() -> Self { AvailabilityStatus::Unavailable }
}

impl std::fmt::Display for AvailabilityStatus { /* ... */ }
```

#### `ClientEvent`

```rust
#[derive(Debug, Clone)]
pub enum ClientEvent {
    ServerAvailable,
    ServerDegraded,
    ServerUnavailable,
}
```

### 2.3 Client Integration

#### Building with Monitoring

```rust
use libindigo::client::{ClientBuilder, MonitoringConfig};

let server_addr: SocketAddr = "192.168.1.50:7624".parse().unwrap();

let client = ClientBuilder::new()
    .with_strategy(strategy)
    .with_monitoring(MonitoringConfig::new(server_addr))
    .build()?;
```

#### Subscribing to Events

```rust
use libindigo::client::ClientEvent;

// Returns Option<mpsc::UnboundedReceiver<ClientEvent>>
// Returns None if monitoring not enabled or not supported
if let Some(mut rx) = client.subscribe_status() {
    while let Some(event) = rx.recv().await {
        match event {
            ClientEvent::ServerAvailable => { /* ... */ }
            ClientEvent::ServerDegraded => { /* ... */ }
            ClientEvent::ServerUnavailable => { /* ... */ }
        }
    }
}
```

#### Lifecycle

- Monitoring **starts automatically** when `strategy.connect()` is called
- Monitoring **stops automatically** when `strategy.disconnect()` is called
- The `subscribe_status()` method can be called **before** connect — subscribers will receive events once monitoring starts

### 2.4 Status Determination Logic

The `StatusTracker` rolling window uses this logic:

```
if recent 3 pings ALL fail:
    → Unavailable

else if recent 3 pings ALL succeed:
    if no handshake data yet:
        → Degraded (host up, server unknown)
    else if recent 3 handshakes ALL succeed:
        if any recent response time > threshold:
            → Degraded (slow)
        else:
            → Available
    else:
        → Degraded (server not responding)

else:
    → maintain previous status (insufficient data)
```

### 2.5 Monitoring Architecture

```
┌──────────────────────────────────────────────────┐
│                  ServerMonitor                     │
│                                                   │
│  HeartbeatChecker          ServerChecker           │
│  • ICMP ping               • TCP handshake        │
│  • TCP fallback            • Protocol check       │
│  • Every 2s                • Every ~10s            │
│         │                         │               │
│         └────────┬────────────────┘               │
│                  ▼                                 │
│           StatusTracker                            │
│           • Rolling window                         │
│           • State machine                          │
│                  │                                 │
│                  ▼                                 │
│       MonitoringEvent::StatusChanged               │
└──────────────────┬────────────────────────────────┘
                   │
                   ▼ (filtered to status changes only)
            ClientEvent::ServerAvailable
            ClientEvent::ServerDegraded
            ClientEvent::ServerUnavailable
                   │
                   ▼
        mpsc::UnboundedReceiver<ClientEvent>
```

---

## 3. Code to Remove from Mission Control

### 3.1 The `run_heartbeat` Function

**File**: `cli/src/main.rs` (around line 2745)

Remove the entire `run_heartbeat` async function. This function currently:

- Creates an ICMP pinger via `surge-ping`
- Falls back to TCP connect
- Maintains a rolling window of 5 ping results
- Sends `AppEvent::HeartbeatAvailable`, `HeartbeatDegraded`, `HeartbeatUnavailable` via mpsc

### 3.2 Types to Remove

**In `cli/src/main.rs`** (or wherever these are defined):

- `PingResult` enum (`Success(Duration)`, `Failed`) — replaced by libindigo's internal `PingResult` struct
- `AvailabilityStatus` enum (`Available`, `Degraded`, `Unavailable`) — replaced by `libindigo::client::AvailabilityStatus`

### 3.3 Dependencies to Remove

**In `cli/Cargo.toml`**:

```toml
# REMOVE these (now transitive through libindigo)
surge-ping = "0.8"
```

> **Note**: Only remove `surge-ping` if it is not used elsewhere in Mission Control. If it's only used in `run_heartbeat`, it's safe to remove.

### 3.4 Code Calling `run_heartbeat`

Find where `run_heartbeat` is spawned (likely in the connection setup or event loop) and remove:

- The `tokio::spawn` call for `run_heartbeat`
- Any task handle stored for the heartbeat task
- Any cancellation/abort logic for the heartbeat task

---

## 4. Code to Add/Modify in Mission Control

### 4.1 Add libindigo Dependency with Monitoring Feature

**File**: `cli/Cargo.toml`

```toml
[dependencies]
# Add or update libindigo with monitoring feature
libindigo = { version = "0.3.3", features = ["monitoring"] }
```

If Mission Control already depends on `libindigo`, add `"monitoring"` to the existing features list.

### 4.2 Configure MonitoringConfig During Connection Setup

**File**: Where the INDIGO client connection is established

```rust
use libindigo::client::{ClientBuilder, MonitoringConfig};
use std::net::SocketAddr;
use std::time::Duration;

// Parse the server address from Mission Control's connection config
let server_addr: SocketAddr = format!("{}:{}", host, port).parse()
    .expect("Invalid server address");

// Create monitoring config matching Mission Control's current behavior
let monitoring_config = MonitoringConfig::new(server_addr)
    .with_ping_interval(Duration::from_secs(2))       // Match current MC interval
    .with_response_time_threshold(Duration::from_secs(1)) // Match current 1s threshold
    .with_window_size(5);                               // Match current window of 5

// Build client with monitoring
let client = ClientBuilder::new()
    .with_strategy(strategy)
    .with_monitoring(monitoring_config)
    .build()?;
```

### 4.3 Subscribe and Bridge to AppEvent

**File**: Where the event loop is set up (near where `run_heartbeat` was spawned)

Create a bridge task that subscribes to `ClientEvent` and sends `AppEvent` variants:

```rust
use libindigo::client::ClientEvent;

// Subscribe to monitoring events BEFORE connecting
if let Some(mut status_rx) = client.subscribe_status() {
    let app_event_tx = app_event_tx.clone(); // Clone the existing AppEvent sender

    tokio::spawn(async move {
        while let Some(event) = status_rx.recv().await {
            let app_event = match event {
                ClientEvent::ServerAvailable => AppEvent::HeartbeatAvailable,
                ClientEvent::ServerDegraded => AppEvent::HeartbeatDegraded,
                ClientEvent::ServerUnavailable => AppEvent::HeartbeatUnavailable,
            };

            if app_event_tx.send(app_event).is_err() {
                // App event channel closed, stop listening
                break;
            }
        }
    });
}

// Then connect (monitoring starts automatically on connect)
client.strategy_mut().connect(&server_url).await?;
```

### 4.4 Update the Event Loop

**No changes needed** to the event loop handler itself. The existing `AppEvent::HeartbeatAvailable`, `HeartbeatDegraded`, `HeartbeatUnavailable` match arms stay exactly as they are. The bridge task in 4.3 translates `ClientEvent` → `AppEvent`.

### 4.5 Handle Disconnect/Reconnect

When Mission Control disconnects and reconnects:

```rust
// On disconnect — monitoring stops automatically
client.strategy_mut().disconnect().await?;

// On reconnect — re-subscribe before connecting
if let Some(mut status_rx) = client.subscribe_status() {
    let app_event_tx = app_event_tx.clone();
    tokio::spawn(async move {
        while let Some(event) = status_rx.recv().await {
            let app_event = match event {
                ClientEvent::ServerAvailable => AppEvent::HeartbeatAvailable,
                ClientEvent::ServerDegraded => AppEvent::HeartbeatDegraded,
                ClientEvent::ServerUnavailable => AppEvent::HeartbeatUnavailable,
            };
            if app_event_tx.send(app_event).is_err() {
                break;
            }
        }
    });
}

client.strategy_mut().connect(&server_url).await?;
```

### 4.6 Import Changes

Add to the relevant source file:

```rust
use libindigo::client::{ClientBuilder, ClientEvent, MonitoringConfig};
// Remove: use of local AvailabilityStatus, PingResult
```

---

## 5. What to Keep in Mission Control

### 5.1 Keep: AppEvent Variants

```rust
enum AppEvent {
    // KEEP these — they are Mission Control's internal event language
    HeartbeatAvailable,
    HeartbeatDegraded,
    HeartbeatUnavailable,
    // ... other variants ...
}
```

These serve as the internal event bus. The bridge task translates `ClientEvent` → `AppEvent`.

### 5.2 Keep: UI Update Logic

All code in the event loop that handles `AppEvent::HeartbeatAvailable`, etc. to update the UI remains unchanged:

```rust
// This stays exactly as-is
match event {
    AppEvent::HeartbeatAvailable => {
        // Update status bar, connection indicator, etc.
    }
    AppEvent::HeartbeatDegraded => {
        // Show warning indicator
    }
    AppEvent::HeartbeatUnavailable => {
        // Show disconnected state
    }
    // ...
}
```

### 5.3 Keep: Reconnection Logic

Any reconnection logic triggered by `HeartbeatUnavailable` or status changes remains in Mission Control. libindigo only reports status — it does not attempt reconnection.

---

## 6. Phased Migration Approach

### Phase 1: Add libindigo Monitoring Alongside Existing

**Goal**: Get libindigo monitoring running without removing existing code.

1. Add `libindigo = { features = ["monitoring"] }` to `cli/Cargo.toml`
2. In the connection setup, create a `MonitoringConfig` and pass it to `ClientBuilder::with_monitoring()`
3. Subscribe to `ClientEvent` and **log** events (don't send to `AppEvent` yet)
4. Verify libindigo monitoring produces the expected status transitions by testing against a real INDIGO server
5. Compare libindigo events with existing `run_heartbeat` events to confirm behavioral parity

```rust
// Phase 1: Logging bridge (temporary)
if let Some(mut status_rx) = client.subscribe_status() {
    tokio::spawn(async move {
        while let Some(event) = status_rx.recv().await {
            tracing::info!("[libindigo-monitor] {:?}", event);
        }
    });
}
```

### Phase 2: Wire libindigo Events to AppEvent

**Goal**: Replace the event source while keeping both systems running temporarily.

1. Create the bridge task that sends `ClientEvent` → `AppEvent` (as shown in section 4.3)
2. Both `run_heartbeat` and the libindigo bridge will be sending `AppEvent` simultaneously
3. Add a temporary flag or config option to choose which source to use
4. Test that the UI and reconnection behavior works correctly with the libindigo source

### Phase 3: Remove Old Monitoring Code

**Goal**: Clean removal of all custom monitoring code.

1. Remove the `run_heartbeat` function entirely
2. Remove the `PingResult` enum
3. Remove the `AvailabilityStatus` enum (use `libindigo::client::AvailabilityStatus` if needed anywhere, though the bridge means Mission Control mostly just sees `AppEvent`)
4. Remove `surge-ping` from `cli/Cargo.toml`
5. Remove the `tokio::spawn(run_heartbeat(...))` call
6. Remove any heartbeat task handle and cancellation code
7. Clean up unused imports

### Phase 4: Test and Verify

**Goal**: Confirm full behavioral parity.

1. **Local server test**: Connect to `localhost:7624`, verify `Available` → `Unavailable` transition when server stops
2. **Remote server test**: Connect to a remote INDIGO server, verify status transitions
3. **Reconnection test**: Verify reconnection logic still triggers correctly on `HeartbeatUnavailable`
4. **Degraded test**: Verify `Degraded` state appears when server process is starting up
5. **UI test**: Verify status indicators update correctly for all three states
6. **ICMP privilege test**: Verify monitoring works on systems without ICMP privileges (TCP fallback)
7. **Localhost test**: Verify ICMP is auto-disabled for `127.0.0.1` connections

---

## 7. Two-Level Monitoring Verification

### 7.1 How libindigo Maps to Mission Control's Needs

| Mission Control Need | libindigo Component | Details |
|---|---|---|
| Host reachable? | `HeartbeatChecker` | ICMP ping with TCP fallback, every `ping_interval` |
| Server running? | `ServerChecker` | TCP handshake + INDIGO protocol greeting check, every ~5 pings |
| Should attempt reconnect? | `ClientEvent::ServerUnavailable` | Host unreachable → trigger reconnection |
| Is server starting up? | `ClientEvent::ServerDegraded` | Host reachable but server not responding properly |
| Is everything working? | `ClientEvent::ServerAvailable` | Host reachable + server responding + fast responses |

### 7.2 Behavioral Differences to Be Aware Of

| Aspect | Mission Control Current | libindigo | Impact |
|---|---|---|---|
| Ping interval | 2s (assumed) | 2s (configurable) | None — match with config |
| Window size | 5 | 5 (configurable) | None — match with config |
| Threshold | 5 consecutive | 3 recent results | **Faster transitions** — libindigo transitions faster since it uses the most recent 3 (not 5 consecutive). Adjustable via `window_size` |
| Response threshold | 1s | 1s (configurable) | None — match with config |
| Server check | TCP handshake | TCP handshake + protocol verification | **Slightly stricter** — libindigo also verifies the INDIGO protocol greeting, not just TCP connectivity |
| ICMP fallback | Yes | Yes | Same — falls back to TCP if ICMP unavailable |
| Event granularity | Individual ping results + status | Status changes only | **Less granular** — see section 1.4 |

### 7.3 Configuration to Match Current Behavior

To get the closest behavioral match with the current Mission Control monitoring:

```rust
let monitoring_config = MonitoringConfig::new(server_addr)
    .with_ping_interval(Duration::from_secs(2))
    .with_response_time_threshold(Duration::from_secs(1))
    .with_window_size(5)
    .with_connection_timeout(Duration::from_secs(3));
```

---

## 8. Logging Integration

libindigo uses `tracing` (not `log`). If Mission Control uses `log`, add the bridge:

```toml
[dependencies]
tracing-log = "0.2"  # Bridge tracing → log (or vice versa)
```

Or, if Mission Control already uses `tracing`, configure the monitoring log level:

```bash
# Production: only status changes
RUST_LOG=libindigo_rs::monitoring=info

# Debugging: individual check failures
RUST_LOG=libindigo_rs::monitoring=debug

# Deep debugging: every ping/handshake
RUST_LOG=libindigo_rs::monitoring=trace
```

---

## 9. Checklist Summary

### Files to Modify

- [ ] `cli/Cargo.toml` — Add `libindigo` with `monitoring` feature, remove `surge-ping`
- [ ] `cli/src/main.rs` — Remove `run_heartbeat`, `PingResult`, `AvailabilityStatus`
- [ ] `cli/src/main.rs` — Add `MonitoringConfig` setup in connection code
- [ ] `cli/src/main.rs` — Add `ClientEvent` → `AppEvent` bridge task
- [ ] `cli/src/main.rs` — Remove `tokio::spawn(run_heartbeat(...))` call
- [ ] `cli/src/main.rs` — Update reconnect logic to re-subscribe on reconnect
- [ ] `cli/src/main.rs` — Clean up unused imports

### Files NOT to Modify

- `AppEvent` enum — Keep `HeartbeatAvailable`, `HeartbeatDegraded`, `HeartbeatUnavailable` variants
- Event loop handlers — Keep all UI/reconnection logic triggered by heartbeat events
- Any other Mission Control business logic

### Dependencies

| Action | Crate | Version | Notes |
|---|---|---|---|
| Add/Update | `libindigo` | `0.3.3+` | Add `monitoring` feature |
| Remove | `surge-ping` | — | Now transitive through libindigo |

---

## 10. Rollback Plan

If issues are found after Phase 3:

1. Revert the `cli/Cargo.toml` changes (re-add `surge-ping`)
2. Revert `cli/src/main.rs` to restore `run_heartbeat` and related types
3. Remove the `ClientEvent` bridge task
4. The `with_monitoring()` call in `ClientBuilder` is harmless — can be left in place or removed

The phased approach (especially Phase 2 running both simultaneously) provides a safety net before the irreversible removal in Phase 3.

---

**Last Updated**: 2026-03-12
