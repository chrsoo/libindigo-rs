# libindigo-rs

Pure Rust implementation of the INDIGO protocol client.

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

## Example

```rust
use libindigo_rs::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::connect("localhost:7624").await?;
    // Use the client...
    Ok(())
}
```

## Features

- `client` (default): Enable client functionality
- `device`: Stub for future device driver support
- `discovery`: Enable mDNS server discovery

## License

MIT
