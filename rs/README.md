# libindigo-rs

Pure Rust implementation of the INDIGO protocol client.

## ⚠️ Important: Crate Name vs Import Name

**This crate is named `libindigo-rs` in Cargo.toml, but you must import it as `libindigo_rs` (with underscore) in your code.**

Rust automatically converts hyphens to underscores in crate names for imports. This is a Rust convention.

```rust
// ✅ CORRECT - use underscore
use libindigo_rs::{Client, ClientBuilder, RsClientStrategy};

// ❌ WRONG - this will cause "unresolved module" errors
use libindigo::{Client, ClientBuilder, RsClientStrategy};
```

## Overview

`libindigo-rs` provides a pure Rust implementation of the INDIGO astronomy protocol client. It supports both XML and JSON protocols and includes optional mDNS-based server discovery.

## Features

- **Pure Rust**: No C dependencies required
- **Async/await**: Built on Tokio for efficient async I/O
- **Protocol Support**: Both XML and JSON INDIGO protocols
- **Server Discovery**: Optional mDNS-based server discovery
- **Type-safe**: Leverages Rust's type system for safety

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
libindigo-rs = "0.3.0"
```

For server discovery support:

```toml
[dependencies]
libindigo-rs = { version = "0.3.0", features = ["discovery"] }
```

## Examples

### Basic Connection

```rust
use libindigo_rs::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::connect("localhost:7624").await?;
    // Use the client...
    Ok(())
}
```

### Property Streaming

The Rust client supports multiple concurrent subscribers to property updates using the multi-subscriber pattern:

```rust
use libindigo_rs::RsClientStrategy;
use libindigo::client::strategy::ClientStrategy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut strategy = RsClientStrategy::new();
    strategy.connect("localhost:7624").await?;

    // Subscribe to property updates
    let mut receiver = strategy.subscribe_properties().await;

    // Receive property updates in a loop
    while let Some(property) = receiver.recv().await {
        println!("Property: {}.{}", property.device, property.name);
        println!("  State: {:?}", property.state);
        println!("  Items: {}", property.items.len());
    }

    Ok(())
}
```

**Multiple Subscribers**: You can create multiple subscribers that all receive property updates independently:

```rust
// Create multiple subscribers
let mut subscriber1 = strategy.subscribe_properties().await;
let mut subscriber2 = strategy.subscribe_properties().await;

// Each subscriber receives all property updates
tokio::spawn(async move {
    while let Some(property) = subscriber1.recv().await {
        // Handle property in subscriber 1
    }
});

tokio::spawn(async move {
    while let Some(property) = subscriber2.recv().await {
        // Handle property in subscriber 2
    }
});
```

**Note**: The old `property_receiver()` method is deprecated. Use `subscribe_properties()` instead for proper multi-subscriber support.

For a complete example, see [`examples/property_streaming.rs`](../examples/property_streaming.rs).

## Features

- `client` (default): Enable client functionality
- `device`: Stub for future device driver support
- `discovery`: Enable mDNS server discovery

## Troubleshooting

### "unresolved module or unlinked crate `libindigo`" Error

If you see this error:

```
error[E0432]: unresolved import `libindigo`
```

**Solution**: Change your import from `libindigo::` to `libindigo_rs::` (with underscore).

**Why?** Rust converts hyphens in crate names to underscores for imports. The crate is named `libindigo-rs` in Cargo.toml, but must be imported as `libindigo_rs` in code.

```rust
// Change this:
use libindigo::{Client, ClientBuilder};

// To this:
use libindigo_rs::{Client, ClientBuilder};
```

For more troubleshooting help and migration guides, see [`TROUBLESHOOTING.md`](TROUBLESHOOTING.md).

## License

MIT
