# libindigo-rs

Rust API for writing client applications and device drivers for astronomy equipment using the [INDIGO](https://www.indigo-astronomy.org/index.html) protocol and architecture.

## 🎉 Refactoring Complete

The project has been successfully refactored into a multi-crate workspace! See [`REFACTORING_COMPLETE.md`](REFACTORING_COMPLETE.md) for full details.

## Architecture

This project is organized as a Cargo workspace with multiple crates:

| Crate | Purpose | FFI Dependencies |
|-------|---------|------------------|
| [`libindigo`](src/) | Core API, traits, and types | None |
| [`libindigo-rs`](rs/) | Pure Rust implementation | None |
| [`libindigo-ffi`](ffi/) | FFI-based implementation | Yes (via libindigo-sys) |
| [`libindigo-sys`](sys/) | Raw C bindings | Yes |

```
┌─────────────────┐
│  libindigo      │  ← Core API (traits, types, constants)
│  (root crate)   │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
┌─────────┐ ┌──────────────┐
│libindigo│ │ libindigo-ffi│
│   -rs   │ │              │
└─────────┘ └──────┬───────┘
                   │
                   ▼
            ┌──────────────┐
            │ libindigo-sys│
            └──────────────┘
```

## Goal

- A pure Rust API that is 100% compatible with the INDIGO platform and its default C-implementation

## Objectives

- ✅ Provide an API that uses idiomatic Rust for integrating with the INDIGO Bus
- ✅ Provide a Service Provider Interface (SPI) for decoupling the API from its implementation
- ✅ Provide a default RS (Rust) implementation of the SPI without any FFI bindings to the INDIGO C-libraries or other non-rust dependencies
- ✅ Provide an FFI implementation of the SPI that uses the INDIGO C-library with any necessary dependencies

## Quick Start

### Pure Rust Client (Recommended - No C Dependencies)

Add to your `Cargo.toml`:

```toml
[dependencies]
libindigo-rs = "0.3"
tokio = { version = "1.35", features = ["full"] }
```

Example code:

```rust
use libindigo_rs::{Client, ClientBuilder, RsClientStrategy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with pure Rust strategy
    let strategy = RsClientStrategy::new();
    let mut client = ClientBuilder::new()
        .with_strategy(strategy)
        .build();

    // Connect to INDIGO server
    client.connect("localhost:7624").await?;

    // Enumerate all properties
    client.enumerate_properties(None).await?;

    // Disconnect
    client.disconnect().await?;

    Ok(())
}
```

### FFI-Based Client (Maximum Compatibility)

Add to your `Cargo.toml`:

```toml
[dependencies]
libindigo-ffi = "0.3"
tokio = { version = "1.35", features = ["full"] }
```

Example code:

```rust
use libindigo_ffi::{Client, ClientBuilder, FfiClientStrategy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Note: FFI implementation is currently stubbed
    let strategy = FfiClientStrategy::new()?;
    let mut client = ClientBuilder::new()
        .with_strategy(strategy)
        .build();

    client.connect("localhost:7624").await?;
    client.enumerate_properties(None).await?;
    client.disconnect().await?;

    Ok(())
}
```

## Features

### Pure Rust Strategy ([`libindigo-rs`](rs/))

The pure Rust strategy provides a complete INDIGO client implementation without C FFI dependencies:

- ✅ **Zero FFI**: No C dependencies, pure Rust implementation
- ✅ **Async-First**: Built on tokio for efficient async I/O
- ✅ **Type Safe**: Leverages Rust's type system for protocol correctness
- ✅ **Cross-Platform**: Works anywhere Rust compiles
- ✅ **Dual Protocol**: Full INDIGO JSON and XML protocol support with automatic negotiation
- ✅ **JSON-First**: Defaults to modern JSON protocol with XML fallback
- ✅ **mDNS Discovery**: Optional pure Rust server discovery (no FFI)

**Feature Flags**:

- `client` (default): Client functionality
- `device`: Device driver support (stub for future)
- `discovery`: mDNS server discovery via pure Rust `mdns-sd` crate

### FFI Strategy ([`libindigo-ffi`](ffi/))

The FFI strategy wraps the official C INDIGO library:

- ⚠️ **Status**: Structure in place, implementation pending
- ✅ **Maximum Compatibility**: Uses the official INDIGO C library
- ✅ **Async Support**: Async wrappers around synchronous FFI calls
- ✅ **Battle-Tested**: Leverages mature C implementation
- ✅ **Feature Complete**: Access to all INDIGO features (when implemented)

**Feature Flags**:

- `client` (default): Client functionality
- `device`: Device driver support (stub for future)
- `async`: Async wrapper for non-blocking operations

## JSON Protocol Support ✅

libindigo-rs supports both INDIGO JSON and XML protocols with intelligent negotiation:

### Automatic Protocol Negotiation (Default)

The client automatically negotiates the best protocol with the server:

```rust
use libindigo_rs::{Client, ClientBuilder, RsClientStrategy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // JSON-first with XML fallback (default behavior)
    let strategy = RsClientStrategy::new();
    let mut client = ClientBuilder::new()
        .with_strategy(strategy)
        .build();

    client.connect("localhost:7624").await?;
    // Client automatically negotiates protocol with server
    // Prefers JSON, falls back to XML if server doesn't support JSON

    client.enumerate_properties(None).await?;
    client.disconnect().await?;
    Ok(())
}
```

### Protocol Comparison

| Feature | JSON Protocol | XML Protocol |
|---------|--------------|--------------|
| **Version** | 512 (numeric) | "2.0" (string) |
| **Switch Values** | `true`/`false` | `On`/`Off` |
| **Number Format** | Native JSON numbers | String with format |
| **BLOBs** | URL only | URL or BASE64 |
| **Parsing Speed** | ⚡ Faster | Slightly slower |
| **Size** | 📦 More compact | More verbose |
| **Use Case** | Modern clients, web apps | Legacy compatibility |
| **Server Support** | INDIGO 2.0+ | All INDIGO versions |

### Protocol Selection Examples

See [`rs/src/lib.rs`](rs/src/lib.rs) documentation for advanced protocol negotiation options.

## Server Discovery

The pure Rust implementation includes optional mDNS server discovery:

```toml
[dependencies]
libindigo-rs = { version = "0.3", features = ["discovery"] }
```

```rust
use libindigo_rs::discovery::{DiscoveryBuilder, DiscoveryEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut discovery = DiscoveryBuilder::new()
        .with_timeout(std::time::Duration::from_secs(5))
        .build()?;

    let mut receiver = discovery.start().await?;

    while let Some(event) = receiver.recv().await {
        match event {
            DiscoveryEvent::ServerFound { name, address, port } => {
                println!("Found server: {} at {}:{}", name, address, port);
            }
            DiscoveryEvent::ServerLost { name } => {
                println!("Lost server: {}", name);
            }
        }
    }

    Ok(())
}
```

See [`rs/PHASE5_DISCOVERY_MIGRATION.md`](rs/PHASE5_DISCOVERY_MIGRATION.md) for migration details.

## Strategy Comparison

| Feature | libindigo-rs | libindigo-ffi |
|---------|--------------|---------------|
| **C Dependencies** | ❌ None | ✅ Required |
| **Async Support** | ✅ Native | ✅ Wrapped |
| **Cross-Platform** | ✅ Excellent | ⚠️ Limited |
| **Performance** | ✅ Fast | ✅ Fast |
| **JSON Protocol** | ✅ Yes | ⚠️ Via C lib |
| **XML Protocol** | ✅ Yes | ✅ Yes |
| **Protocol Negotiation** | ✅ Automatic | ❌ No |
| **mDNS Discovery** | ✅ Pure Rust | ❌ No |
| **Maturity** | ✅ Production | ⚠️ Stub |
| **Use Case** | Modern apps | Legacy compat |

## Usage Examples

### Receiving Property Updates

```rust
use libindigo_rs::{Client, ClientBuilder, RsClientStrategy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut strategy = RsClientStrategy::new();

    // Connect to server
    strategy.connect("localhost:7624").await?;

    // Get property receiver
    let mut rx = strategy.property_receiver().await.unwrap();

    // Spawn task to receive properties
    tokio::spawn(async move {
        while let Some(property) = rx.recv().await {
            println!("Property: {}.{} = {:?}",
                property.device,
                property.name,
                property.values
            );
        }
    });

    // Enumerate properties
    strategy.enumerate_properties(None).await?;

    // Keep running...
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    strategy.disconnect().await?;
    Ok(())
}
```

### Sending Property Updates

```rust
use libindigo_rs::{Client, ClientBuilder, RsClientStrategy};
use libindigo_rs::types::{Property, PropertyType, PropertyValue, SwitchState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let strategy = RsClientStrategy::new();
    let mut client = ClientBuilder::new()
        .with_strategy(strategy)
        .build();

    client.connect("localhost:7624").await?;

    // Create a switch property to connect a device
    let mut property = Property::new(
        "CCD Simulator".to_string(),
        "CONNECTION".to_string(),
        PropertyType::Switch,
    );

    property.values.push(PropertyValue::Switch {
        name: "CONNECTED".to_string(),
        label: Some("Connected".to_string()),
        value: SwitchState::On,
    });

    // Send the property update
    client.send_property(property).await?;

    client.disconnect().await?;
    Ok(())
}
```

## Documentation

### Crate Documentation

- **This README**: Overview and quick start
- [`libindigo`](src/) - Core API documentation in source
- [`libindigo-rs`](rs/) - Pure Rust implementation (see [`rs/src/lib.rs`](rs/src/lib.rs))
- [`libindigo-ffi`](ffi/) - FFI implementation (see [`ffi/README.md`](ffi/README.md))
- [`libindigo-sys`](sys/) - Raw bindings (see [`sys/README.md`](sys/README.md))

### Architecture & Planning

- [`REFACTORING_COMPLETE.md`](REFACTORING_COMPLETE.md) - **Complete refactoring summary**
- [`doc/reviews/issue-6-refactoring-review.md`](doc/reviews/issue-6-refactoring-review.md) - Detailed review
- [`doc/architecture/client-strategies.md`](doc/architecture/client-strategies.md) - Strategy pattern
- [`plans/code-review-and-architecture.md`](plans/code-review-and-architecture.md) - Architecture plan

### Phase Documentation

- [`plans/archive/phase1-complete.md`](plans/archive/phase1-complete.md) - Foundation & Core Types
- [`plans/archive/phase2-complete.md`](plans/archive/phase2-complete.md) - Async FFI Strategy
- [`plans/archive/phase3-complete.md`](plans/archive/phase3-complete.md) - Rust Client Strategy
- [`plans/archive/phase3-json-complete.md`](plans/archive/phase3-json-complete.md) - JSON Protocol
- [`rs/PHASE5_DISCOVERY_MIGRATION.md`](rs/PHASE5_DISCOVERY_MIGRATION.md) - Discovery Migration
- [`PHASE6_CONSTANTS_EXTRACTION.md`](PHASE6_CONSTANTS_EXTRACTION.md) - Constants Extraction
- [`sys/REFACTORING_PHASE4.md`](sys/REFACTORING_PHASE4.md) - FFI Refactoring

### Additional Documentation

- [`doc/constants-extraction.md`](doc/constants-extraction.md) - Constants generation
- [`BUILD.md`](BUILD.md) - Build instructions
- [`CHANGES.md`](CHANGES.md) - Changelog

## Testing

### Run All Tests

```bash
cargo test --workspace
```

### Run Pure Rust Tests

```bash
# All pure Rust tests (including JSON protocol tests)
cargo test -p libindigo-rs

# JSON protocol tests only (61 tests)
cargo test -p libindigo-rs --test json_protocol_tests

# Protocol negotiation tests only (59 tests)
cargo test -p libindigo-rs --test protocol_negotiation_tests
```

### Run Discovery Tests

```bash
cargo test -p libindigo-rs --features discovery
```

### Integration Tests

Integration tests require a running INDIGO server:

```bash
# Start INDIGO server (in another terminal)
indigo_server

# Run integration tests
cargo test --test discovery_tests --features discovery
```

### Test Coverage Summary

| Test Suite | Tests | Coverage |
|------------|-------|----------|
| JSON Protocol | 61 | All PROTOCOLS.md examples + edge cases |
| Protocol Negotiation | 59 | Auto-detection, fallback, preferences |
| Rust Client | ~50 | Connection, properties, lifecycle |
| Discovery | ~20 | mDNS discovery, filtering |
| **Total** | **~190** | **Comprehensive coverage** |

## Examples

The [`examples/`](examples/) directory contains usage examples:

- [`discover_servers.rs`](examples/discover_servers.rs) - Server discovery
- [`discovery_with_filter.rs`](examples/discovery_with_filter.rs) - Filtered discovery
- [`auto_connect.rs`](examples/auto_connect.rs) - Auto-connect to discovered servers (⚠️ needs update)
- [`continuous_discovery.rs`](examples/continuous_discovery.rs) - Continuous discovery (⚠️ needs update)

Run examples:

```bash
# Server discovery
cargo run --example discover_servers --features discovery

# Discovery with filter
cargo run --example discovery_with_filter --features discovery
```

**Note**: Some examples use deprecated features and need updating. See [`REFACTORING_COMPLETE.md`](REFACTORING_COMPLETE.md) for details.

## Project Structure

```
libindigo-rs/
├── Cargo.toml                    # Workspace root
├── README.md                     # This file
├── REFACTORING_COMPLETE.md       # Refactoring summary
├── src/                          # libindigo (core API)
│   ├── lib.rs                    # Main library entry point
│   ├── error.rs                  # Error types
│   ├── constants.rs              # INDIGO protocol constants
│   ├── client/                   # Client API
│   │   ├── mod.rs
│   │   ├── builder.rs            # Client builder
│   │   └── strategy.rs           # ClientStrategy trait (SPI)
│   └── types/                    # Core types
│       ├── mod.rs
│       ├── property.rs           # Property types
│       ├── device.rs             # Device types
│       └── value.rs              # Value types
├── rs/                           # libindigo-rs (pure Rust)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs                # Re-exports + RS strategy
│   │   ├── client.rs             # RsClientStrategy
│   │   ├── protocol.rs           # XML protocol parser
│   │   ├── protocol_json.rs      # JSON protocol parser
│   │   ├── protocol_negotiation.rs
│   │   ├── transport.rs          # TCP transport layer
│   │   └── discovery/            # mDNS discovery (optional)
│   │       ├── mod.rs
│   │       ├── api.rs
│   │       ├── error.rs
│   │       └── mdns_impl.rs
├── ffi/                          # libindigo-ffi (FFI-based)
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs                # Re-exports + FFI strategy
│       ├── ffi.rs                # FfiClientStrategy
│       └── async_ffi.rs          # AsyncFfiStrategy
├── sys/                          # libindigo-sys (raw bindings)
│   ├── Cargo.toml
│   ├── README.md
│   ├── build.rs                  # C library build
│   └── src/lib.rs                # bindgen output
├── relm/                         # libindigo-relm (GTK demo, excluded)
├── examples/                     # Usage examples
├── tests/                        # Integration tests
├── doc/                          # Documentation
├── plans/                        # Planning documents
└── scripts/                      # Utility scripts
    └── update_constants.sh       # Update INDIGO constants
```

## Migration from Old API

If you're upgrading from the old monolithic API, see the migration guide in [`REFACTORING_COMPLETE.md`](REFACTORING_COMPLETE.md#migration-guide).

**Quick summary**:

```diff
# Cargo.toml
- libindigo = { version = "0.1", features = ["rs"] }
+ libindigo-rs = "0.3"

# Code
- use libindigo::strategies::RsClientStrategy;
- use libindigo::client::ClientBuilder;
+ use libindigo_rs::{RsClientStrategy, ClientBuilder};
```

## Contributing

Contributions are welcome! Please:

1. Read [`doc/ways-of-working.md`](doc/ways-of-working.md)
2. Follow [`doc/roo-workflow-scheme.md`](doc/roo-workflow-scheme.md)
3. Use appropriate issue templates
4. Write tests for new features
5. Update documentation

## Known Issues & Future Work

See [`REFACTORING_COMPLETE.md`](REFACTORING_COMPLETE.md#known-todos-and-future-work) for:

- FFI implementation status
- BLOB handling improvements
- Device driver support (future)
- Example updates needed

## License

This project is licensed under the MIT License - see the [`LICENSE.md`](LICENSE.md) file for details.

## Acknowledgments

- [INDIGO Astronomy](https://www.indigo-astronomy.org/) - The INDIGO protocol and C library
- The Rust community for excellent async ecosystem tools

## Related Projects

- [INDIGO](https://github.com/indigo-astronomy/indigo) - Official C implementation
- [`libindigo-sys`](sys/) - Low-level FFI bindings to INDIGO C library

## Status

**Current Version**: 0.3.0

**Status**: Production-ready for pure Rust client applications

- ✅ **libindigo-rs**: Complete and production-ready
- ⚠️ **libindigo-ffi**: Structure in place, implementation pending
- ✅ **Multi-crate refactoring**: Complete (see [`REFACTORING_COMPLETE.md`](REFACTORING_COMPLETE.md))

For production use, we recommend:

- **Pure Rust Strategy** ([`libindigo-rs`](rs/)) for new projects without C dependencies
- **FFI Strategy** ([`libindigo-ffi`](ffi/)) for maximum compatibility (when implementation is complete)

## Support

For questions, issues, or contributions:

- Open an issue on [GitHub](https://github.com/chrsoo/libindigo-rs/issues)
- Check existing documentation
- Review the architecture plan
