# INDIGO JSON Protocol

## Overview

The JSON protocol provides the same features as XML protocol version 2.0 but in JSON format, offering better integration with modern web applications and JavaScript clients.

**Protocol Version**: 512 (equivalent to XML 2.0)

## Key Differences from XML

| Feature | XML Protocol | JSON Protocol |
|---------|--------------|---------------|
| Version number | "2.0" (string) | 512 (number) |
| Switch values | "On"/"Off" (strings) | true/false (booleans) |
| Number values | Strings | JSON numbers |
| BLOB data | BASE64 inline or URL | URL only |
| Message structure | XML tags/attributes | JSON objects |
| Array representation | Repeated elements | JSON arrays with `items` key |

## BLOB Handling

**Important**: JSON protocol only supports URL-referenced BLOBs. No inline BASE64 data support.

BLOBs are referenced by URL paths:

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

## Message Examples

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

### deleteProperty

```json
{
  "deleteProperty": {
    "device": "Mount IEQ (guider)"
  }
}
```

## API Usage

### Parsing JSON Messages

```rust
use libindigo::strategies::rs::protocol_json::JsonProtocolParser;

let json = r#"{"getProperties": {"version": 512, "device": "Server"}}"#;
let message = JsonProtocolParser::parse_message(json)?;

match message {
    ProtocolMessage::GetProperties(gp) => {
        println!("Device: {:?}", gp.device);
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

## Supported Message Types

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

## Implementation

**Location**: `src/strategies/rs/protocol_json.rs`

**Components**:

- `JsonProtocolParser` - Parses JSON strings into `ProtocolMessage` enums
- `JsonProtocolSerializer` - Serializes `ProtocolMessage` enums to JSON strings
- Helper functions for switch state boolean conversion

**Shared Types**: Reuses all message type structs from `protocol.rs` for consistency between XML and JSON protocols.

## Testing

Comprehensive unit tests are included:

- Parse/serialize roundtrip tests
- Boolean switch value conversion
- All message types covered

Run tests:

```bash
cargo test --features rs protocol_json --lib
```

## Protocol Compliance

This implementation follows the INDIGO JSON protocol specification:

- ✅ Version 512 for JSON protocol
- ✅ URL-only BLOB references
- ✅ Boolean switch values
- ✅ JSON numeric types
- ✅ Items array structure
- ✅ All message types supported
- ✅ Compatible with INDIGO server JSON protocol

## References

- [INDIGO PROTOCOLS.md](../../sys/externals/indigo/indigo_docs/PROTOCOLS.md) - Official specification
- [indigo_json.c](../../sys/externals/indigo/indigo_libs/indigo_json.c) - C implementation reference
- [Client Strategies](../architecture/client-strategies.md) - Strategy architecture
