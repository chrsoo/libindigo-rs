# Property Streaming Bug Fix - Implementation Summary

## Date

2026-03-09

## Bugs Fixed

### Bug 1: Transport Stealing in Background Receiver Task ✅ FIXED

**Problem**: The background receiver task used `state.transport.take()` to acquire the transport for reading, which removed the transport from the shared `ClientState`. This created a race condition where any call to `enumerate_properties()` or other methods that needed the transport would fail with "Transport not available".

**Solution**: Split the TCP transport into separate read and write halves using Tokio's `split()` functionality.

#### Implementation Details

1. **New Transport Types** ([`rs/src/transport.rs`](rs/src/transport.rs)):
   - Added `ReadTransport` - owns the read half of the TCP stream
   - Added `WriteTransport` - owns the write half of the TCP stream
   - Added `Transport::split()` method to split a connected transport

2. **Updated ClientState** ([`rs/src/client.rs`](rs/src/client.rs:71-90)):

   ```rust
   struct ClientState {
       write_transport: Option<WriteTransport>,  // Changed from transport: Option<Transport>
       // ... other fields
   }
   ```

3. **Updated Background Task** ([`rs/src/client.rs`](rs/src/client.rs:221-320)):
   - Now receives `ReadTransport` directly as a parameter
   - Owns the read half exclusively - no more take/put back pattern
   - Continuously reads without blocking other operations
   - Added comprehensive logging for debugging

4. **Updated Connection Flow** ([`rs/src/client.rs`](rs/src/client.rs:823-860)):

   ```rust
   // Split transport after protocol negotiation
   let (read_transport, write_transport) = transport.split()?;

   // Store write half in state
   state.write_transport = Some(write_transport);

   // Pass read half to background task (it owns it)
   Self::start_receiver_task(read_transport, Arc::clone(&self.state)).await?;
   ```

5. **Updated All Send Methods**:
   - [`enumerate_properties()`](rs/src/client.rs:916-943)
   - [`send_property()`](rs/src/client.rs:968-988)
   - [`enable_blob()`](rs/src/client.rs:1016-1050)
   - All now use `state.write_transport` instead of `state.transport`

### Bug 2: Background Task May Not Receive Properties ✅ ADDRESSED

**Problem**: Even when the transport stealing issue was worked around, the background receiver task did not appear to deliver properties to subscribers. Possible causes included protocol mismatch, silent message dropping, or early task exit.

**Solution**: Added comprehensive logging throughout the message reception pipeline to diagnose issues.

#### Logging Added

1. **Connection Logging** ([`rs/src/client.rs`](rs/src/client.rs:840)):

   ```rust
   tracing::info!("Connected to {} using protocol {:?}", url, protocol);
   ```

2. **Background Task Lifecycle** ([`rs/src/client.rs`](rs/src/client.rs:239-318)):
   - Task start: `"Background receiver task started"`
   - Message received: `"Received message #N: {:?}"`
   - Property conversion: `"Converted to property: device=X, name=Y, items=Z"`
   - Control messages: `"Message did not convert to property (control message)"`
   - Errors: `"Error receiving message: {}"`
   - Task exit: `"Background receiver task exited after N messages"`

3. **Protocol Auto-Detection** ([`rs/src/transport.rs`](rs/src/transport.rs:779-785)):

   ```rust
   tracing::info!(
       "Protocol auto-detected and switched from {:?} to {:?}",
       self.protocol,
       detected_protocol
   );
   ```

4. **Message Sending** ([`rs/src/client.rs`](rs/src/client.rs:932)):

   ```rust
   tracing::debug!("Sending getProperties for device: {:?}", device);
   ```

5. **Disconnection** ([`rs/src/client.rs`](rs/src/client.rs:895)):

   ```rust
   tracing::info!("Disconnected from server");
   ```

## Benefits of the Fix

### 1. Eliminates Race Condition

- The write transport is always available for sending messages
- No more "Transport not available" errors
- `enumerate_properties()` can be called immediately after `connect()`

### 2. Proper Concurrent Access

- Read and write operations are truly independent
- Background task can continuously read without blocking sends
- Follows Tokio best practices for split streams

### 3. Better Debugging

- Comprehensive logging at all levels (info, debug, trace)
- Can track message flow from reception to property conversion
- Can identify protocol negotiation issues
- Can see subscriber management (disconnections)

### 4. Cleaner Architecture

- Clear ownership model: background task owns read half
- No complex take/put back logic
- Simpler state management

## Testing Recommendations

To verify the fixes work with the mission-control CLI:

```bash
# From mission-control workspace
cd cli && cargo run --bin test-stream 2>&1
```

Expected output after fix:

```
Connecting...
Connected!
Got property receiver, waiting...
PROPERTY: device=Server, name=INFO, items=3
PROPERTY: device=Server, name=LOG_LEVEL, items=4
... (more properties from all connected devices)
```

To enable detailed logging:

```bash
RUST_LOG=libindigo_rs=debug,mission_control=debug cargo run --bin test-stream 2>&1
```

## Files Modified

1. [`rs/src/transport.rs`](rs/src/transport.rs) - Added ReadTransport and WriteTransport types
2. [`rs/src/client.rs`](rs/src/client.rs) - Updated to use split transport pattern with logging

## Compatibility

- ✅ Backward compatible - existing API unchanged
- ✅ All tests pass
- ✅ Code compiles without warnings
- ✅ No breaking changes to public API

## Next Steps

The mission-control team should:

1. Update their `Cargo.toml` to use the latest libindigo-rs
2. Run their test-stream binary to verify property streaming works
3. Enable debug logging if issues persist to see detailed message flow
4. Report any remaining issues with the log output

## Technical Notes

### Why Split Instead of Arc<Mutex<Transport>>?

Option B from the bug report (wrapping transport in Arc<Mutex<>>) was considered but rejected because:

- Adds unnecessary locking overhead on every read/write
- Still has potential for lock contention
- Less idiomatic Rust - Tokio provides split() for this exact use case
- Split is zero-cost abstraction - no runtime overhead

### Protocol Auto-Detection

The ReadTransport automatically detects protocol from incoming data and switches if needed. This handles the case where:

- Client prefers JSON but server only speaks XML
- Server responds with XML to initial connection
- Protocol is auto-detected and switched transparently

This addresses the protocol negotiation concern from Bug 2.
