# Rust Client Strategy Tests

This document describes the test suite for the Rust INDIGO client implementation.

## Test Files

### 1. Protocol Compliance Tests (`rs_protocol_compliance.rs`)

Comprehensive tests for the INDIGO XML protocol parser and serializer.

#### Test Coverage

- **GetProperties Message Tests** (5 tests)
  - Minimal message parsing
  - With device filter
  - With device and name filter
  - Serialization
  - Roundtrip conversion

- **DefTextVector Tests** (5 tests)
  - Minimal parsing
  - All attributes parsing
  - Empty value handling
  - Serialization
  - Roundtrip conversion

- **DefNumberVector Tests** (4 tests)
  - Standard parsing
  - Negative values
  - Serialization
  - Roundtrip conversion

- **DefSwitchVector Tests** (4 tests)
  - Standard parsing
  - All switch rules (OneOfMany, AtMostOne, AnyOfMany)
  - Serialization
  - Roundtrip conversion

- **DefLightVector Tests** (4 tests)
  - Standard parsing
  - All light states (Idle, Ok, Busy, Alert)
  - Serialization
  - Roundtrip conversion

- **DefBLOBVector Tests** (3 tests)
  - Standard parsing
  - Serialization
  - Roundtrip conversion

- **Set Vector Tests** (4 tests)
  - SetTextVector parsing
  - SetNumberVector parsing
  - SetSwitchVector parsing
  - Roundtrip conversion

- **New Vector Tests** (6 tests)
  - NewTextVector parsing
  - NewNumberVector parsing
  - NewSwitchVector parsing
  - Serialization
  - Roundtrip conversion

- **Control Message Tests** (7 tests)
  - EnableBLOB parsing (all modes: Never, Also, Only)
  - Message element parsing
  - DelProperty parsing (single and all properties)
  - Serialization
  - Roundtrip conversion

- **Error Handling Tests** (8 tests)
  - Invalid XML
  - Unknown message types
  - Missing required attributes
  - Invalid state values
  - Invalid permission values
  - Invalid switch states
  - Invalid switch rules
  - Invalid number values
  - Invalid BLOB enable modes

- **Edge Case Tests** (6 tests)
  - Empty text values
  - Special characters and XML entities
  - Very large numbers
  - Vectors with many elements (100+)
  - Minimal attributes
  - Unicode and emoji support

- **Property State Tests** (2 tests)
  - All property states
  - All permission types

**Total: 70+ protocol compliance tests**

### 2. Client Integration Tests (`rs_client_integration.rs`)

Integration tests for the client, transport, and protocol layers.

#### Test Coverage

- **Transport Layer Tests** (4 tests)
  - Invalid address handling
  - Connection timeout
  - Send without connection
  - Receive without connection
  - Mock server communication

- **Client Strategy Tests** (6 tests)
  - Client creation
  - Invalid URL handling
  - Disconnect without connection
  - Enumerate without connection
  - Send property without connection
  - Mock server integration

- **Property Conversion Tests** (3 tests)
  - Text property conversion
  - Number property conversion
  - Switch property conversion

- **End-to-End Workflow Tests** (1 test)
  - Complete client workflow with mock server

- **Concurrent Operations Tests** (2 tests)
  - Multiple independent clients
  - Async operation verification

- **Error Recovery Tests** (2 tests)
  - Reconnect after disconnect
  - Multiple disconnect calls

- **Live Server Tests** (5 tests, marked `#[ignore]`)
  - Connect to live INDIGO server
  - Enumerate all properties
  - Enumerate with device filter
  - Send property update
  - Receive property updates

**Total: 23 integration tests (18 unit tests + 5 live server tests)**

## Running Tests

### Run All Protocol Compliance Tests

```bash
cargo test --test rs_protocol_compliance
```

### Run All Client Integration Tests (Excluding Live Server)

```bash
cargo test --test rs_client_integration
```

### Run Specific Test Module

```bash
# Protocol tests
cargo test --test rs_protocol_compliance get_properties_tests
cargo test --test rs_protocol_compliance def_text_vector_tests
cargo test --test rs_protocol_compliance error_handling_tests

# Integration tests
cargo test --test rs_client_integration transport_tests
cargo test --test rs_client_integration client_strategy_tests
```

### Run Specific Test

```bash
cargo test --test rs_protocol_compliance test_parse_get_properties_minimal
cargo test --test rs_client_integration test_transport_with_mock_server
```

### Run Tests with Output

```bash
cargo test --test rs_protocol_compliance -- --nocapture
```

### Run Tests Against Live INDIGO Server

Some tests require a live INDIGO server running on `localhost:7624`. These tests are marked with `#[ignore]` and must be explicitly enabled.

**Prerequisites:**

1. Start an INDIGO server on localhost:7624
2. Ensure at least one device is available (e.g., CCD Simulator)

**Run live server tests:**

```bash
# Run all ignored tests
cargo test --test rs_client_integration -- --ignored

# Run specific live test
cargo test --test rs_client_integration test_connect_to_live_server -- --ignored
cargo test --test rs_client_integration test_enumerate_properties_live -- --ignored
```

## Test Coverage Summary

### Protocol Parser Coverage

- ✅ All message types (18 types)
- ✅ All property states (Idle, Ok, Busy, Alert)
- ✅ All permissions (ro, wo, rw)
- ✅ All switch rules (OneOfMany, AtMostOne, AnyOfMany)
- ✅ All switch states (On, Off)
- ✅ All BLOB enable modes (Never, Also, Only)
- ✅ Required and optional attributes
- ✅ Empty values
- ✅ Special characters and XML entities
- ✅ Unicode and emoji
- ✅ Large numbers and edge cases

### Transport Layer Coverage

- ✅ Connection establishment
- ✅ Connection timeout
- ✅ Message sending
- ✅ Message receiving
- ✅ Message framing
- ✅ Disconnect handling
- ✅ Error states

### Client Strategy Coverage

- ✅ Connection lifecycle
- ✅ Property enumeration
- ✅ Property sending
- ✅ Property receiving
- ✅ Device filtering
- ✅ Error handling
- ✅ State management

### Error Handling Coverage

- ✅ Invalid XML
- ✅ Unknown message types
- ✅ Missing attributes
- ✅ Invalid values
- ✅ Connection errors
- ✅ State errors
- ✅ Timeout errors

## Test Utilities

### Mock INDIGO Server

The test suite includes a `MockIndigoServer` utility for testing without a live INDIGO server:

```rust
let server = MockIndigoServer::new().await.unwrap();
let addr = server.addr();

// Use addr to connect clients for testing
```

The mock server:

- Listens on a random available port
- Accepts TCP connections
- Can respond to protocol messages
- Useful for isolated unit testing

### Sample Data Builders

Helper functions for creating test data:

- `sample_vector_attrs()` - Creates complete vector attributes
- `minimal_vector_attrs()` - Creates minimal required attributes

## Continuous Integration

These tests are designed to run in CI/CD environments:

1. **Unit Tests** (no external dependencies)
   - Protocol compliance tests
   - Client integration tests (with mock server)
   - Run on every commit

2. **Integration Tests** (require live server)
   - Live server tests (marked `#[ignore]`)
   - Run manually or in dedicated test environment
   - Require INDIGO server setup

## Test Maintenance

### Adding New Tests

1. **Protocol Tests**: Add to `rs_protocol_compliance.rs`
   - Group related tests in modules
   - Test both parsing and serialization
   - Include roundtrip tests
   - Test error cases

2. **Integration Tests**: Add to `rs_client_integration.rs`
   - Use mock server for unit tests
   - Mark live server tests with `#[ignore]`
   - Test error handling
   - Test concurrent scenarios

### Test Naming Convention

- `test_parse_<message_type>` - Parsing tests
- `test_serialize_<message_type>` - Serialization tests
- `test_roundtrip_<message_type>` - Roundtrip tests
- `test_<operation>_<scenario>` - Integration tests

### Documentation

Each test should have:

- Clear, descriptive name
- Comment explaining what's being tested (if not obvious)
- Assertions with meaningful messages

## Known Limitations

1. **BLOB Tests**: BLOB data handling is not fully tested due to base64 encoding complexity
2. **Timeout Tests**: Some timeout tests may be flaky in slow CI environments
3. **Mock Server**: The mock server is simplified and doesn't implement full INDIGO protocol
4. **Live Server Tests**: Require manual setup and are not run by default

## Future Improvements

- [ ] Add property builder utilities for easier test data creation
- [ ] Add more comprehensive BLOB tests
- [ ] Add performance/benchmark tests
- [ ] Add fuzzing tests for protocol parser
- [ ] Add tests for malformed XML edge cases
- [ ] Add tests for very large messages
- [ ] Add tests for connection recovery scenarios
- [ ] Add tests for concurrent property updates

## Troubleshooting

### Tests Won't Compile

Check that the main library compiles:

```bash
cargo build --lib
```

### Tests Timeout

Increase timeout for slow systems:

```bash
cargo test -- --test-threads=1
```

### Live Server Tests Fail

1. Verify INDIGO server is running: `telnet localhost 7624`
2. Check server has devices available
3. Verify no firewall blocking port 7624
4. Check server logs for errors

### Mock Server Tests Fail

- May indicate timing issues
- Try running with `--test-threads=1`
- Check for port conflicts

## Contributing

When adding new protocol features:

1. Add protocol parsing/serialization code
2. Add protocol compliance tests
3. Add integration tests
4. Update this documentation
5. Ensure all tests pass

## References

- [INDIGO Protocol Specification](https://github.com/indigo-astronomy/indigo)
- [INDIGO 1.7 DTD](../relm/src/indi-1.7.dtd)
- [Transport Implementation](../doc/transport_implementation.md)
