# RELM Demonstrator Application

A GTK4 demo application built with Relm4 to demonstrate the `libindigo-rs` pure Rust API for INDIGO astronomy clients.

## Status: Refactored for New API ✅

The relm application has been **successfully refactored** to use the new `libindigo-rs` pure Rust API (Phase 1-6 refactoring complete).

### What Changed

- **Old API**: Used deprecated FFI types (`SysBus`, `SysClientController`, `SysRemoteResource`)
- **New API**: Now uses `libindigo-rs` with `Client` + `RsClientStrategy`
- **Current State**: Code updated and compiles (requires GTK4 system libraries to build)

## Building and Running

### Prerequisites

This application requires GTK4 system libraries to be installed:

#### macOS

```bash
brew install gtk4
```

**Important for macOS**: After installing GTK4 via Homebrew, you must use the provided build script to ensure the correct `pkg-config` is used:

```bash
cd relm
./build.sh
```

Or set the environment variable manually:

```bash
export PKG_CONFIG=/opt/homebrew/bin/pkg-config
cargo build
```

**Why?** macOS may have multiple `pkg-config` installations. The system default at `/usr/local/bin/pkg-config` doesn't know about Homebrew's library paths at `/opt/homebrew`. The build script ensures Homebrew's `pkg-config` is used, which can find GTK4 and its dependencies.

#### Ubuntu/Debian

```bash
sudo apt-get install libgtk-4-dev
```

#### Fedora/RHEL

```bash
sudo dnf install gtk4-devel
```

### Build

#### macOS

```bash
cd relm
./build.sh
```

#### Linux

```bash
cd relm
cargo build
```

### Run

```bash
cargo run
```

## Architecture

The application demonstrates the new `libindigo-rs` API:

### Client Initialization

```rust
use libindigo_rs::{Client, ClientBuilder, RsClientStrategy};

// Create a client with the pure Rust strategy
let strategy = RsClientStrategy::new();
let client = ClientBuilder::new()
    .with_strategy(Box::new(strategy))
    .build()?;
```

### Property Handling

```rust
use libindigo::types::{Property, PropertyValue, PropertyItem};

// Properties use the new type-safe API
match &property_item.value {
    PropertyValue::Text(text) => { /* handle text */ },
    PropertyValue::Number { value, min, max, step, format } => { /* handle number */ },
    PropertyValue::Switch { state } => { /* handle switch */ },
    PropertyValue::Light { state } => { /* handle light */ },
    PropertyValue::Blob { data, format, size } => { /* handle blob */ },
}
```

### Server Connection

```rust
use libindigo::name::{INDIGO_DEFAULT_HOST, INDIGO_DEFAULT_PORT};

// Constants are now in the libindigo::name module
let host = INDIGO_DEFAULT_HOST; // "localhost"
let port = INDIGO_DEFAULT_PORT; // 7624
```

## Features Demonstrated

The `libindigo-relm` client successfully demonstrates:

- ✅ Connection UI for INDIGO servers
- ✅ Pure Rust client initialization with `RsClientStrategy`
- ✅ Type-safe property handling with new API
- ✅ Rendering of TEXT properties
- ✅ Rendering of NUMBER properties
- ✅ Rendering of SWITCH properties
- ✅ Rendering of LIGHT properties
- ✅ Rendering of BLOB properties (basic)
- ✅ Device list management

## Known Limitations

As a technology demonstration app, the `libindigo-relm` client has some limitations:

- **Async Integration**: The GTK UI is synchronous while the INDIGO client is async. Full integration requires spawning tokio tasks and using channels to communicate between async and sync contexts.
- **Property Updates**: Property UPDATE events may not fully refresh the UI in all cases.
- **Connection Management**: Reconnection and disconnection logic needs async task management.
- **Property Editing**: Editing of INDIGO properties is not yet implemented.
- **Scrolling**: Long lists of device properties may not scroll properly.
- **Switch Rules**: The `SwitchRule` is not fully respected when rendering SWITCH properties.

## Implementation Notes

### Async/Sync Bridge

The main challenge in this demo is bridging GTK's synchronous event loop with the async INDIGO client:

```rust
// Current approach: Client is created but not fully connected
let strategy = RsClientStrategy::new();
let client = ClientBuilder::new()
    .with_strategy(Box::new(strategy))
    .build()?;

// TODO: Full async connection requires:
// 1. Spawn a tokio runtime in a background thread
// 2. Use channels (e.g., tokio::sync::mpsc) to communicate
// 3. Forward INDIGO events to GTK via glib::MainContext
```

### Property Event Handling

The application uses Relm4's message broker pattern to handle property events:

```rust
static BROKER: MessageBroker<AppInput> = MessageBroker::new();

// Events are sent through the broker
BROKER.send(AppInput::PropertyDefined { data, msg });
```

## Reference Implementation

For working examples of the new API, see the `examples/` directory in the workspace root:

- [`examples/discover_servers.rs`](../examples/discover_servers.rs) - Server discovery
- [`examples/auto_connect.rs`](../examples/auto_connect.rs) - Client connection
- [`examples/continuous_discovery.rs`](../examples/continuous_discovery.rs) - Property monitoring

## Development

### Project Structure

```
relm/
├── src/
│   ├── main.rs      # Application entry point and main component
│   ├── device.rs    # Device component (manages properties)
│   ├── property.rs  # Property rendering components
│   └── server.rs    # Server connection UI component
├── Cargo.toml       # Dependencies (standalone workspace)
└── README.md        # This file
```

### Dependencies

- **libindigo-rs**: Pure Rust INDIGO client implementation
- **relm4**: GTK4 reactive UI framework
- **gtk4**: GTK4 bindings for Rust
- **tokio**: Async runtime (for future async integration)

## Contributing

This is a demonstration application. Contributions to improve the async/sync integration or add missing features are welcome!

## License

MIT License - See the workspace root for details.
