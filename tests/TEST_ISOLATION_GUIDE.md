# Test Isolation Guide

This guide documents best practices for writing isolated tests and understanding the test isolation mechanisms in the libindigo test harness.

## Overview

Test isolation ensures that tests are independent and don't interfere with each other. The test harness provides several mechanisms to achieve this:

1. **State Management** - Tracks and resets server state between tests
2. **State Verification** - Detects resource leaks and incomplete cleanup
3. **Resource Tracking** - Monitors connections and device interactions
4. **Cleanup Utilities** - RAII guards and helper functions for automatic cleanup

## Quick Start

### Basic Test Setup

For simple tests that don't need strict verification:

```rust
#[tokio::test]
async fn test_basic_operation() -> Result<(), Box<dyn std::error::Error>> {
    let addr = match common::setup_test().await {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Skipping test: {}", e);
            return Ok(());
        }
    };

    // Your test code here...

    Ok(())
}
```

### Test Setup with Verification

For tests that need strict isolation guarantees:

```rust
#[tokio::test]
async fn test_with_verification() -> Result<(), Box<dyn std::error::Error>> {
    let (addr, _guard) = match common::setup_test_with_verification().await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Skipping test: {}", e);
            return Ok(());
        }
    };

    // Your test code here...

    // Guard automatically verifies state on drop
    Ok(())
}
```

### Lenient Test Setup

For tests that need verification but want to be forgiving of pre-existing issues:

```rust
#[tokio::test]
async fn test_lenient() -> Result<(), Box<dyn std::error::Error>> {
    let (addr, _guard) = common::setup_test_lenient().await?;

    // Your test code here...

    Ok(())
}
```

## State Management

### How State Reset Works

The test harness performs the following steps during state reset:

1. **Pre-reset Verification** - Detects any dirty state from previous tests
2. **Wait for Pending Operations** - Allows in-flight messages to complete (100ms)
3. **Clear Tracked State** - Resets connection and device tracking
4. **Settling Time** - Gives the server time to stabilize (100ms)
5. **Post-reset Verification** - Ensures state is clean

### Manual State Reset

```rust
// Reset state between test phases
TestHarness::reset_for_test().await?;
```

### Force Clean State

For recovery scenarios when normal reset fails:

```rust
// Aggressively clear all tracked state
TestHarness::force_clean_state().await?;
```

## State Verification

### Pre-test Verification

Ensures the test starts with clean state:

```rust
// Verify state is clean before test
TestHarness::verify_pre_test_state().await?;
```

If this fails, it indicates incomplete cleanup from a previous test.

### Post-test Verification

Detects resource leaks after test completion:

```rust
// Verify state is clean after test
TestHarness::verify_post_test_state().await?;
```

If this fails, it indicates the test didn't clean up properly.

## Resource Tracking

### Connection Tracking

Track connections to detect leaks:

```rust
// Manual tracking
TestHarness::track_connection()?;
// ... use connection ...
TestHarness::untrack_connection()?;

// Or use RAII guard (recommended)
{
    let _conn = common::ConnectionGuard::new()?;
    // Connection automatically tracked and untracked
}
```

### Device Tracking

Track device interactions:

```rust
// Manual tracking
TestHarness::track_device("CCD Simulator")?;

// Or use RAII guard (recommended)
{
    let _device = common::DeviceGuard::new("CCD Simulator")?;
    // Device automatically tracked
}
```

## Best Practices

### 1. Always Use Setup Utilities

✅ **Good:**

```rust
let addr = common::setup_test().await?;
```

❌ **Bad:**

```rust
TestHarness::initialize().await?;
let addr = TestHarness::server_address()?;
// Missing reset!
```

### 2. Use RAII Guards for Resource Management

✅ **Good:**

```rust
{
    let _conn = common::ConnectionGuard::new()?;
    // Use connection
} // Automatically cleaned up
```

❌ **Bad:**

```rust
TestHarness::track_connection()?;
// Use connection
// Forgot to untrack!
```

### 3. Clean Up Resources Explicitly

✅ **Good:**

```rust
let mut client = create_client();
client.connect(&addr).await?;
// ... test code ...
client.disconnect().await?; // Explicit cleanup
```

❌ **Bad:**

```rust
let mut client = create_client();
client.connect(&addr).await?;
// ... test code ...
// Connection left open!
```

### 4. Use Verification Guards for Critical Tests

✅ **Good:**

```rust
let (addr, _guard) = common::setup_test_with_verification().await?;
// Guard ensures cleanup verification
```

❌ **Bad:**

```rust
let addr = common::setup_test().await?;
// No verification of cleanup
```

### 5. Handle Errors Gracefully

✅ **Good:**

```rust
let addr = match common::setup_test().await {
    Ok(addr) => addr,
    Err(e) => {
        eprintln!("Skipping test: {}", e);
        return Ok(());
    }
};
```

❌ **Bad:**

```rust
let addr = common::setup_test().await.unwrap(); // Panics if server unavailable
```

## Common Patterns

### Pattern 1: Simple Test

```rust
#[tokio::test]
async fn test_simple() -> Result<(), Box<dyn std::error::Error>> {
    let addr = match common::setup_test().await {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Skipping test: {}", e);
            return Ok(());
        }
    };

    let mut client = ClientBuilder::new()
        .with_async_ffi_strategy()
        .build()?;

    client.connect(&addr).await?;
    // Test operations...
    client.disconnect().await?;

    Ok(())
}
```

### Pattern 2: Test with Resource Tracking

```rust
#[tokio::test]
async fn test_with_tracking() -> Result<(), Box<dyn std::error::Error>> {
    let (addr, _guard) = common::setup_test_with_verification().await?;

    let _conn = common::ConnectionGuard::new()?;
    let _device = common::DeviceGuard::new("CCD Simulator")?;

    let mut client = ClientBuilder::new()
        .with_async_ffi_strategy()
        .build()?;

    client.connect(&addr).await?;
    // Test operations...
    client.disconnect().await?;

    Ok(())
}
```

### Pattern 3: Multi-phase Test

```rust
#[tokio::test]
async fn test_multi_phase() -> Result<(), Box<dyn std::error::Error>> {
    let addr = common::setup_test().await?;

    // Phase 1
    {
        let mut client = create_client();
        client.connect(&addr).await?;
        // Phase 1 operations...
        client.disconnect().await?;
    }

    // Reset between phases
    TestHarness::reset_for_test().await?;

    // Phase 2
    {
        let mut client = create_client();
        client.connect(&addr).await?;
        // Phase 2 operations...
        client.disconnect().await?;
    }

    Ok(())
}
```

## Monitoring and Debugging

### Get State Statistics

```rust
let stats = TestHarness::get_state_statistics()?;
eprintln!("Active connections: {}", stats.active_connections);
eprintln!("Tracked devices: {}", stats.tracked_devices);
eprintln!("Total resets: {}", stats.total_resets);
eprintln!("Verification failures: {}", stats.verification_failures);
eprintln!("Resource leaks: {}", stats.resource_leaks);
```

### Print Diagnostics

```rust
TestHarness::print_diagnostics();
```

This prints comprehensive information about:

- Server state
- Server uptime
- Server PID
- State statistics
- Recent resets

## Limitations

### Current Limitations

1. **Time-based Reset** - The current implementation uses time-based delays (200ms total) rather than protocol-level state verification. This is sufficient for most scenarios but may not catch all edge cases.

2. **No Protocol-level Reset** - The harness cannot send protocol commands to reset device properties to defaults. Tests must handle their own device state management.

3. **No Forced Disconnection** - The harness cannot forcibly disconnect clients. Tests must disconnect explicitly.

4. **Async Drop Limitation** - The `TestGuard` cannot perform async verification in its `Drop` implementation, so it only logs warnings. Actual verification happens during the next reset.

5. **Shared Server State** - All tests share the same INDIGO server instance. While state tracking helps, some server-level state may persist between tests.

### Future Enhancements

The following improvements are planned for future releases:

1. **Protocol-level State Verification** - Send protocol commands to verify and reset device state
2. **Active Connection Tracking** - Track actual client connections and force disconnect if needed
3. **Property State Snapshots** - Capture and restore device property states
4. **Parallel Test Isolation** - Support for concurrent test execution with proper isolation
5. **Configurable Reset Strategies** - Allow tests to choose between time-based and protocol-based reset

## Troubleshooting

### Problem: Tests fail with "Pre-test state is dirty"

**Cause:** A previous test didn't clean up properly.

**Solution:**

1. Use `setup_test_lenient()` to force clean state
2. Review the previous test for missing cleanup
3. Add explicit cleanup code or use RAII guards

### Problem: Tests fail with "Post-test state is dirty"

**Cause:** The current test didn't clean up properly.

**Solution:**

1. Ensure all connections are disconnected
2. Use RAII guards for automatic cleanup
3. Add explicit cleanup in a `defer` or at test end

### Problem: Resource leak warnings in test output

**Cause:** Tests are leaving resources tracked.

**Solution:**

1. Use `ConnectionGuard` and `DeviceGuard` for automatic tracking
2. Ensure `untrack_connection()` is called for each `track_connection()`
3. Review test cleanup logic

### Problem: Tests interfere with each other

**Cause:** Insufficient state reset between tests.

**Solution:**

1. Use `setup_test_with_verification()` for strict isolation
2. Add explicit `reset_for_test()` calls between test phases
3. Ensure proper cleanup in all tests

## Examples

See [`tests/isolation_tests.rs`](isolation_tests.rs) for comprehensive examples of:

- State verification
- Resource tracking
- RAII guards
- Sequential test isolation
- Statistics tracking

## Summary

The test isolation mechanisms provide:

✅ **Automatic state reset** between tests
✅ **Resource leak detection** via tracking
✅ **Pre/post-test verification** for strict isolation
✅ **RAII guards** for automatic cleanup
✅ **Statistics tracking** for monitoring
✅ **Flexible setup utilities** for different needs

By following these best practices, you can write tests that are:

- **Independent** - Don't rely on other tests
- **Repeatable** - Produce consistent results
- **Isolated** - Don't interfere with each other
- **Clean** - Properly clean up resources
- **Debuggable** - Easy to diagnose issues
