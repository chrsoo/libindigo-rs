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
   - Enhanced with diagnostic and helper methods

2. **ServerManager** ([`server.rs`](server.rs))
   - Manages INDIGO server process lifecycle
   - Server discovery (system path, built binary, submodule)
   - Process spawning with proper configuration
   - Output capture for debugging (last 1000 lines)
   - Graceful shutdown with timeout
   - **New**: Restart capability, uptime tracking, PID access

3. **HealthMonitor** ([`health.rs`](health.rs))
   - TCP connectivity checks
   - Readiness detection with retries and timeout
   - **New**: Exponential backoff retry logic
   - **New**: Configurable retry strategy
   - **New**: Comprehensive health checks
   - **New**: Stability testing
   - Health status reporting

4. **StateManager** ([`state.rs`](state.rs))
   - Lightweight state reset between tests
   - **New**: Connection tracking
   - **New**: Device tracking
   - **New**: Pre/post-test state verification
   - **New**: Resource leak detection
   - **New**: Statistics collection
   - Avoids full server restart for performance
   - Ensures clean state for each test run
   - See [Test Isolation Guide](../TEST_ISOLATION_GUIDE.md) for best practices

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
    TestHarness::initialize().await.unwrap();

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

### Advanced Usage

#### Server Restart

```rust
#[tokio::test]
async fn test_with_restart() {
    TestHarness::initialize().await.unwrap();

    // Restart the server if needed
    TestHarness::restart_server().await.unwrap();

    // Continue testing...
}
```

#### Health Monitoring

```rust
#[tokio::test]
async fn test_health_check() {
    TestHarness::initialize().await.unwrap();

    // Check connectivity
    let connected = TestHarness::check_connectivity().await.unwrap();
    assert!(connected);

    // Get full health status
    let status = TestHarness::get_server_status().await.unwrap();
    assert!(status.reachable);
    assert!(status.protocol_responsive);
}
```

#### State Tracking and Verification

```rust
#[tokio::test]
async fn test_with_state_tracking() {
    TestHarness::initialize().await.unwrap();
    TestHarness::reset_for_test().await.unwrap();

    // Verify pre-test state is clean
    TestHarness::verify_pre_test_state().await.unwrap();

    // Get state statistics
    let stats = TestHarness::get_state_statistics().unwrap();
    println!("Total resets: {}", stats.total_resets);
    println!("Active connections: {}", stats.active_connections);
    println!("Verification failures: {}", stats.verification_failures);
    println!("Resource leaks: {}", stats.resource_leaks);

    // Verify post-test state is clean
    TestHarness::verify_post_test_state().await.unwrap();
}
```

For comprehensive examples and best practices, see the [Test Isolation Guide](../TEST_ISOLATION_GUIDE.md).

#### Diagnostics

```rust
#[tokio::test]
async fn test_with_diagnostics() {
    TestHarness::initialize().await.unwrap();

    // Print comprehensive diagnostics
    TestHarness::print_diagnostics();

    // Get server uptime
    if let Ok(Some(uptime)) = TestHarness::server_uptime() {
        println!("Server uptime: {:?}", uptime);
    }

    // Get server PID
    if let Ok(Some(pid)) = TestHarness::server_pid() {
        println!("Server PID: {}", pid);
    }
}
```

#### Debug Output

```rust
#[tokio::test]
async fn test_with_debug_output() {
    TestHarness::initialize().await.unwrap();

    // Get last 20 lines of server output
    if let Ok(output) = TestHarness::tail_server_output(20) {
        for line in output {
            println!("{}", line);
        }
    }
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

# Increase startup timeout
export INDIGO_TEST_STARTUP_TIMEOUT=30

# Run tests
cargo test --features rs
```

## Server Discovery

The harness attempts to locate the INDIGO server binary in this order:

1. **Environment Variable**: `INDIGO_SERVER_PATH`
2. **Built from Source**: `sys/externals/indigo/build/bin/indigo_server`
3. **System PATH**: Using `which` (Unix) or `where` (Windows)
4. **System Installation**: `/usr/local/bin/indigo_server`, `/usr/bin/indigo_server`, `/opt/indigo/bin/indigo_server`

If no server is found, the harness will initialize in "unavailable" mode, and tests can check `TestHarness::is_available()` to skip gracefully.

## Running Tests

```bash
# Run all tests (unit + integration)
cargo test --features rs

# Run only integration tests
cargo test --features rs --test integration

# Run with verbose output
cargo test --features rs -- --nocapture

# Run with single thread (for debugging)
cargo test --features rs -- --test-threads=1
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

1. Waits for pending operations to complete (100ms)
2. Clears tracked state (connections, devices)
3. Provides settling time for the server (100ms)
4. Tracks statistics for monitoring

This lightweight approach avoids full server restarts, significantly improving test execution speed.

### Health Monitoring

The health monitor implements:

- **Exponential Backoff**: Retry delays increase exponentially (100ms → 200ms → 400ms → ...)
- **Maximum Delay**: Caps retry delay at 5 seconds
- **Configurable Retries**: Default 20 retries, customizable
- **Stability Testing**: Can perform multiple checks to verify server stability

### Error Handling

The harness implements graceful degradation:

- If server binary is not found, tests can skip
- If server fails to start, initialization doesn't fail
- Tests check `is_available()` before running
- Detailed error messages guide troubleshooting

## Performance

Expected performance improvements over per-test server startup:

- **Startup Time**: 5s total vs 5s per test (40x faster for 40 tests)
- **Test Execution**: 2-3 minutes vs 10-15 minutes (5x faster)
- **Memory Usage**: Constant 100MB vs 100MB per test
- **State Reset**: 200ms per test (lightweight)

## API Reference

### TestHarness Methods

#### Initialization & Lifecycle

- `initialize() -> Result<(), String>` - Initialize harness (async, idempotent)
- `shutdown() -> Result<(), String>` - Shutdown harness and stop server
- `restart_server() -> Result<(), String>` - Restart the server (async)
- `is_available() -> bool` - Check if harness is available

#### Server Information

- `server_address() -> Result<String, String>` - Get server address
- `server_state() -> Result<ServerState, String>` - Get server state
- `server_uptime() -> Result<Option<Duration>, String>` - Get server uptime
- `server_pid() -> Result<Option<u32>, String>` - Get server process ID

#### State Management

- `reset_for_test() -> Result<(), String>` - Reset state between tests (async)
- `verify_pre_test_state() -> Result<(), String>` - Verify clean state before test (async)
- `verify_post_test_state() -> Result<(), String>` - Verify clean state after test (async)
- `force_clean_state() -> Result<(), String>` - Force clean state (async)
- `get_state_statistics() -> Result<StateStatistics, String>` - Get state stats
- `track_connection() -> Result<(), String>` - Track a connection
- `untrack_connection() -> Result<(), String>` - Untrack a connection
- `track_device(device: &str) -> Result<(), String>` - Track a device interaction

#### Health Monitoring

- `check_connectivity() -> Result<bool, String>` - Check TCP connectivity (async)
- `wait_for_ready(timeout: Duration) -> Result<(), String>` - Wait for server (async)
- `get_server_status() -> Result<ServerStatus, String>` - Get health status (async)

#### Debugging

- `server_output() -> Result<Vec<String>, String>` - Get all captured output
- `tail_server_output(lines: usize) -> Result<Vec<String>, String>` - Get last N lines
- `clear_server_output() -> Result<(), String>` - Clear output buffer
- `print_diagnostics()` - Print comprehensive diagnostics

## Troubleshooting

### Server Binary Not Found

```
Error: INDIGO server binary not found
```

**Solution**: Set `INDIGO_SERVER_PATH` or install INDIGO server:

```bash
export INDIGO_SERVER_PATH=/usr/local/bin/indigo_server
# or
cd sys/externals/indigo && make
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
cargo test --features rs -- --nocapture
```

Or access programmatically:

```rust
let output = TestHarness::tail_server_output(20).unwrap();
for line in output {
    println!("{}", line);
}
```

### Diagnostics

Print comprehensive diagnostics:

```rust
TestHarness::print_diagnostics();
```

Output includes:

- Server state
- Server uptime
- Server PID
- Server address
- State reset count
- Active connections
- Tracked devices
- Last reset time

## Enhancements in This Implementation

### ServerManager Enhancements

- ✅ **Restart capability**: Can restart server without recreating harness
- ✅ **Uptime tracking**: Track how long server has been running
- ✅ **PID access**: Get process ID for debugging
- ✅ **Better logging**: More detailed status messages
- ✅ **Output management**: Clear output buffer when needed

### HealthMonitor Enhancements

- ✅ **Exponential backoff**: Intelligent retry delays
- ✅ **Configurable retries**: Custom retry strategies
- ✅ **Comprehensive checks**: Full health status reporting
- ✅ **Stability testing**: Verify server stability over time
- ✅ **Better error messages**: Detailed failure information

### StateManager Enhancements

- ✅ **Connection tracking**: Track active test connections
- ✅ **Device tracking**: Track which devices have been used
- ✅ **Pre/post-test verification**: Strict state verification before and after tests
- ✅ **Resource leak detection**: Detect and report resource leaks
- ✅ **Statistics**: Collect and report state management stats including leaks and failures
- ✅ **Custom timeouts**: Per-reset timeout configuration
- ✅ **Force clean**: Recovery mechanism for dirty state

### TestHarness Enhancements

- ✅ **Restart support**: Restart server without full reinitialization
- ✅ **Health checks**: Direct access to health monitoring
- ✅ **State statistics**: Access to state management metrics
- ✅ **Diagnostics**: Comprehensive diagnostic output
- ✅ **Better error messages**: User-friendly error reporting
- ✅ **More helper methods**: Uptime, PID, output management

## Test Isolation

The harness now includes comprehensive test isolation mechanisms:

- **State Verification**: Pre/post-test state checks to ensure clean state
- **Resource Tracking**: Track connections and devices to detect leaks
- **RAII Guards**: Automatic cleanup with `ConnectionGuard` and `DeviceGuard`
- **Setup Utilities**: Multiple setup functions for different isolation needs
- **Statistics**: Track verification failures and resource leaks

For detailed information on writing isolated tests, see the [Test Isolation Guide](../TEST_ISOLATION_GUIDE.md).

## Future Enhancements

Potential improvements for future phases:

- **Enhanced State Management**: Full protocol-level property reset
- **Active Connection Tracking**: Monitor and disconnect active test clients
- **Protocol Verification**: Full protocol handshake in health checks
- **Parallel Test Support**: Allow multiple tests to run concurrently with proper isolation
- **Docker Support**: Optional Docker-based server for CI/CD
- **Test Fixtures**: Reusable test data and setup helpers
- **Performance Metrics**: Track and report test execution times

## References

- [Test Isolation Guide](../TEST_ISOLATION_GUIDE.md) - Best practices for writing isolated tests
- [Isolation Tests](../isolation_tests.rs) - Examples of isolation verification
- [Architecture Document](../../plans/integration_test_harness_architecture.md)
- [INDIGO Documentation](../../sys/externals/indigo/indigo_docs/)
- [Implementation Summary](../HARNESS_IMPLEMENTATION.md)

## License

Same as parent project (MIT).
