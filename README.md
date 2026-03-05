# libindigo-rs

Rust API for writing client applications and device drivers for astronomy equipment using the [INDIGO](https://www.indigo-astronomy.org/index.html) protocol and architecture.

> [!NOTE]
> **Phase 3 Complete!** ✅ The pure Rust client strategy is now fully implemented and production-ready with **JSON and XML protocol support**. See [plans/archive/phase3-complete.md](plans/archive/phase3-complete.md) for details.

## Implementation Status

- ✅ **Phase 1**: Foundation & Core Types (Complete) - [Details](plans/archive/phase1-complete.md)
- ✅ **Phase 2**: Async FFI Strategy (Complete) - [Details](plans/archive/phase2-complete.md)
- ✅ **Phase 3**: Rust Client Strategy (Complete) - [Details](plans/archive/phase3-complete.md)
- 🚧 **Phase 4**: Device Driver Support (Planned)

## Features

### Rust Strategy (Phase 3) ✅

The pure Rust strategy provides a complete INDIGO client implementation without C FFI dependencies:

- **Zero FFI**: No C dependencies, pure Rust implementation
- **Async-First**: Built on tokio for efficient async I/O
- **Type Safe**: Leverages Rust's type system for protocol correctness
- **Cross-Platform**: Works anywhere Rust compiles
- **Dual Protocol**: Full INDIGO JSON and XML protocol support with automatic negotiation
- **JSON-First**: Defaults to modern JSON protocol with XML fallback

### FFI Strategy (Phase 2) ✅

The FFI strategy wraps the official C INDIGO library:

- **Maximum Compatibility**: Uses the official INDIGO C library
- **Async Support**: Async wrappers around synchronous FFI calls
- **Battle-Tested**: Leverages mature C implementation
- **Feature Complete**: Access to all INDIGO features

## Quick Start

### Rust Client (No C Dependencies)

```rust
use libindigo::client::ClientBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with pure Rust strategy
    let mut client = ClientBuilder::new()
        .with_rs_strategy()
        .build()?;

    // Connect to INDIGO server
    client.connect("localhost:7624").await?;

    // Enumerate all properties
    client.enumerate_properties(None).await?;

    // Disconnect
    client.disconnect().await?;

    Ok(())
}
```

### FFI-Based Client (C INDIGO Library)

```rust
use libindigo::client::ClientBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with async FFI strategy
    let mut client = ClientBuilder::new()
        .with_async_ffi_strategy()
        .build()?;

    // Connect to INDIGO server
    client.connect("localhost:7624").await?;

    // Enumerate all properties
    client.enumerate_properties(None).await?;

    // Disconnect
    client.disconnect().await?;

    Ok(())
}
```

## JSON Protocol Support (Phase 3 Enhancement) ✅

libindigo now supports both INDIGO JSON and XML protocols with intelligent negotiation:

### Automatic Protocol Negotiation (Default)

The client automatically negotiates the best protocol with the server:

```rust
use libindigo::client::ClientBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // JSON-first with XML fallback (default behavior)
    let mut client = ClientBuilder::new()
        .with_rs_strategy()
        .build()?;

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

#### JSON-First with XML Fallback (Recommended)

```rust
use libindigo::client::ClientBuilder;

let mut client = ClientBuilder::new()
    .with_rs_strategy()
    .build()?;

// Automatically tries JSON first, falls back to XML
client.connect("localhost:7624").await?;
```

#### Force JSON Protocol Only

```rust
use libindigo::strategies::RsClientStrategy;
use libindigo::strategies::rs::protocol_negotiation::ProtocolType;

let mut strategy = RsClientStrategy::new();
strategy.set_preferred_protocol(ProtocolType::Json);
strategy.set_allow_fallback(false); // No XML fallback

strategy.connect("localhost:7624").await?;
```

#### Force XML Protocol Only

```rust
use libindigo::strategies::RsClientStrategy;
use libindigo::strategies::rs::protocol_negotiation::ProtocolType;

let mut strategy = RsClientStrategy::new();
strategy.set_preferred_protocol(ProtocolType::Xml);
strategy.set_allow_fallback(false);

strategy.connect("localhost:7624").await?;
```

#### Check Negotiated Protocol

```rust
use libindigo::strategies::RsClientStrategy;

let mut strategy = RsClientStrategy::new();
strategy.connect("localhost:7624").await?;

// Check which protocol was negotiated
let protocol = strategy.negotiated_protocol().await;
println!("Using protocol: {}", protocol); // "JSON" or "XML"
```

### JSON Protocol Features

- ✅ **Full PROTOCOLS.md Compliance**: Implements all examples from INDIGO PROTOCOLS.md
- ✅ **All Message Types**: `getProperties`, `defXXXVector`, `setXXXVector`, `newXXXVector`, `delProperty`, `message`
- ✅ **All Property Types**: Text, Number, Switch, Light, BLOB
- ✅ **Native JSON Types**: Uses JSON booleans, numbers, and strings appropriately
- ✅ **Efficient Parsing**: Fast JSON parsing with `serde_json`
- ✅ **Type Safety**: Strong typing with Rust's type system
- ✅ **120 Tests**: Comprehensive test coverage including all PROTOCOLS.md examples

### Known Limitations

- **JSON BLOB Encoding**: JSON protocol only supports URL-based BLOBs (not BASE64 inline)
  - This is per INDIGO specification
  - XML protocol supports both URL and BASE64
  - For inline BLOB data, use XML protocol

### Migration from XML-Only

No code changes required! The pure Rust strategy now automatically uses JSON with XML fallback:

```rust
// This code works with both JSON and XML servers
let mut client = ClientBuilder::new()
    .with_rs_strategy()
    .build()?;

client.connect("localhost:7624").await?;
```

For more details, see [plans/archive/phase3-json-complete.md](plans/archive/phase3-json-complete.md).

## Installation

Add to your `Cargo.toml`:

### Rust (Recommended)

```toml
[dependencies]
libindigo = { version = "0.1", features = ["rs"] }
tokio = { version = "1.35", features = ["full"] }
```

### FFI-Based

```toml
[dependencies]
libindigo = { version = "0.1", features = ["ffi", "async"] }
tokio = { version = "1.35", features = ["full"] }
```

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `async` | Enable async/await support | ✅ Yes |
| `ffi` | Enable FFI-based strategy using C INDIGO library | ✅ Yes |
| `rs` | Enable pure Rust strategy implementation | ❌ No |
| `blocking` | Enable synchronous wrappers around async APIs | ❌ No |
| `sys` | Include low-level FFI bindings | ✅ Yes |
| `std` | Enable standard library features | ✅ Yes |
| `auto` | Enable auto-discovery features | ✅ Yes |

## Strategy Comparison

| Feature | Rust | Async FFI | Sync FFI |
|---------|------|-----------|----------|
| **C Dependencies** | ❌ None | ✅ Required | ✅ Required |
| **Async Support** | ✅ Native | ✅ Wrapped | ❌ No |
| **Cross-Platform** | ✅ Excellent | ⚠️ Limited | ⚠️ Limited |
| **Performance** | ✅ Fast | ✅ Fast | ✅ Fast |
| **JSON Protocol** | ✅ Yes | ⚠️ Via C lib | ⚠️ Via C lib |
| **XML Protocol** | ✅ Yes | ✅ Yes | ✅ Yes |
| **Protocol Negotiation** | ✅ Automatic | ❌ No | ❌ No |
| **Maturity** | ✅ Production | ✅ Stable | ✅ Stable |
| **Use Case** | Modern apps | Legacy compat | Sync apps |

## Usage Examples

### Receiving Property Updates

```rust
use libindigo::client::ClientBuilder;
use libindigo::strategies::RsClientStrategy;

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
use libindigo::client::ClientBuilder;
use libindigo::types::{Property, PropertyType, PropertyValue, SwitchState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ClientBuilder::new()
        .with_rs_strategy()
        .build()?;

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

## Architecture

The library uses a strategy pattern to support multiple implementations:

```
┌─────────────────────────────────────┐
│         Client Builder              │
│  (Fluent API for construction)      │
└──────────────┬──────────────────────┘
               │
       ┌───────┴────────┐
       │                │
       ▼                ▼
┌──────────────┐  ┌──────────────┐
│ FFI Strategy │  │ Rust         │
│              │  │ Strategy     │
│ - Async FFI  │  │              │
│ - Sync FFI   │  │ - Protocol   │
│              │  │ - Transport  │
└──────────────┘  └──────────────┘
```

### Rust Strategy Architecture

```
┌─────────────────────────────────────────────────────────┐
│             RsClientStrategy                            │
│  ┌───────────────────────────────────────────────────┐ │
│  │  ClientState (Arc<Mutex<>>)                       │ │
│  │  - transport: Transport                           │ │
│  │  - property_tx/rx: mpsc channels                  │ │
│  │  - background_task: JoinHandle                    │ │
│  └───────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
                    │                    │
        ┌───────────┴──────────┐        │
        │                      │        │
        ▼                      ▼        ▼
┌──────────────┐      ┌──────────────┐ ┌──────────────┐
│  Transport   │      │   Protocol   │ │  Background  │
│              │      │              │ │     Task     │
│ - TCP Stream │      │ - Parser     │ │              │
│ - Buffering  │      │ - Serializer │ │ - Receive    │
│ - Framing    │      │ - Messages   │ │ - Convert    │
└──────────────┘      └──────────────┘ │ - Forward    │
                                        └──────────────┘
```

## Documentation

- [API Documentation](https://docs.rs/libindigo) (coming soon)
- [Phase 1 Complete](plans/archive/phase1-complete.md) - Foundation & Core Types
- [Phase 2 Complete](plans/archive/phase2-complete.md) - Async FFI Strategy
- [Phase 3 Complete](plans/archive/phase3-complete.md) - Rust Client Strategy
- [Phase 3 JSON Protocol](plans/archive/phase3-json-complete.md) - JSON Protocol Implementation
- [JSON Protocol Tests](tests/JSON_PROTOCOL_TEST_SUMMARY.md) - Test Coverage Summary
- [Architecture Plan](plans/code-review-and-architecture.md)
- [Known Issues](plans/issues.md)
- [Build Instructions](BUILD.md)

## Testing

### Run All Tests

```bash
cargo test --all-features
```

### Run Rust Strategy Tests

```bash
# All pure Rust tests (including JSON protocol tests)
cargo test --features rs

# JSON protocol tests only (61 tests)
cargo test --test json_protocol_tests --features rs

# Protocol negotiation tests only (59 tests)
cargo test --test protocol_negotiation_tests --features rs
```

### Run FFI Strategy Tests

```bash
cargo test --features ffi
```

### Integration Tests

Integration tests require a running INDIGO server:

```bash
# Start INDIGO server (in another terminal)
indigo_server

# Run integration tests
cargo test --test rs_client_integration --features rs
```

### Test Coverage Summary

| Test Suite | Tests | Coverage |
|------------|-------|----------|
| JSON Protocol | 61 | All PROTOCOLS.md examples + edge cases |
| Protocol Negotiation | 59 | Auto-detection, fallback, preferences |
| Rust Client | ~50 | Connection, properties, lifecycle |
| **Total New Tests** | **120** | **Comprehensive JSON support** |

## Project Structure

```
libindigo/
├── src/
│   ├── lib.rs                      # Main library entry point
│   ├── error.rs                    # Error types
│   ├── client/                     # Client API
│   │   ├── mod.rs
│   │   ├── builder.rs              # Client builder
│   │   └── strategy.rs             # Strategy trait
│   ├── types/                      # Core types
│   │   ├── mod.rs
│   │   ├── property.rs             # Property types
│   │   ├── device.rs               # Device types
│   │   └── value.rs                # Value types
│   └── strategies/                 # Strategy implementations
│       ├── mod.rs
│       ├── ffi.rs                  # Sync FFI strategy
│       ├── async_ffi.rs            # Async FFI strategy
│       └── rs/                     # Rust strategy (Phase 3)
│           ├── mod.rs
│           ├── client.rs           # Client implementation
│           ├── protocol.rs         # XML protocol parser
│           └── transport.rs        # TCP transport layer
├── sys/                            # FFI bindings crate
├── tests/                          # Integration tests
├── Cargo.toml
└── README.md
```

## Contributing

Contributions are welcome! Please see our contributing guidelines (coming soon).

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

## Acknowledgments

- [INDIGO Astronomy](https://www.indigo-astronomy.org/) - The INDIGO protocol and C library
- The Rust community for excellent async ecosystem tools

## Related Projects

- [INDIGO](https://github.com/indigo-astronomy/indigo) - Official C implementation
- [libindigo-sys](sys/) - Low-level FFI bindings to INDIGO C library

## Status

This library is under active development. The pure Rust client strategy (Phase 3) is complete and production-ready. Device driver support (Phase 4) is planned for future releases.

For production use, we recommend:

- **Rust Strategy** for new projects without C dependencies
- **Async FFI Strategy** for maximum compatibility with existing INDIGO ecosystem

## Support

For questions, issues, or contributions:

- Open an issue on GitHub
- Check existing documentation
- Review the architecture plan
