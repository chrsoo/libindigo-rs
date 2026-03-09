# Mock INDIGO Server Architecture

## Overview

The mock INDIGO server is a pure Rust implementation of an INDIGO protocol server designed specifically for testing the libindigo-rs client library. It provides a lightweight, zero-dependency alternative to running a full INDIGO server with hardware drivers, enabling fast and reliable integration tests.

**Key Features:**

- Pure Rust implementation with zero FFI dependencies
- Async/await using tokio runtime
- JSON protocol support (version 512)
- Support for basic message types: `getProperties`, `def*Vector`, `set*Vector`
- Maintains internal state of devices and properties
- Handles multiple concurrent client connections
- Property streaming capability (continuous updates)
- Preset device configurations (CCD, Mount simulators)

**Location**: [`tests/mock_server/`](../../tests/mock_server/)

## Architecture

### Component Overview

The mock server follows a modular architecture with clear separation of concerns:

```
tests/mock_server/
├── mod.rs              # Public API and re-exports
├── builder.rs          # Fluent builder for server configuration
├── server.rs           # Main server implementation
├── connection.rs       # Per-client connection handler
├── handler.rs          # Message routing and handling
├── device.rs           # Mock device management
├── property.rs         # Property state management
├── subscription.rs     # Property subscription tracking
└── presets/            # Preset device configurations
    ├── mod.rs
    ├── ccd.rs          # CCD camera simulator
    └── mount.rs        # Mount simulator
```

### Core Components

#### 1. MockIndigoServer

**Location**: [`server.rs`](../../tests/mock_server/server.rs)

The main server struct that coordinates all components and manages the server lifecycle.

```rust
pub struct MockIndigoServer {
    config: ServerConfig,
    addr: SocketAddr,
    state: Arc<ServerState>,
    shutdown_tx: broadcast::Sender<()>,
    task_handle: Option<JoinHandle<Result<()>>>,
}
```

**Responsibilities:**

- Bind TCP listener and accept connections
- Manage shared server state
- Coordinate graceful shutdown
- Spawn property updater for streaming (optional)
- Track server statistics

#### 2. ServerState

**Location**: [`server.rs`](../../tests/mock_server/server.rs)

Thread-safe shared state accessible by all connections.

```rust
pub struct ServerState {
    pub devices: RwLock<DeviceRegistry>,
    pub subscriptions: RwLock<SubscriptionManager>,
    pub connection_count: AtomicUsize,
    pub stats: RwLock<ServerStats>,
}
```

**Concurrency Model:**

- Uses `RwLock` for read-heavy operations (device/property lookups)
- Uses `AtomicUsize` for lock-free counters
- Enables multiple concurrent readers with single writer

#### 3. Connection

**Location**: [`connection.rs`](../../tests/mock_server/connection.rs)

Handles a single client connection with its own tokio task.

```rust
pub struct Connection {
    id: usize,
    stream: TcpStream,
    state: Arc<ServerState>,
    shutdown_rx: broadcast::Receiver<()>,
    update_rx: mpsc::UnboundedReceiver<ProtocolMessage>,
    update_tx: mpsc::UnboundedSender<ProtocolMessage>,
    negotiated: bool,
}
```

**Responsibilities:**

- Read messages from client (newline-delimited JSON)
- Parse protocol messages using [`JsonProtocolParser`](../../rs/src/protocol_json.rs)
- Route messages to [`MessageHandler`](../../tests/mock_server/handler.rs)
- Send responses and property updates to client
- Handle graceful disconnection

#### 4. MessageHandler

**Location**: [`handler.rs`](../../tests/mock_server/handler.rs)

Routes and processes INDIGO protocol messages.

**Supported Messages:**

- `getProperties` - Returns property definitions
- `newTextVector` - Updates text properties
- `newNumberVector` - Updates number properties
- `newSwitchVector` - Updates switch properties
- `enableBLOB` - Acknowledges BLOB configuration (no-op)

**Message Flow:**

1. Parse incoming message
2. Validate device/property exists
3. Apply updates to internal state
4. Generate response message
5. Notify all subscribers of changes

#### 5. DeviceRegistry

**Location**: [`device.rs`](../../tests/mock_server/device.rs)

Manages mock devices and their properties.

```rust
pub struct DeviceRegistry {
    devices: HashMap<String, MockDevice>,
}

pub struct MockDevice {
    pub name: String,
    pub interface: u32,
    pub properties: HashMap<String, MockProperty>,
    pub metadata: DeviceMetadata,
}
```

**Operations:**

- Add/remove devices
- Query devices by name
- List all devices
- Add/update/query properties
- Filter properties by device

#### 6. MockProperty

**Location**: [`property.rs`](../../tests/mock_server/property.rs)

Represents a property with its current state and values.

```rust
pub struct MockProperty {
    pub device: String,
    pub name: String,
    pub group: String,
    pub label: String,
    pub state: PropertyState,
    pub perm: PropertyPerm,
    pub property_type: PropertyType,
    pub items: Vec<PropertyItem>,
    pub timeout: Option<f64>,
    pub timestamp: Option<String>,
    pub message: Option<String>,
    pub type_metadata: PropertyTypeMetadata,
}
```

**Property Types:**

- `Text` - String values
- `Number` - Numeric values with format/min/max/step
- `Switch` - Boolean values with switch rules
- `Light` - Status indicators
- `Blob` - Binary data (URL references only)

#### 7. SubscriptionManager

**Location**: [`subscription.rs`](../../tests/mock_server/subscription.rs)

Tracks which clients are subscribed to which properties.

```rust
pub struct SubscriptionManager {
    subscriptions: HashMap<usize, ClientSubscription>,
}

pub struct ClientSubscription {
    pub connection_id: usize,
    pub device_filter: Option<String>,
    pub property_filter: Option<String>,
    pub sender: mpsc::UnboundedSender<ProtocolMessage>,
}
```

**Features:**

- Subscribe/unsubscribe connections
- Filter subscriptions by device/property
- Broadcast property updates to matching subscribers
- Non-blocking sends (drops if channel full)

#### 8. MockServerBuilder

**Location**: [`builder.rs`](../../tests/mock_server/builder.rs)

Fluent builder API for server configuration.

```rust
pub struct MockServerBuilder {
    config: ServerConfig,
    devices: Vec<MockDevice>,
}
```

**Configuration Options:**

- Bind address (default: `127.0.0.1:0` for random port)
- Maximum concurrent connections
- Property streaming interval
- Verbose logging
- Preset devices (CCD, Mount)

## Protocol Implementation

### Message Framing

INDIGO protocol uses newline-delimited JSON:

- Each message is a single line of JSON
- Messages are terminated by `\n`
- Connection reads lines asynchronously
- Parser handles one message at a time

### Protocol Negotiation

The server supports JSON protocol version 512:

1. Client sends `getProperties` with `version: 512`
2. Server validates version (always accepts 512)
3. Server responds with property definitions
4. Connection is marked as negotiated

### Message Conversion

The server converts between internal [`MockProperty`](../../tests/mock_server/property.rs) representation and protocol messages:

**Property Definition (`def*Vector`):**

- Sent when client requests properties
- Contains full property metadata
- Includes all items with their constraints

**Property Update (`set*Vector`):**

- Sent when property values change
- Contains only changed items
- Includes state and timestamp

**Conversion Functions:**

- [`property_to_def_message()`](../../tests/mock_server/handler.rs:252) - MockProperty → def*Vector
- [`property_to_set_message()`](../../tests/mock_server/handler.rs:370) - MockProperty → set*Vector

## Preset Devices

### CCD Simulator

**Location**: [`presets/ccd.rs`](../../tests/mock_server/presets/ccd.rs)

Simulates a CCD camera with the following properties:

| Property | Type | Permission | Description |
|----------|------|------------|-------------|
| `CONNECTION` | Switch | RW | Connection state (CONNECTED/DISCONNECTED) |
| `CCD_EXPOSURE` | Number | RW | Exposure time (0.001-3600s) |
| `CCD_TEMPERATURE` | Number | RW | CCD temperature (-50 to 50°C) |
| `CCD_INFO` | Number | RO | Sensor info (width, height, pixel size) |

**Usage:**

```rust
let server = MockServerBuilder::new()
    .with_ccd_simulator()
    .build()
    .await?;
```

### Mount Simulator

**Location**: [`presets/mount.rs`](../../tests/mock_server/presets/mount.rs)

Simulates a telescope mount with the following properties:

| Property | Type | Permission | Description |
|----------|------|------------|-------------|
| `CONNECTION` | Switch | RW | Connection state (CONNECTED/DISCONNECTED) |
| `MOUNT_EQUATORIAL_COORDINATES` | Number | RW | RA/DEC coordinates |
| `MOUNT_PARK` | Switch | RW | Park state (PARKED/UNPARKED) |

**Usage:**

```rust
let server = MockServerBuilder::new()
    .with_mount_simulator()
    .build()
    .await?;
```

## Usage Examples

### Basic Server Setup

```rust
use tests::mock_server::MockServerBuilder;

#[tokio::test]
async fn test_basic_server() {
    // Create and start mock server
    let server = MockServerBuilder::new()
        .with_ccd_simulator()
        .with_mount_simulator()
        .build()
        .await
        .unwrap();

    let addr = server.addr();
    println!("Server listening on {}", addr);

    // Use server for testing...

    // Shutdown server
    server.shutdown().await.unwrap();
}
```

### Property Streaming

Enable automatic property updates at regular intervals:

```rust
use std::time::Duration;

let server = MockServerBuilder::new()
    .with_ccd_simulator()
    .with_streaming(Duration::from_millis(100))  // Update every 100ms
    .verbose()
    .build()
    .await
    .unwrap();
```

The server will automatically update simulated properties (e.g., CCD temperature) and broadcast changes to all subscribers.

### Manual Property Updates

Update properties programmatically for testing:

```rust
use tests::mock_server::{PropertyUpdate, PropertyValue, NumberValue};
use libindigo_rs::protocol::PropertyState;

// Update CCD temperature
server.update_property(
    "CCD Simulator",
    "CCD_TEMPERATURE",
    PropertyUpdate {
        state: Some(PropertyState::Ok),
        items: vec![
            ("CCD_TEMPERATURE_VALUE".to_string(), PropertyValue::Number(NumberValue {
                value: -10.5,
                format: "%.2f".to_string(),
                min: -50.0,
                max: 50.0,
                step: 0.1,
            }))
        ],
        message: Some("Temperature updated".to_string()),
    }
).await.unwrap();

// Verify update
let property = server.get_property("CCD Simulator", "CCD_TEMPERATURE").await.unwrap();
assert_eq!(property.state, PropertyState::Ok);
```

### Server Statistics

Query server statistics for test assertions:

```rust
let stats = server.stats().await;
println!("Total connections: {}", stats.total_connections);
println!("Active connections: {}", stats.active_connections);
println!("Messages received: {}", stats.messages_received);
println!("Messages sent: {}", stats.messages_sent);
```

### Custom Device Configuration

Create custom devices for specific test scenarios:

```rust
use tests::mock_server::{MockDevice, MockProperty, PropertyType, PropertyItem, PropertyValue};
use libindigo_rs::protocol::{PropertyState, PropertyPerm};

let mut device = MockDevice {
    name: "Custom Device".to_string(),
    interface: 0x01,
    properties: HashMap::new(),
    metadata: DeviceMetadata::default(),
};

// Add custom property
device.properties.insert("CUSTOM_PROP".to_string(), MockProperty {
    device: "Custom Device".to_string(),
    name: "CUSTOM_PROP".to_string(),
    group: "Main".to_string(),
    label: "Custom Property".to_string(),
    state: PropertyState::Idle,
    perm: PropertyPerm::ReadWrite,
    property_type: PropertyType::Text,
    items: vec![
        PropertyItem {
            name: "VALUE".to_string(),
            label: "Value".to_string(),
            value: PropertyValue::Text("test".to_string()),
        }
    ],
    timeout: None,
    timestamp: None,
    message: None,
    type_metadata: PropertyTypeMetadata::Text,
});

let server = MockServerBuilder::new()
    .with_device(device)
    .build()
    .await
    .unwrap();
```

## Integration with Test Framework

### Test Structure

The mock server is designed to work seamlessly with Rust's test framework:

```rust
#[tokio::test]
async fn test_client_with_mock_server() {
    // 1. Start mock server
    let server = MockServerBuilder::new()
        .with_ccd_simulator()
        .build()
        .await
        .unwrap();

    let addr = server.addr();

    // 2. Create client and connect
    let mut client = RsClientStrategy::new();
    client.connect(&format!("localhost:{}", addr.port())).await.unwrap();

    // 3. Test client operations
    // ... test code ...

    // 4. Cleanup
    client.disconnect().await.unwrap();
    server.shutdown().await.unwrap();
}
```

### Example Integration Test

**Location**: [`tests/mock_server_test.rs`](../../tests/mock_server_test.rs)

```rust
#[tokio::test]
async fn test_mock_server_connection() {
    let server = MockServerBuilder::new()
        .with_ccd_simulator()
        .build()
        .await
        .expect("Failed to start mock server");

    let addr = server.addr();

    // Connect to server
    let stream = TcpStream::connect(addr).await.unwrap();
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Send getProperties request
    let request = r#"{"getProperties":{"version":512}}"#;
    writer.write_all(request.as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.flush().await.unwrap();

    // Read responses (should get property definitions)
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();

    // Verify response contains property definition
    assert!(line.contains("defNumberVector") || line.contains("defSwitchVector"));

    server.shutdown().await.unwrap();
}
```

## Design Decisions

### Why Pure Rust?

**Advantages:**

- No FFI overhead or complexity
- No C library dependencies
- Faster test execution
- Better error messages
- Memory safety guarantees
- Cross-platform compatibility

**Trade-offs:**

- Must maintain protocol implementation
- No hardware driver support (not needed for testing)
- Limited to JSON protocol (sufficient for testing)

### Why JSON Protocol Only?

The mock server only supports JSON protocol (version 512) because:

1. **Simplicity** - JSON is easier to parse and debug than XML
2. **Modern** - JSON is the preferred protocol for new INDIGO clients
3. **Sufficient** - All client features can be tested with JSON
4. **Reuse** - Leverages existing [`JsonProtocolParser`](../../rs/src/protocol_json.rs) and [`JsonProtocolSerializer`](../../rs/src/protocol_json.rs)

### Why Async/Await?

The server uses tokio's async runtime for:

1. **Concurrency** - Handle multiple connections efficiently
2. **Non-blocking I/O** - Better resource utilization
3. **Consistency** - Matches client implementation
4. **Scalability** - Support many concurrent test connections

### State Management

The server uses a shared state model with `Arc<RwLock<T>>`:

**Benefits:**

- Multiple readers can access state concurrently
- Single writer ensures consistency
- No data races or corruption
- Simple mental model

**Alternatives Considered:**

- Actor model (too complex for testing)
- Message passing (more overhead)
- Mutex (less concurrent reads)

## Performance Characteristics

### Expected Performance

- **Connections**: Support 10+ concurrent connections
- **Throughput**: Handle 100+ messages/second per connection
- **Latency**: <10ms message processing time
- **Memory**: <50MB for typical test scenarios

### Optimization Strategies

1. **Lock Granularity** - Use `RwLock` for read-heavy operations
2. **Channel Sizing** - Unbounded channels for property updates
3. **Message Batching** - Send multiple property definitions together
4. **Lazy Updates** - Only update properties when streaming is enabled

## Testing Strategy

### Unit Tests

Each module includes unit tests for core functionality:

- [`device.rs`](../../tests/mock_server/device.rs) - Device registry operations
- [`property.rs`](../../tests/mock_server/property.rs) - Property updates
- [`subscription.rs`](../../tests/mock_server/subscription.rs) - Subscription filtering
- [`builder.rs`](../../tests/mock_server/builder.rs) - Builder configuration

### Integration Tests

**Location**: [`tests/mock_server_test.rs`](../../tests/mock_server_test.rs)

Tests cover:

- Server startup and shutdown
- Multiple device configurations
- TCP connection handling
- Protocol message exchange
- Property updates and notifications
- Server statistics tracking

### Test Coverage

Run tests with coverage:

```bash
cargo test --test mock_server_test
```

## Limitations

### Current Limitations

1. **JSON Protocol Only** - No XML protocol support
2. **No BLOB Data** - Only URL references (sufficient for JSON protocol)
3. **Simple Simulation** - Basic property updates, no complex device behavior
4. **No Authentication** - Not required for testing
5. **No Encryption** - Not required for testing

### Not Implemented

- XML protocol support
- BLOB data transfer
- Device discovery (not part of INDIGO protocol)
- Complex device simulations (e.g., actual exposure timing)
- Protocol version negotiation (always accepts 512)

## Future Enhancements

Potential improvements for future iterations:

1. **XML Protocol Support** - Add XML parser/serializer for completeness
2. **Advanced Simulations** - More realistic device behavior (e.g., exposure timing)
3. **Recording/Playback** - Record real server interactions for replay
4. **Configuration Files** - Load device configurations from JSON/TOML
5. **Hot Reload** - Update devices/properties without restart
6. **Network Simulation** - Simulate latency, packet loss, etc.
7. **Performance Metrics** - Built-in benchmarking and profiling
8. **Multi-Server** - Simulate multiple INDIGO servers

## Troubleshooting

### Common Issues

**Server won't start:**

- Check if port is already in use
- Verify bind address is valid
- Check for firewall restrictions

**No property definitions received:**

- Verify client sends `getProperties` with `version: 512`
- Check server logs for parse errors
- Ensure devices were added to server

**Property updates not received:**

- Verify client is subscribed (sent `getProperties`)
- Check subscription filters match device/property names
- Ensure property exists in device

**Connection hangs:**

- Verify messages are newline-terminated
- Check for deadlocks in test code
- Use timeout for read operations

### Debug Mode

Enable verbose logging:

```rust
let server = MockServerBuilder::new()
    .verbose()
    .build()
    .await
    .unwrap();
```

This will print:

- Connection events
- Message parsing
- Property updates
- Error messages

## References

- [INDIGO Protocol Documentation](../protocols/json-protocol.md)
- [Client Strategies Architecture](./client-strategies.md)
- [Mock Server Architecture Plan](../../plans/mock-indigo-server-architecture.md)
- [Protocol Implementation](../../rs/src/protocol_json.rs)
- [INDIGO Protocol Specification](../../sys/externals/indigo/indigo_docs/PROTOCOLS.md)

## Conclusion

The mock INDIGO server provides a lightweight, pure Rust implementation of the INDIGO protocol specifically designed for testing. It offers:

✅ **Zero Dependencies** - No FFI or C library requirements
✅ **Fast Execution** - Async I/O and efficient state management
✅ **Easy Configuration** - Fluent builder API with preset devices
✅ **Full Protocol Support** - JSON protocol version 512
✅ **Observable State** - Query properties and statistics for assertions
✅ **Concurrent Connections** - Handle multiple test clients
✅ **Property Streaming** - Simulate real-time device updates

The mock server enables comprehensive integration testing of the libindigo-rs client without requiring a full INDIGO server installation or hardware devices.
