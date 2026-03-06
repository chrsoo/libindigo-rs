# libindigo-ffi

FFI-based implementation of the INDIGO protocol client using the C INDIGO library.

## Overview

This crate provides an implementation of the INDIGO astronomy protocol using FFI bindings to the official C INDIGO library. It offers maximum compatibility with the reference implementation while providing a safe Rust API.

## Status

⚠️ **Work in Progress** - This crate currently contains stub implementations. The FFI integration with the C INDIGO library is planned for future development.

## Features

- **C Library Integration**: Uses the official INDIGO C library via FFI (planned)
- **Maximum Compatibility**: Guaranteed compatibility with INDIGO servers (planned)
- **Safe API**: Wraps unsafe FFI calls in a safe Rust interface
- **Optional Async**: Async wrapper available with the `async` feature

## Feature Flags

- `client` (default): Enable client functionality
- `device`: Enable device driver support (future)
- `async`: Enable async wrapper for non-blocking operations

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
libindigo-ffi = "0.3"
```

### Basic Example

```rust
use libindigo_ffi::{Client, ClientBuilder, FfiClientStrategy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Note: Currently returns NotSupported error
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

### Async Support

Enable the `async` feature for async wrapper support:

```toml
[dependencies]
libindigo-ffi = { version = "0.3", features = ["async"] }
```

```rust
use libindigo_ffi::{AsyncFfiStrategy, PropertyStream};

// Note: Currently returns NotSupported error
let strategy = AsyncFfiStrategy::new()?;
// Use with Client as normal
```

## Architecture

The crate is organized into several layers:

- **Core API** (re-exported from `libindigo`): Types, traits, and error handling
- **FFI Layer**: Safe wrappers around C INDIGO library calls (planned)
- **Client Layer**: Implementation of `ClientStrategy` trait (stub)
- **Async Layer** (optional): Async wrapper for non-blocking operations (stub)

## Comparison with libindigo-rs

| Feature | libindigo-ffi | libindigo-rs |
|---------|---------------|--------------|
| C Dependencies | Yes (libindigo-sys) | No |
| Implementation | FFI to C library | Pure Rust |
| Compatibility | Maximum | Protocol-level |
| Status | Planned | Complete |
| Cross-platform | Limited by C library | Full Rust support |

## Development Status

This crate is part of Phase 3 of the libindigo-rs refactoring (Issue #6). The structure is in place, but the actual FFI implementation is pending.

### Current State

- ✅ Crate structure created
- ✅ Dependencies configured
- ✅ API surface defined
- ⚠️ FFI implementation pending
- ⚠️ Async wrapper pending

### Next Steps

1. Implement FFI bindings to C INDIGO client functions
2. Add type conversion between Rust and C types
3. Implement callback handling for property updates
4. Add memory management for FFI boundary
5. Implement async wrapper with tokio
6. Add comprehensive tests

## Dependencies

- [`libindigo`](../README.md) - Core API with traits and types
- [`libindigo-sys`](../sys/README.md) - Low-level FFI bindings to C library
- `async-trait` - Async trait support
- `thiserror` - Error handling
- `tracing` - Logging
- `tokio` (optional) - Async runtime for async feature

## License

This project is licensed under the MIT License - see the [LICENSE.md](../LICENSE.md) file for details.

## Contributing

Contributions are welcome! This crate needs implementation work. See the [GitHub issues](https://github.com/chrsoo/libindigo-rs/issues) for planned work.
