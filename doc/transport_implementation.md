# TCP Transport Layer Implementation

## Overview

The TCP transport layer (`src/strategies/rs/transport.rs`) provides robust, asynchronous TCP communication with INDIGO servers. It handles connection management, XML message framing, buffering, and error handling.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Transport Layer                       │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌──────────────┐    ┌──────────────┐    ┌───────────┐ │
│  │  Connection  │───▶│   Message    │───▶│  Message  │ │
│  │  Management  │    │   Framing    │    │  Parsing  │ │
│  └──────────────┘    └──────────────┘    └───────────┘ │
│         │                    │                    │      │
│         ▼                    ▼                    ▼      │
│  ┌──────────────────────────────────────────────────┐  │
│  │           TcpStream (tokio async I/O)            │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Key Features

### 1. Connection Management

- **Async Connection**: Uses `tokio::net::TcpStream` for non-blocking I/O
- **URL Parsing**: Supports `host:port` and `host` (default port 7624) formats
- **Timeouts**: Configurable connection and read timeouts
- **State Tracking**: Maintains connection state (Connected/Disconnected)
- **Graceful Shutdown**: Properly flushes and closes connections

### 2. XML Message Framing

INDIGO protocol sends XML messages sequentially over TCP without explicit delimiters. The transport layer implements intelligent message boundary detection:

```rust
// Example: Multiple messages in stream
<getProperties version="1.7"/><defTextVector device="CCD">...</defTextVector>
                               ↑
                        Message boundary detected here
```

**Framing Algorithm**:

- Tracks XML tag depth (opening/closing tags)
- Handles self-closing tags (`<tag/>`)
- Respects quoted strings in attributes
- Detects complete messages when depth returns to 0
- Buffers partial messages across TCP reads

### 3. Buffering Strategy

- **Read Buffer**: Accumulates incoming data until complete messages are formed
- **Dynamic Growth**: Buffer grows as needed (up to 10MB limit)
- **Efficient Parsing**: Removes parsed messages from buffer to free memory
- **Partial Message Handling**: Correctly handles messages split across multiple TCP reads

### 4. Error Handling

Comprehensive error handling for all failure modes:

- **Connection Errors**: Failed connections, DNS resolution issues
- **Timeout Errors**: Connection and read timeouts
- **Protocol Errors**: Malformed XML, buffer overflow
- **I/O Errors**: Network failures, connection drops
- **State Errors**: Operations on disconnected transport

## API Reference

### Core Types

#### `Transport`

Main transport struct managing TCP connection and message I/O.

```rust
pub struct Transport {
    stream: Option<TcpStream>,
    read_buffer: Vec<u8>,
    state: ConnectionState,
    connect_timeout: Duration,
    read_timeout: Duration,
}
```

### Public Methods

#### Connection Management

```rust
// Create new transport (not connected)
pub fn new() -> Self

// Connect to server with default timeouts
pub async fn connect(url: &str) -> Result<Self>

// Connect with custom timeouts
pub async fn connect_with_timeout(
    url: &str,
    connect_timeout: Duration,
    read_timeout: Duration,
) -> Result<Self>

// Disconnect gracefully
pub async fn disconnect(&mut self) -> Result<()>

// Check connection status
pub fn is_connected(&self) -> bool
```

#### Message I/O

```rust
// Send a protocol message
pub async fn send_message(&mut self, message: &ProtocolMessage) -> Result<()>

// Receive a single message
pub async fn receive_message(&mut self) -> Result<ProtocolMessage>

// Get a stream of incoming messages
pub fn receive_stream(&mut self) -> MessageStream<'_>
```

### Constants

```rust
pub const DEFAULT_INDIGO_PORT: u16 = 7624;
const DEFAULT_CONNECT_TIMEOUT: u64 = 10;  // seconds
const DEFAULT_READ_TIMEOUT: u64 = 30;     // seconds
const INITIAL_BUFFER_SIZE: usize = 8192;
const MAX_BUFFER_SIZE: usize = 10 * 1024 * 1024;  // 10 MB
```

## Usage Examples

### Basic Connection

```rust
use libindigo::strategies::rs::transport::Transport;

// Connect to local INDIGO server
let mut transport = Transport::connect("localhost:7624").await?;

// Or use default port
let mut transport = Transport::connect("localhost").await?;

// Check connection
assert!(transport.is_connected());

// Disconnect when done
transport.disconnect().await?;
```

### Sending Messages

```rust
use libindigo::strategies::rs::protocol::{GetProperties, ProtocolMessage};

let mut transport = Transport::connect("localhost:7624").await?;

// Create a message
let msg = ProtocolMessage::GetProperties(GetProperties {
    version: Some("1.7".to_string()),
    device: None,
    name: None,
});

// Send it
transport.send_message(&msg).await?;
```

### Receiving Messages

```rust
// Receive single message
let msg = transport.receive_message().await?;
match msg {
    ProtocolMessage::DefTextVector(v) => {
        println!("Received text vector: {}", v.attrs.name);
    }
    _ => {}
}

// Or use streaming API
let mut stream = transport.receive_stream();
while let Some(result) = stream.next().await {
    match result {
        Ok(msg) => {
            // Process message
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            break;
        }
    }
}
```

### Custom Timeouts

```rust
use std::time::Duration;

let transport = Transport::connect_with_timeout(
    "192.168.1.100:7624",
    Duration::from_secs(5),   // connection timeout
    Duration::from_secs(60),  // read timeout
).await?;
```

## Implementation Details

### Message Boundary Detection

The `find_message_boundary()` method implements a state machine to track XML structure:

```rust
fn find_message_boundary(&self) -> Result<Option<usize>> {
    let mut depth = 0;
    let mut in_tag = false;
    let mut in_string = false;
    let mut is_closing_tag = false;
    let mut is_self_closing = false;

    // Scan buffer byte by byte
    for (i, &byte) in self.read_buffer.iter().enumerate() {
        match byte {
            b'<' => { /* opening tag */ }
            b'>' => {
                // Update depth
                if is_closing_tag { depth -= 1; }
                else if !is_self_closing { depth += 1; }

                // Complete message when depth returns to 0
                if depth == 0 { return Ok(Some(i)); }
            }
            // ... handle quotes, self-closing tags, etc.
        }
    }

    Ok(None)  // No complete message yet
}
```

### Buffer Management

The transport maintains a growing buffer that:

1. Accumulates data from TCP reads
2. Parses complete messages
3. Removes parsed data to free memory
4. Prevents unbounded growth (10MB limit)

```rust
async fn read_more_data(&mut self) -> Result<()> {
    // Check buffer size limit
    if self.read_buffer.len() >= MAX_BUFFER_SIZE {
        return Err(IndigoError::ProtocolError(
            "Read buffer exceeded maximum size".to_string()
        ));
    }

    // Read into temporary buffer
    let mut temp_buffer = vec![0u8; 4096];
    let bytes_read = timeout(
        self.read_timeout,
        self.stream.read(&mut temp_buffer)
    ).await??;

    // Append to main buffer
    self.read_buffer.extend_from_slice(&temp_buffer[..bytes_read]);

    Ok(())
}
```

## Testing

### Unit Tests

The module includes comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_url() { /* ... */ }

    #[test]
    fn test_find_message_boundary() { /* ... */ }

    #[tokio::test]
    async fn test_send_message_not_connected() { /* ... */ }

    // ... more tests
}
```

Run tests with:

```bash
cargo test --lib transport
```

### Integration Tests

Integration tests are in `tests/transport_integration.rs`:

```bash
cargo test --test transport_integration --features rs
```

## Performance Considerations

### Memory Usage

- Initial buffer: 8 KB
- Grows dynamically as needed
- Maximum buffer: 10 MB (prevents DoS)
- Parsed messages removed immediately

### Network Efficiency

- Uses buffered I/O (4KB reads)
- Minimizes system calls
- Async I/O prevents blocking
- Configurable timeouts prevent hangs

### CPU Usage

- Efficient XML boundary detection (single pass)
- Zero-copy where possible
- Minimal allocations in hot path

## Error Recovery

The transport layer handles various error conditions:

1. **Connection Failures**: Return `ConnectionError` with details
2. **Timeouts**: Return `Timeout` error after configured duration
3. **Malformed XML**: Return `ParseError` from protocol parser
4. **Connection Drops**: Detect EOF and update state
5. **Buffer Overflow**: Prevent unbounded growth

## Future Enhancements

Potential improvements for future versions:

1. **Reconnection Logic**: Automatic reconnection on connection loss
2. **Connection Pooling**: Reuse connections for multiple clients
3. **Compression**: Optional gzip compression for large messages
4. **TLS Support**: Secure connections via tokio-rustls
5. **Metrics**: Connection statistics and performance monitoring
6. **Backpressure**: Flow control for high-throughput scenarios

## Integration with Protocol Layer

The transport layer integrates seamlessly with the protocol layer:

```rust
// Transport uses protocol serializer
let xml_bytes = ProtocolSerializer::serialize(message)?;
stream.write_all(&xml_bytes).await?;

// Transport uses protocol parser
let message = ProtocolParser::parse_message(&message_bytes)?;
```

## Thread Safety

The `Transport` struct is **not** `Send` or `Sync` by default because:

- It contains a `TcpStream` which is not `Sync`
- It's designed for single-threaded async use

For multi-threaded scenarios, wrap in `Arc<Mutex<Transport>>` or use message passing.

## Conclusion

The TCP transport layer provides a robust, efficient foundation for INDIGO protocol communication. It handles the complexities of TCP streaming, XML message framing, and error conditions, presenting a clean async API to higher layers.
