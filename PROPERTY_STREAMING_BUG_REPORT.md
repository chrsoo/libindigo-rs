# Property Streaming Bug Report & Fix Proposal

## Reporter
Mission Control CLI team (mission-control workspace)

## Environment
- **Consumer**: mission-control CLI (`/Users/csoop/src/astronomy/mission-control/cli`)
- **Library**: libindigo-rs (local path: `/Users/csoop/src/astronomy/libindigo/rs`)
- **Base library**: libindigo (local path: `/Users/csoop/src/astronomy/libindigo`)
- **INDIGO Server**: xolotl.local.:7624 (XML protocol only)
- **Test binary**: `mission-control/cli/src/bin/test_stream.rs` - minimal reproduction

## Summary

Property streaming via `subscribe_properties()` and `property_receiver()` does not deliver any properties to consumers despite successful TCP connection. Two bugs have been identified in `libindigo-rs/src/client.rs`.

## Bug 1: Transport Stealing in Background Receiver Task

### Location
`rs/src/client.rs` — `start_receiver_task()` function (approximately lines 225-297)

### Description
The background receiver task uses `state.transport.take()` to acquire the transport for reading, which **removes** the transport from the shared `ClientState`. While the transport is returned after each `receive_message()` call, there is a window where the transport is `None` in the state.

### Impact
- Any call to `enumerate_properties()` (or any method that needs the transport) during this window fails with `"Transport not available"`
- Since the background task runs in a tight loop, the transport is almost always unavailable to other callers
- The `connect()` method calls `enumerate_properties(None)` at line ~828 AFTER starting the background task, creating an immediate race condition

### Reproduction
```rust
let mut strategy = RsClientStrategy::new();
strategy.connect("xolotl.local.:7624").await.unwrap(); // succeeds
strategy.enumerate_properties(None).await; // FAILS: "Transport not available"
```

### Proposed Fix

**Option A (Recommended): Split transport into read/write halves**

Instead of sharing a single transport via `Option<Transport>`, split the TCP stream into read and write halves. The background task owns the read half exclusively, while the write half stays in the state for sending messages:

```rust
struct ClientState {
    write_transport: Option<WriteTransport>,  // For sending (enumerate_properties, etc.)
    // read_transport is owned by background task directly
    property_tx: Option<mpsc::UnboundedSender<Property>>,
    property_rx: Option<mpsc::UnboundedReceiver<Property>>,
    property_subscribers: Vec<mpsc::UnboundedSender<Property>>,
    background_task: Option<JoinHandle<()>>,
    connected: bool,
}
```

The `start_receiver_task()` function would receive the read half directly instead of reaching into the state:

```rust
async fn start_receiver_task(
    read_transport: ReadTransport,
    state: Arc<Mutex<ClientState>>
) -> Result<()> {
    let handle = tokio::spawn(async move {
        let mut transport = read_transport;
        loop {
            match transport.receive_message().await {
                Ok(msg) => {
                    if let Some(property) = Self::convert_to_property(msg) {
                        let mut state = state.lock().await;
                        // Broadcast to subscribers
                        state.property_subscribers.retain(|sub| {
                            sub.send(property.clone()).is_ok()
                        });
                        // Also send to legacy channel
                        if let Some(ref tx) = state.property_tx {
                            let _ = tx.send(property);
                        }
                    }
                }
                Err(e) => {
                    // Handle error
                    break;
                }
            }
        }
    });
    // Store handle
    let mut state_lock = state.lock().await;
    state_lock.background_task = Some(handle);
    Ok(())
}
```

**Option B (Simpler): Use Arc<Mutex<Transport>> shared by both**

Wrap the transport in its own `Arc<Mutex<>>`:

```rust
struct ClientState {
    transport: Arc<Mutex<Option<Transport>>>,  // Shared reference
    // ...
}
```

The background task locks the transport only for the duration of `receive_message()`, and other callers can acquire the lock between reads:

```rust
// In background task:
let transport_lock = transport_arc.lock().await;
if let Some(ref mut t) = *transport_lock {
    match t.receive_message().await {
        // ...
    }
}
drop(transport_lock); // Explicitly release before processing
```

**Option C (Quick fix): Retry with backoff in enumerate_properties**

Add retry logic to `enumerate_properties()` that waits for the transport to become available:

```rust
pub async fn enumerate_properties(&mut self, device: Option<&str>) -> Result<()> {
    let mut retries = 10;
    loop {
        let mut state = self.state.lock().await;
        if let Some(ref mut transport) = state.transport {
            // Send getProperties message
            return transport.send_message(/* ... */).await;
        }
        drop(state);
        retries -= 1;
        if retries == 0 {
            return Err(Error::InvalidState("Transport not available".into()));
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
```

## Bug 2: Background Task May Not Receive Properties

### Location
`rs/src/client.rs` — `start_receiver_task()` and `connect()` interaction

### Description
Even when the transport stealing issue is worked around, the background receiver task does not appear to deliver properties to subscribers. The `property_rx.recv()` always times out with 0 properties received.

### Possible Causes (needs investigation in the library)

1. **Protocol mismatch**: `RsClientStrategy::new()` creates a `ProtocolNegotiator` that prefers JSON. The server at xolotl.local.:7624 only speaks XML. The negotiation may not properly fall back to XML.

2. **`convert_to_property()` returns None**: If the protocol message type isn't handled, properties are silently dropped. Add logging to this function to verify.

3. **Background task exits early**: If `receive_message()` returns an error on the first attempt, the task breaks out of the loop and stops.

### Proposed Investigation
Add `tracing` or `log` output to:
- `start_receiver_task()` — log when entering the loop, when receiving a message, when `convert_to_property` returns None
- `convert_to_property()` — log what message types are being converted
- The protocol negotiation — log which protocol was selected

### Proposed Fix for Protocol Negotiation
In `connect()`, after protocol negotiation completes, verify the selected protocol works by checking the first received message. If the server responds with XML to a JSON request, detect this and switch to XML mode.

## Test Case

A minimal reproduction test exists at:
`/Users/csoop/src/astronomy/mission-control/cli/src/bin/test_stream.rs`

To run it (from the mission-control workspace):
```bash
cd cli && cargo run --bin test-stream 2>&1
```

Expected output with bugs:
```
Connecting...
Connected!
Got property receiver, waiting...
Timeout 2s

=== Testing subscribe_properties ===
Calling enumerate_properties...
enumerate_properties FAILED: InvalidState("Transport not available")
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

## Server Verification

The INDIGO server at `xolotl.local.:7624` is working correctly. Raw XML communication succeeds:
```bash
echo '<getProperties version="2.0"/>' | nc xolotl.local. 7624
```
Returns valid INDIGO XML property definitions including `defTextVector` for Server device's INFO property.

## Priority
**High** — This blocks the mission-control CLI from displaying real server information. The consumer code is ready and waiting for the library fix.

## Files to Modify
1. `rs/src/client.rs` — Fix transport ownership in `start_receiver_task()` and `connect()`
2. `rs/src/client.rs` — Add logging to `convert_to_property()` and the background task
3. `rs/src/protocol_negotiation.rs` — Verify XML fallback works correctly
4. `rs/src/transport.rs` — Consider splitting into read/write halves
