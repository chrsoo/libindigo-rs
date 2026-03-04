# ZeroConf/Bonjour Server Discovery API Architecture

**Document Version**: 1.0
**Date**: 2026-03-04
**Status**: Proposed
**Author**: Architecture Planning

---

## Executive Summary

This document defines the architecture for a ZeroConf/Bonjour server discovery API for the libindigo client library. The API enables automatic discovery of INDIGO servers on the local network using mDNS/DNS-SD, supporting both RS (pure Rust) and FFI strategies.

**Key Design Goals:**

1. ✅ **Ergonomic API** - Simple, intuitive discovery interface
2. ✅ **Strategy-agnostic** - Works with both RS and FFI implementations
3. ✅ **Cross-platform** - macOS (Bonjour), Linux (Avahi), Windows (Bonjour)
4. ✅ **Async-first** - Non-blocking discovery using tokio
5. ✅ **Optional** - Behind feature flag, graceful degradation
6. ✅ **Type-safe** - Leverages Rust's type system

---

## Table of Contents

1. [Background Research](#background-research)
2. [API Design](#api-design)
3. [Discovery Flow](#discovery-flow)
4. [Integration with Client Builder](#integration-with-client-builder)
5. [Error Handling](#error-handling)
6. [Cross-Platform Considerations](#cross-platform-considerations)
7. [Implementation Plan](#implementation-plan)
8. [Example Usage](#example-usage)
9. [Testing Strategy](#testing-strategy)
10. [Future Enhancements](#future-enhancements)

---

## Background Research

### INDIGO Server mDNS Advertisement

From INDIGO documentation:

- **Service Type**: `_indigo._tcp.local.`
- **Protocol**: mDNS/DNS-SD (Bonjour on macOS, Avahi on Linux)
- **TXT Records**: May contain version, capabilities metadata
- **Discovery Events**:
  - `INDIGO_SERVICE_ADDED` - Service discovered
  - `INDIGO_SERVICE_REMOVED` - Service disappeared
  - `INDIGO_SERVICE_END_OF_RECORD` - Initial discovery complete

### Existing Implementation

Current [`src/auto.rs`](../src/auto.rs) has:

- Stub implementation with `unimplemented!()`
- Basic `zeroconf` crate imports
- Legacy `Controller<B>` trait usage (old architecture)
- **Status**: Incomplete, needs redesign

### Crate Evaluation

#### Option 1: `zeroconf` (v0.15) - **RECOMMENDED**

**Pros:**

- ✅ Already in [`Cargo.toml`](../Cargo.toml) (optional dependency)
- ✅ Cross-platform (macOS, Linux, Windows)
- ✅ Active maintenance
- ✅ Simple callback-based API
- ✅ Supports both browsing and resolving

**Cons:**

- ⚠️ Callback-based (not native async)
- ⚠️ Requires platform-specific dependencies (Avahi on Linux)

**Platform Dependencies:**

- **Linux**: `libavahi-compat-libdnssd-dev`
- **macOS**: Built-in Bonjour
- **Windows**: Bonjour SDK

#### Option 2: `mdns-sd` (v0.11)

**Pros:**

- ✅ Pure Rust implementation
- ✅ No platform dependencies
- ✅ Cross-platform

**Cons:**

- ⚠️ Less mature than `zeroconf`
- ⚠️ May have compatibility issues with some networks
- ⚠️ Different API surface

**Decision**: Use `zeroconf` (Option 1) for production quality and cross-platform support.

---

## API Design

### Core Types

```rust
/// Discovered INDIGO server information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredServer {
    /// Service name (e.g., "INDIGO Server @ hostname")
    pub name: String,

    /// Hostname or IP address
    pub host: String,

    /// TCP port number
    pub port: u16,

    /// Network interface index where service was discovered
    pub interface_index: u32,

    /// Optional TXT record metadata
    pub metadata: HashMap<String, String>,

    /// Timestamp when discovered
    pub discovered_at: std::time::Instant,
}

impl DiscoveredServer {
    /// Returns the connection URL for this server.
    pub fn url(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Returns the full service identifier.
    pub fn service_id(&self) -> String {
        format!("{}@{}", self.name, self.host)
    }
}

/// Discovery mode configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryMode {
    /// One-shot discovery: collect servers for a duration, then stop.
    OneShot,

    /// Continuous discovery: keep monitoring for server changes.
    Continuous,
}

/// Discovery configuration builder.
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    timeout: Duration,
    mode: DiscoveryMode,
    interface: Option<u32>,
    name_filter: Option<String>,
}

impl DiscoveryConfig {
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            mode: DiscoveryMode::OneShot,
            interface: None,
            name_filter: None,
        }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn mode(mut self, mode: DiscoveryMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn interface(mut self, interface: u32) -> Self {
        self.interface = Some(interface);
        self
    }

    pub fn name_filter(mut self, pattern: impl Into<String>) -> Self {
        self.name_filter = Some(pattern.into());
        self
    }
}
```

### Discovery API

```rust
/// Server discovery handle for continuous monitoring.
pub struct ServerDiscovery {
    rx: mpsc::UnboundedReceiver<DiscoveryEvent>,
    task: JoinHandle<()>,
    servers: Arc<Mutex<HashMap<String, DiscoveredServer>>>,
}

/// Discovery events for continuous monitoring.
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    ServerAdded(DiscoveredServer),
    ServerRemoved(String), // service_id
    DiscoveryComplete,
}

impl ServerDiscovery {
    pub async fn next_event(&mut self) -> Option<DiscoveryEvent> {
        self.rx.recv().await
    }

    pub async fn servers(&self) -> Vec<DiscoveredServer> {
        let servers = self.servers.lock().await;
        servers.values().cloned().collect()
    }

    pub async fn stop(self) {
        self.task.abort();
        let _ = self.task.await;
    }
}

/// Main discovery API.
pub struct ServerDiscoveryApi;

impl ServerDiscoveryApi {
    /// Discovers INDIGO servers on the local network (one-shot).
    pub async fn discover(config: DiscoveryConfig) -> Result<Vec<DiscoveredServer>> {
        Self::discover_impl(config).await
    }

    /// Starts continuous server discovery.
    pub async fn start_continuous(config: DiscoveryConfig) -> Result<ServerDiscovery> {
        Self::start_continuous_impl(config).await
    }

    #[cfg(feature = "auto")]
    async fn discover_impl(config: DiscoveryConfig) -> Result<Vec<DiscoveredServer>>;

    #[cfg(not(feature = "auto"))]
    async fn discover_impl(_config: DiscoveryConfig) -> Result<Vec<DiscoveredServer>> {
        Err(IndigoError::NotSupported(
            "Server discovery requires the 'auto' feature".to_string()
        ))
    }
}
```

---

## Discovery Flow

### One-Shot Discovery Sequence

```
Application → ServerDiscoveryApi::discover(config)
    ↓
Create MdnsBrowser for "_indigo._tcp"
    ↓
Set discovery callback
    ↓
Start browsing (spawn_blocking)
    ↓
[For timeout duration]
    Network → Service discovered
    Resolve service (host, port)
    Apply filters
    Add to results
    ↓
Stop browsing
    ↓
Return Vec<DiscoveredServer>
```

### Continuous Discovery Sequence

```
Application → ServerDiscoveryApi::start_continuous(config)
    ↓
Create channels (tx, rx)
    ↓
Spawn background task
    ↓
Create MdnsBrowser
    ↓
Set callbacks (added, removed)
    ↓
Return ServerDiscovery handle
    ↓
[Continuous loop]
    Service added → Send ServerAdded event
    Service removed → Send ServerRemoved event
    ↓
Application → discovery.next_event()
    ↓
Application → discovery.stop()
```

---

## Integration with Client Builder

### Extended Client API

```rust
impl Client {
    /// Discovers INDIGO servers on the local network.
    #[cfg(feature = "auto")]
    pub async fn discover_servers() -> Result<Vec<DiscoveredServer>> {
        ServerDiscoveryApi::discover(DiscoveryConfig::default()).await
    }

    /// Discovers INDIGO servers with custom configuration.
    #[cfg(feature = "auto")]
    pub async fn discover_servers_with_config(
        config: DiscoveryConfig
    ) -> Result<Vec<DiscoveredServer>> {
        ServerDiscoveryApi::discover(config).await
    }
}

impl ClientBuilder {
    /// Discovers and connects to the first available INDIGO server.
    #[cfg(feature = "auto")]
    pub async fn discover_and_connect(self) -> Result<Client> {
        let servers = Client::discover_servers().await?;
        let server = servers.first()
            .ok_or_else(|| IndigoError::ConnectionError(
                "No INDIGO servers found".to_string()
            ))?;

        let mut client = self.build()?;
        client.connect(&server.url()).await?;
        Ok(client)
    }

    /// Discovers and connects to a server matching the given name pattern.
    #[cfg(feature = "auto")]
    pub async fn discover_and_connect_to(self, name_pattern: &str) -> Result<Client> {
        let config = DiscoveryConfig::new()
            .name_filter(name_pattern);

        let servers = ServerDiscoveryApi::discover(config).await?;
        let server = servers.first()
            .ok_or_else(|| IndigoError::ConnectionError(
                format!("No INDIGO server matching '{}' found", name_pattern)
            ))?;

        let mut client = self.build()?;
        client.connect(&server.url()).await?;
        Ok(client)
    }
}
```

---

## Error Handling

### Error Types

Discovery operations use existing [`IndigoError`](../src/error.rs) variants:

```rust
pub enum IndigoError {
    NotSupported(String),      // Feature not enabled
    ConnectionError(String),   // mDNS initialization failed
    Timeout(String),          // No servers found
    // ... existing variants
}
```

### Error Scenarios

| Scenario | Error Type | Message |
|----------|------------|---------|
| `auto` feature not enabled | `NotSupported` | "Server discovery requires the 'auto' feature..." |
| mDNS initialization fails | `ConnectionError` | "Failed to initialize mDNS browser: ..." |
| No servers found | `Timeout` | "No INDIGO servers discovered within timeout" |
| Platform dependencies missing | `ConnectionError` | "mDNS not available (install Avahi on Linux)" |

### Graceful Degradation

```rust
// Fallback to manual connection
let client = match Client::discover_servers().await {
    Ok(servers) if !servers.is_empty() => {
        let mut client = ClientBuilder::new()
            .with_rs_strategy()
            .build()?;
        client.connect(&servers[0].url()).await?;
        client
    }
    _ => {
        let mut client = ClientBuilder::new()
            .with_rs_strategy()
            .build()?;
        client.connect("localhost:7624").await?;
        client
    }
};
```

---

## Cross-Platform Considerations

### Platform Support Matrix

| Platform | Backend | Dependencies | Status |
|----------|---------|--------------|--------|
| **macOS** | Bonjour (built-in) | None | ✅ Fully supported |
| **Linux** | Avahi | `libavahi-compat-libdnssd-dev` | ✅ Fully supported |
| **Windows** | Bonjour SDK | Bonjour installer | ✅ Supported |
| **BSD** | Avahi | `avahi` package | ⚠️ Untested |

### Platform-Specific Notes

#### macOS

- Bonjour is built into the OS
- No additional dependencies required
- Works out of the box

#### Linux

- Requires Avahi daemon running
- Install: `sudo apt-get install libavahi-compat-libdnssd-dev`
- May require firewall configuration for mDNS (port 5353 UDP)

#### Windows

- Requires Bonjour for Windows
- May require administrator privileges
- Firewall may need to allow mDNS traffic

---

## Implementation Plan

### Phase 1: Core Discovery API (Week 1)

**Goal**: Implement basic one-shot discovery

**Tasks**:

1. Create `src/discovery/mod.rs` module
2. Define core types: `DiscoveredServer`, `DiscoveryConfig`, `DiscoveryMode`
3. Implement `ServerDiscoveryApi::discover()` for one-shot discovery
4. Add feature gate: `#[cfg(feature = "auto")]`
5. Implement graceful error when feature disabled
6. Add unit tests for types and configuration

**Files**:

- `src/discovery/mod.rs` (new)
- `src/discovery/types.rs` (new)
- `src/discovery/api.rs` (new)
- `src/lib.rs` (update exports)

**Acceptance Criteria**:

- [ ] One-shot discovery works on macOS
- [ ] Returns list of discovered servers
- [ ] Respects timeout configuration
- [ ] Graceful error when `auto` feature disabled

### Phase 2: Continuous Discovery (Week 1)

**Goal**: Add continuous monitoring support

**Tasks**:

1. Implement `ServerDiscovery` handle type
2. Implement `DiscoveryEvent` enum
3. Implement `ServerDiscoveryApi::start_continuous()`
4. Add background task management
5. Implement service removal detection
6. Add tests for continuous discovery

**Files**:

- `src/discovery/continuous.rs` (new)
- `src/discovery/api.rs` (update)

**Acceptance Criteria**:

- [ ] Continuous discovery detects new servers
- [ ] Detects when servers go offline
- [ ] Can be stopped gracefully
- [ ] No resource leaks

### Phase 3: Client Builder Integration (Week 2)

**Goal**: Integrate discovery with Client API

**Tasks**:

1. Add `Client::discover_servers()` convenience method
2. Add `ClientBuilder::discover_and_connect()`
3. Add `ClientBuilder::discover_and_connect_to(name)`
4. Update documentation with examples
5. Add integration tests

**Files**:

- `src/client/builder.rs` (update)
- `src/client/mod.rs` (update)
- `tests/discovery_integration.rs` (new)

**Acceptance Criteria**:

- [ ] Can discover and connect in one call
- [ ] Can filter by server name
- [ ] Works with both RS and FFI strategies
- [ ] Clear error messages

### Phase 4: Cross-Platform Testing (Week 2)

**Goal**: Verify on all platforms

**Tasks**:

1. Test on macOS (Bonjour)
2. Test on Linux (Avahi)
3. Test on Windows (Bonjour SDK)
4. Update CI/CD workflows
5. Document platform-specific requirements
6. Add troubleshooting guide

**Files**:

- `.github/workflows/test.yml` (update)
- `docs/discovery.md` (new)
- `README.md` (update)

**Acceptance Criteria**:

- [ ] Works on macOS, Linux, Windows
- [ ] CI tests pass on all platforms
- [ ] Clear documentation for setup
- [ ] Troubleshooting guide available

### Phase 5: Documentation & Examples (Week 3)

**Goal**: Complete documentation and examples

**Tasks**:

1. Write API documentation
2. Create example: `examples/discover_servers.rs`
3. Create example: `examples/continuous_discovery.rs`
4. Create example: `examples/auto_connect.rs`
5. Update README with discovery section

**Files**:

- `examples/discover_servers.rs` (new)
- `examples/continuous_discovery.rs` (new)
- `examples/auto_connect.rs` (new)
- `README.md` (update)

**Acceptance Criteria**:

- [ ] All examples compile and run
- [ ] API documentation complete
- [ ] README has discovery section
- [ ] Examples demonstrate all features

---

## Example Usage

### Example 1: Simple Discovery

```rust
use libindigo::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let servers = Client::discover_servers().await?;

    println!("Found {} INDIGO servers:", servers.len());
    for server in &servers {
        println!("  - {} at {}", server.name, server.url());
    }

    if let Some(server) = servers.first() {
        let mut client = ClientBuilder::new()
            .with_rs_strategy()
            .build()?;

        client.connect(&server.url()).await?;
        client.enumerate_properties(None).await?;
        client.disconnect().await?;
    }

    Ok(())
}
```

### Example 2: Custom Configuration

```rust
use libindigo::discovery::{DiscoveryConfig, ServerDiscoveryApi};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let config = DiscoveryConfig::new()
        .timeout(Duration::from_secs(10))
        .name_filter("Observatory");

    let servers = ServerDiscoveryApi::discover(config).await?;

    for server in servers {
        println!("Found: {} at {}", server.name, server.url());
    }

    Ok(())
}
```

### Example 3: Continuous Discovery

```rust
use libindigo::discovery::{DiscoveryConfig, DiscoveryMode, DiscoveryEvent};

#[tokio::main]
async fn main() -> Result<()> {
    let config = DiscoveryConfig::new()
        .mode(DiscoveryMode::Continuous);

    let mut discovery = ServerDiscoveryApi::start_continuous(config).await?;

    while let Some(event) = discovery.next_event().await {
        match event {
            DiscoveryEvent::ServerAdded(server) => {
                println!("✓ Server added: {}", server.name);
            }
            DiscoveryEvent::ServerRemoved(id) => {
                println!("✗ Server removed: {}", id);
            }
            DiscoveryEvent::DiscoveryComplete => {
                println!("Initial discovery complete");
            }
        }
    }

    discovery.stop().await;
    Ok(())
}
```

### Example 4: Auto-Connect

```rust
use libindigo::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = ClientBuilder::new()
        .with_rs_strategy()
        .discover_and_connect()
        .await?;

    println!("Auto-connected!");
    client.enumerate_properties(None).await?;
    client.disconnect().await?;
    Ok(())
}
```

### Example 5: Fallback Pattern

```rust
use libindigo::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = ClientBuilder::new()
        .with_rs_strategy()
        .build()?;

    match Client::discover_servers().await {
        Ok(servers) if !servers.is_empty() => {
            println!("Discovered {} servers", servers.len());
            client.connect(&servers[0].url()).await?;
        }
        _ => {
            println!("No servers discovered, using localhost");
            client.connect("localhost:7624").await?;
        }
    }

    client.enumerate_properties(None).await?;
    client.disconnect().await?;
    Ok(())
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovered_server_url() {
        let server = DiscoveredServer {
            name: "Test".to_string(),
            host: "192.168.1.100".to_string(),
            port: 7624,
            interface_index: 0,
            metadata: HashMap::new(),
            discovered_at: std::time::Instant::now(),
        };

        assert_eq!(server.url(), "192.168.1.100:7624");
    }

    #[test]
    #[cfg(not(feature = "auto"))]
    fn test_discovery_without_feature() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            ServerDiscoveryApi::discover(DiscoveryConfig::default()).await
        });

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IndigoError::NotSupported(_)));
    }
}
```

### Integration Tests

```rust
#[cfg(feature = "auto")]
mod discovery_tests {
    use libindigo::prelude::*;

    #[tokio::test]
    async fn test_discover_servers() {
        let result = ServerDiscoveryApi::discover(
            DiscoveryConfig::default()
        ).await;

        assert!(result.is_ok());
    }
}
```

---

## Future Enhancements

1. **Server Capabilities Detection** - Parse TXT records for version/capabilities
2. **Smart Server Selection** - Prefer local servers, load balancing
3. **Persistent Server Cache** - Remember previously discovered servers
4. **IPv6 Support** - Discover servers on IPv6 networks
5. **Service Announcement** - Allow Rust clients to announce themselves

---

## Summary

This architecture provides a comprehensive ZeroConf/Bonjour server discovery API that:

✅ **Works with both strategies** - RS and FFI implementations
✅ **Cross-platform** - macOS, Linux, Windows support
✅ **Async-first** - Non-blocking discovery using tokio
✅ **Optional** - Behind `auto` feature flag
✅ **Ergonomic** - Simple, intuitive API
✅ **Production-ready** - Error handling and graceful degradation

---

**Next Steps:**

1. Review and approve this architecture
2. Begin Phase 1 implementation
3. Test on all target platforms
4. Iterate based on feedback

**Questions for Discussion:**

1. Should we support `mdns-sd` as an alternative pure-Rust backend?
2. Should continuous discovery be in Phase 1 or Phase 2?
3. Timeline: Is 3 weeks realistic for full implementation?

---

**Document Status**: ✅ Complete and ready for review
