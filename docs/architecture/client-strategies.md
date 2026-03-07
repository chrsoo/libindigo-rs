# Client Strategies Architecture

## Overview

The libindigo library implements a dual-strategy pattern for INDIGO client connectivity, allowing users to choose between FFI-based (C library) or pure Rust implementations.

## Strategy Pattern

The client architecture uses the Strategy pattern to abstract the underlying implementation:

```rust
#[async_trait]
pub trait ClientStrategy: Send + Sync {
    async fn connect(&mut self, url: &str) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
    async fn enumerate_properties(&mut self, device: Option<&str>) -> Result<()>;
    async fn send_property(&mut self, property: Property) -> Result<()>;
}
```

## Available Strategies

### 1. Pure Rust Strategy (RS)

**Location**: `src/strategies/rs/`

**Features**:

- Zero C dependencies at runtime
- Pure Rust implementation of INDIGO protocol
- Supports both XML and JSON protocols
- Async-first design with tokio
- Cross-platform compatibility

**Components**:

- `protocol.rs` - XML protocol parser/serializer
- `protocol_json.rs` - JSON protocol parser/serializer
- `protocol_negotiation.rs` - Protocol version negotiation
- `transport.rs` - TCP transport layer
- `client.rs` - Client strategy implementation

**Usage**:

```rust
let mut client = ClientBuilder::new()
    .with_rs_strategy()
    .build()?;
```

### 2. FFI Strategy

**Location**: `src/strategies/ffi.rs`, `src/strategies/async_ffi.rs`

**Features**:

- Uses upstream INDIGO C library
- Full hardware driver support
- Battle-tested C implementation
- Async wrapper around synchronous FFI

**Components**:

- `ffi.rs` - Synchronous FFI strategy
- `async_ffi.rs` - Async FFI strategy with tokio integration

**Usage**:

```rust
let mut client = ClientBuilder::new()
    .with_ffi_strategy()
    .build()?;
```

## Strategy Selection

### Compile-Time Selection

Use Cargo features to select strategy:

```toml
# Default: client with async support
libindigo = "0.2"

# Pure Rust strategy (zero FFI dependencies)
libindigo = { version = "0.2", default-features = false, features = ["rs"] }

# FFI strategy
libindigo = { version = "0.2", default-features = false, features = ["ffi", "async"] }
```

### Runtime Selection

Explicitly choose strategy in code:

```rust
// Pure Rust
let client = ClientBuilder::new()
    .with_rs_strategy()
    .build()?;

// FFI
let client = ClientBuilder::new()
    .with_ffi_strategy()
    .build()?;
```

## Protocol Support

### XML Protocol (Version 2.0)

- Traditional INDIGO protocol
- Full feature support
- BASE64 BLOB encoding
- Supported by both strategies

### JSON Protocol (Version 512)

- Modern JSON-based protocol
- Better web integration
- URL-referenced BLOBs only
- Supported by RS strategy

## Implementation Status

| Feature | RS Strategy | FFI Strategy |
|---------|-------------|--------------|
| XML Protocol | ✅ Complete | ✅ Complete |
| JSON Protocol | ✅ Complete | ❌ Not available |
| Protocol Negotiation | ✅ Complete | ✅ Complete |
| Async API | ✅ Native | ✅ Wrapped |
| Property Streaming | ✅ Complete | ✅ Complete |
| BLOB Support | ✅ Complete | ✅ Complete |
| Hardware Drivers | ❌ N/A | ✅ Complete |

## Performance Considerations

### Pure Rust Strategy

**Advantages**:

- No FFI overhead
- Native async/await
- Better error handling
- Memory safety guarantees

**Trade-offs**:

- No hardware driver support
- Protocol implementation maintenance

### FFI Strategy

**Advantages**:

- Full hardware support
- Proven C implementation
- Access to all INDIGO features

**Trade-offs**:

- FFI overhead
- Async wrapping complexity
- C library dependencies

## Testing

Both strategies have comprehensive test coverage:

- Unit tests for protocol parsing/serialization
- Integration tests against live INDIGO server
- Protocol compliance tests
- Performance benchmarks

See `tests/` directory for test implementations.

## Future Enhancements

1. **WebSocket Transport** - Add WebSocket support for web clients
2. **Protocol Extensions** - Support for custom protocol extensions
3. **Connection Pooling** - Reuse connections across clients
4. **Automatic Failover** - Switch strategies on connection failure

## References

- [INDIGO Protocol Documentation](../INDIGO.pdf)
- [Transport Implementation](../transport_implementation.md)
- [Code Review and Architecture](../../plans/code-review-and-architecture.md)
