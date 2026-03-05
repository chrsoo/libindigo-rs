# INDIGO JSON Protocol Implementation

## Overview

This document describes the JSON protocol implementation for INDIGO in the libindigo-rs library. The JSON protocol provides the same features as XML protocol version 2.0 but in JSON format, offering better integration with modern web applications and JavaScript clients.

## Implementation Location

- **Main Module**: [`src/strategies/rs/protocol_json.rs`](src/strategies/rs/protocol_json.rs)
- **Dependencies**: Added `serde_json` to [`Cargo.toml`](Cargo.toml)
- **Module Export**: Updated [`src/strategies/rs/mod.rs`](src/strategies/rs/mod.rs)

## Key Features

### 1. Protocol Version

- JSON protocol uses version number **512** (equivalent to XML 2.0)
- Automatically included in serialized messages

### 2. BLOB Handling

- **JSON protocol only supports URL-referenced BLOBs**
- No inline BASE64 data support (unlike XML protocol)
- BLOBs are referenced by URL paths (e.g., `/blob/0x10381d798.fits`)

### 3. Data Type Mapping

| XML Format | JSON Format | Notes |
|------------|-------------|-------|
| Switch states: "On"/"Off" | `true`/`false` | Boolean values |
| Numbers as strings | JSON numbers | Native numeric types |
| XML tags | JSON object keys | Cleaner structure |
| XML attributes | JSON properties | Direct mapping |
| Nested elements | `items` array | Consistent structure |

## Message Types Supported

All INDIGO protocol message types are supported:

### Definition Messages (Server → Client)

- `defTextVector` - Text property definitions
- `defNumberVector` - Number property definitions
- `defSwitchVector` - Switch property definitions
- `defLightVector` - Light property definitions
- `defBLOBVector` - BLOB property definitions

### Set Messages (Server → Client)

- `setTextVector` - Text property updates
- `setNumberVector` - Number property updates
- `setSwitchVector` - Switch property updates
- `setLightVector` - Light property updates
- `setBLOBVector` - BLOB property updates

### New Messages (Client → Server)

- `newTextVector` - Text property commands
- `newNumberVector` - Number property commands
- `newSwitchVector` - Switch property commands
- `newBLOBVector` - BLOB property commands

### Control Messages

- `getProperties` - Request property definitions
- `enableBLOB` - Control BLOB transfer
- `message` - Status messages
- `deleteProperty` - Property deletion

## API Usage

### Parsing JSON Messages

```rust
use libindigo::strategies::rs::protocol_json::JsonProtocolParser;

let json = r#"{"getProperties": {"version": 512, "device": "Server", "name": "LOAD"}}"#;
let message = JsonProtocolParser::parse_message(json)?;

match message {
    ProtocolMessage::GetProperties(gp) => {
        println!("Device: {:?}", gp.device);
        println!("Property: {:?}", gp.name);
    }
    _ => {}
}
```

### Serializing Messages

```rust
use libindigo::strategies::rs::protocol_json::JsonProtocolSerializer;
use libindigo::strategies::rs::protocol::{GetProperties, ProtocolMessage};

let msg = ProtocolMessage::GetProperties(GetProperties {
    version: Some("2.0".to_string()),
    device: Some("Server".to_string()),
    name: Some("LOAD".to_string()),
});

let json = JsonProtocolSerializer::serialize(&msg)?;
// Output: {"getProperties":{"version":512,"device":"Server","name":"LOAD"}}
```

## JSON Message Examples

### getProperties

```json
{
  "getProperties": {
    "version": 512,
    "client": "My Client",
    "device": "Server",
    "name": "LOAD"
  }
}
```

### defTextVector

```json
{
  "defTextVector": {
    "version": 512,
    "device": "Server",
    "name": "LOAD",
    "group": "Main",
    "label": "Load driver",
    "perm": "rw",
    "state": "Idle",
    "items": [
      {
        "name": "DRIVER",
        "label": "Load driver",
        "value": ""
      }
    ]
  }
}
```

### defSwitchVector

```json
{
  "defSwitchVector": {
    "version": 512,
    "device": "Server",
    "name": "RESTART",
    "group": "Main",
    "label": "Restart",
    "perm": "rw",
    "state": "Idle",
    "rule": "AnyOfMany",
    "hints": "order: 10; widget: button",
    "items": [
      {
        "name": "RESTART",
        "label": "Restart server",
        "value": false
      }
    ]
  }
}
```

### setSwitchVector

```json
{
  "setSwitchVector": {
    "device": "CCD Imager Simulator",
    "name": "CONNECTION",
    "state": "Ok",
    "items": [
      {
        "name": "CONNECTED",
        "value": true
      },
      {
        "name": "DISCONNECTED",
        "value": false
      }
    ]
  }
}
```

### newNumberVector

```json
{
  "newNumberVector": {
    "device": "CCD Imager Simulator",
    "name": "CCD_EXPOSURE",
    "token": "FA0012",
    "items": [
      {
        "name": "EXPOSURE",
        "value": 1
      }
    ]
  }
}
```

### setBLOBVector (URL-referenced)

```json
{
  "setBLOBVector": {
    "device": "CCD Imager Simulator",
    "name": "CCD_IMAGE",
    "state": "Ok",
    "items": [
      {
        "name": "IMAGE",
        "value": "/blob/0x10381d798.fits"
      }
    ]
  }
}
```

### deleteProperty

```json
{
  "deleteProperty": {
    "device": "Mount IEQ (guider)"
  }
}
```

## Implementation Details

### Code Structure

1. **JsonProtocolParser** - Parses JSON strings into `ProtocolMessage` enums
   - Uses `serde_json` for JSON parsing
   - Validates message structure
   - Converts JSON booleans to `SwitchState` enum
   - Handles optional fields correctly

2. **JsonProtocolSerializer** - Serializes `ProtocolMessage` enums to JSON strings
   - Produces compact JSON (no pretty printing)
   - Automatically adds version number (512)
   - Converts `SwitchState` enum to JSON booleans
   - Handles optional fields correctly

3. **Helper Functions**
   - `SwitchState::to_bool()` - Convert switch state to boolean
   - `SwitchState::from_bool()` - Convert boolean to switch state
   - Vector attribute helpers for consistent serialization

### Shared Types

The JSON protocol implementation reuses all message type structs from [`protocol.rs`](src/strategies/rs/protocol.rs):

- `ProtocolMessage` enum
- All vector definition structs (`DefTextVector`, `DefNumberVector`, etc.)
- All vector update structs (`SetTextVector`, `SetNumberVector`, etc.)
- All element structs (`DefText`, `OneNumber`, etc.)
- Enums: `PropertyState`, `PropertyPerm`, `SwitchState`, `SwitchRule`, `BLOBEnable`

This ensures consistency between XML and JSON protocols.

## Testing

Comprehensive unit tests are included in [`protocol_json.rs`](src/strategies/rs/protocol_json.rs):

- ✅ Parse `getProperties` message
- ✅ Serialize `getProperties` message
- ✅ Parse `defTextVector` message
- ✅ Parse `setSwitchVector` message with boolean values
- ✅ Parse `newNumberVector` message
- ✅ Roundtrip test for `defSwitchVector` (parse → serialize → parse)
- ✅ Switch state boolean conversion tests

Run tests with:

```bash
cargo test --features rs protocol_json --lib
```

## Differences from XML Protocol

| Feature | XML Protocol | JSON Protocol |
|---------|--------------|---------------|
| Version number | "2.0" (string) | 512 (number) |
| Switch values | "On"/"Off" (strings) | true/false (booleans) |
| Number values | Strings | JSON numbers |
| BLOB data | BASE64 inline or URL | URL only |
| Message structure | XML tags/attributes | JSON objects |
| Array representation | Repeated elements | JSON arrays with `items` key |

## Integration with Pure Rust Strategy

The JSON protocol can be used alongside the XML protocol in the pure Rust client strategy:

1. **Protocol Negotiation**: Client can request JSON protocol during connection
2. **Transport Layer**: Same TCP transport works for both protocols
3. **Message Routing**: Protocol type detected from message format
4. **Seamless Switching**: Can switch between protocols during session

## Future Enhancements

Potential improvements for future versions:

1. **WebSocket Support**: Add WebSocket transport for JSON protocol
2. **Streaming Parser**: Support for partial JSON messages in streaming scenarios
3. **Schema Validation**: JSON schema for message validation
4. **Pretty Printing**: Optional pretty-printed JSON for debugging
5. **Compression**: Optional JSON compression for large messages

## References

- [INDIGO PROTOCOLS.md](sys/externals/indigo/indigo_docs/PROTOCOLS.md) - Official protocol specification
- [indigo_json.c](sys/externals/indigo/indigo_libs/indigo_json.c) - C implementation reference
- [serde_json documentation](https://docs.rs/serde_json/) - JSON library used

## Compliance

This implementation follows the INDIGO JSON protocol specification as documented in PROTOCOLS.md:

- ✅ Version 512 for JSON protocol
- ✅ URL-only BLOB references
- ✅ Boolean switch values
- ✅ JSON numeric types
- ✅ Items array structure
- ✅ All message types supported
- ✅ Compatible with INDIGO server JSON protocol

## License

This implementation is part of libindigo-rs and follows the same MIT license.
