# INDIGO Server Test Harness

This directory contains the test harness infrastructure for managing an INDIGO server across integration tests.

## Overview

The test harness provides a comprehensive solution for integration testing with a live INDIGO server. It starts the server once before all tests, maintains it across test executions, ensures proper state management between tests, and cleanly shuts down after all tests complete.

## Architecture

The harness consists of several components:

### Core Components

1. **TestHarness** ([`harness.rs`](harness.rs))
   - Global singleton that coordinates all components
   - Provides simple API: `initialize()`, `reset_for_test()`, `server_address()`, `shutdown()`
   - Thread-safe access using `once_cell::sync::Lazy`
   - Automatic cleanup on exit

2. **ServerManager** ([`server.rs`](server.rs))
   - Manages INDIGO server process lifecycle
   - Server discovery (system path, built binary, submodule)
   - Process spawning with proper configuration
   - Output capture for debugging
   - Graceful shutdown with timeout

3. **HealthMonitor** ([`health.rs`](health.rs))
   - TCP connectivity checks
   - Readiness detection with retries and timeout
   - Health status reporting

4. **StateManager** ([`state.rs`](state.rs))
   - Lightweight state reset between tests
   - Avoids full server restart for performance
   - Ensures clean state for each test

5. **TestConfig** ([`config.rs`](config.rs))
   - Configuration from environment variables
   - Default values and validation
   - Support for custom server paths, ports, drivers, etc.

## Usage

### Basic Integration Test

```rust
use tests::harness::TestHarness;

#[tokio::test]
async fn test_something() {
    // Initialize harness (idempotent - safe to call multiple times)
    TestHarness::initialize().unwrap();

    // Reset state before test
    TestHarness::reset_for_test().await.unwrap();

    // Get server address
    let addr = TestHarness::server_address().unwrap();

    // Your test code here...
    // let mut client = RsClientStrategy::new();
    // client.connect(&addr).await.unwrap();
    // ...
}
```

### Graceful Degradation

If the INDIGO server is not available, tests can check and skip:

```rust
#[tokio::test]
async fn test_with_server() {
    if !TestHarness::is_available() {
        println!("Skipping: INDIGO server not available");
        return;
    }

    // Test continues...
}
```

## Configuration

The harness is configured via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `INDIGO_SERVER_PATH` | Path to indigo_server binary | Auto-detect |
| `INDIGO_TEST_PORT` | Port for test server | `7624` |
| `INDIGO_TEST_DRIVERS` | Comma-separated driver list | `indigo_ccd_simulator,indigo_mount_simulator` |
| `INDIGO_TEST_STARTUP_TIMEOUT` | Server startup timeout (seconds) | `10` |
| `INDIGO_TEST_SHUTDOWN_TIMEOUT` | Server shutdown timeout (seconds) | `5` |
| `INDIGO_TEST_SKIP_SERVER` | Skip server startup (use existing) | `false` |
| `INDIGO_TEST_SERVER_HOST` | Server host (if using existing) | `localhost` |
| `INDIGO_TEST_LOG_LEVEL` | Logging level | `info` |
| `INDIGO_TEST_STATE_RESET_TIMEOUT` | State reset timeout (seconds) | `2` |

### Example Configuration

```bash
# Use custom port
export INDIGO_TEST_PORT=7625

# Use specific server binary
export INDIGO_SERVER_PATH=/usr/local/bin/indigo_server

# Load additional drivers
export INDIGO_TEST_DRIVERS="indigo_ccd_simulator,indigo_mount_simulator,indigo_wheel_simulator"

# Run tests
cargo test --features rs-strategy
```

## Server Discovery

The harness attempts to locate the INDIGO server binary in this order:

1. **Environment Variable**: `INDIGO_SERVER_PATH`
2. **System Installation**: `/usr/local/bin/indigo_server`, `/usr/bin/indigo_server`
3. **Built from Source**: `sys/externals/indigo/build/bin/indigo_server`

If no server is found, the harness will initialize in "unavailable" mode, and tests can check `TestHarness::is_available()` to skip gracefully.

## Running Tests

```bash
# Run all tests (unit + integration)
cargo test --features rs-strategy

# Run only integration tests
cargo test --features rs-strategy --test integration

# Run with verbose output
cargo test --features rs-strategy -- --nocapture

# Run with single thread (for debugging)
cargo test --features rs-strategy -- --test-threads=1
```

## Implementation Details

### Singleton Pattern

The harness uses `once_cell::sync::Lazy` for one-time initialization:

```rust
static TEST_HARNESS: Lazy<Arc<Mutex<Option<TestHarnessInner>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(None))
});
```

This ensures:

- Server starts only once
- Thread-safe access
- Automatic cleanup on process exit

### State Management

Between tests, the state manager:

1. Waits for pending operations to complete
2. Allows time for connections to close
3. Provides settling time for the server

This lightweight approach avoids full server restarts, significantly improving test execution speed.

### Error Handling

The harness implements graceful degradation:

- If server binary is not found, tests can skip
- If server fails to start, initialization doesn't fail
- Tests check `is_available()` before running

## Performance

Expected performance improvements over per-test server startup:

- **Startup Time**: 5s total vs 5s per test (40x faster for 40 tests)
- **Test Execution**: 2-3 minutes vs 10-15 minutes (5x faster)
- **Memory Usage**: Constant 100MB vs 100MB per test

## Troubleshooting

### Server Binary Not Found

```
Error: INDIGO server binary not found
```

**Solution**: Set `INDIGO_SERVER_PATH` or install INDIGO server:

```bash
export INDIGO_SERVER_PATH=/usr/local/bin/indigo_server
```

### Port Already in Use

```
Error: Failed to start server: Address already in use
```

**Solution**: Use a different port or kill existing server:

```bash
export INDIGO_TEST_PORT=7625
# or
pkill indigo_server
```

### Server Not Ready Timeout

```
Error: Server failed to become ready within 10s
```

**Solution**: Increase timeout:

```bash
export INDIGO_TEST_STARTUP_TIMEOUT=30
```

### Debug Output

To see server output and debug information:

```bash
cargo test --features rs-strategy -- --nocapture
```

Or access programmatically:

```rust
let output = TestHarness::tail_server_output(20).unwrap();
for line in output {
    println!("{}", line);
}
```

## Future Enhancements

Potential improvements for future phases:

- **Enhanced State Management**: Track and reset specific device properties
- **Connection Tracking**: Monitor and disconnect active test clients
- **Protocol Verification**: Full protocol handshake in health checks
- **Parallel Test Support**: Allow multiple tests to run concurrently
- **Docker Support**: Optional Docker-based server for CI/CD
- **Test Fixtures**: Reusable test data and setup helpers

## References

- [Architecture Document](../../plans/integration_test_harness_architecture.md)
- [INDIGO Documentation](../../sys/externals/indigo/indigo_docs/)
- [Pure Rust Tests README](../README_PURE_RUST_TESTS.md)

## License

Same as parent project (MIT).
