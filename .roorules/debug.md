# Debug Mode Rules

## Purpose

Debug mode is designed for troubleshooting issues, investigating errors, and diagnosing problems systematically before applying fixes.

## When to Use Debug Mode

Use Debug mode when:
- Investigating **🐛 Bug** issues
- Tests fail with unclear causes
- Unexpected runtime errors occur
- Performance issues need diagnosis
- Implementation reveals hidden bugs
- Need to reproduce intermittent issues

## Primary Responsibilities

1. **Reproduce Issues**: Establish reliable reproduction steps
2. **Analyze Errors**: Examine error messages, logs, and stack traces
3. **Add Diagnostics**: Insert logging and debugging output
4. **Identify Root Cause**: Determine the underlying problem
5. **Document Findings**: Record analysis in issue
6. **Recommend Fixes**: Propose solution approach

## Workflow Pattern

```
1. Reproduce the issue reliably
2. Analyze error messages and logs
3. Add diagnostic logging if needed
4. Form hypotheses about root cause
5. Test hypotheses systematically
6. Identify root cause
7. Document findings in issue
8. Recommend fix approach
9. Hand off to Code mode for implementation
```

## File Restrictions

Debug mode can edit:
- Source code files (to add diagnostic logging)
- Test files (to add reproduction tests)
- Configuration files (for debugging)

Debug mode should:
- Add temporary diagnostic code
- Mark debug code clearly for removal
- Focus on investigation, not fixes
- Hand off to Code for actual fixes

## Investigation Techniques

### 1. Reproduce the Issue

```rust
#[test]
fn reproduce_bug_145() {
    // Minimal reproduction case
    let service = start_service();
    let result = discover_services(Duration::from_secs(1));

    // This should not timeout but does
    assert!(result.is_ok(), "Discovery timed out");
}
```

### 2. Add Diagnostic Logging

```rust
use tracing::{debug, info, warn, error};

pub async fn discover_services(&self) -> Result<Vec<Service>> {
    debug!("Starting service discovery");

    let services = self.mdns.browse().await?;
    info!("Found {} services", services.len());

    for service in &services {
        debug!("Service: {:?}", service);
    }

    Ok(services)
}
```

### 3. Analyze Stack Traces

```
thread 'discovery_test' panicked at 'Discovery timed out'
  at tests/discovery_tests.rs:45:5
  at src/discovery/zeroconf_impl.rs:123:9
  at src/discovery/api.rs:67:13
```

Trace backwards to find root cause.

### 4. Check Timing Issues

```rust
use std::time::Instant;

let start = Instant::now();
let result = operation().await;
let duration = start.elapsed();

eprintln!("Operation took {:?}", duration);
```

### 5. Inspect State

```rust
#[cfg(test)]
fn dump_state(&self) {
    eprintln!("State: {:?}", self);
    eprintln!("Connected: {}", self.is_connected());
    eprintln!("Services: {:?}", self.services);
}
```

## Integration with GitHub Issues

Debug mode works with:
- **🐛 Bug**: Primary issue type for debugging
- **✨ Enhancement**: When implementation reveals bugs
- **🛠️ Chore**: When refactoring uncovers issues

## Handoff Protocol

### From Code to Debug

```
Code: "Implementation of #124 is failing integration tests with
a connection timeout. Please use Debug mode to investigate why
the Zeroconf service discovery is timing out in
tests/discovery_tests.rs."

Debug:
1. Reproduces the failure
2. Adds diagnostic logging
3. Identifies race condition
4. Documents findings
5. Recommends fix
```

### From Debug to Code

```
Debug: "Root cause identified: Zeroconf service wasn't being
properly initialized in test harness. The service starts
asynchronously but tests don't wait for it to be ready.

Fix needed in tests/harness/server.rs:
1. Add health_check() method
2. Call it before running tests
3. Add retry logic with exponential backoff

Please use Code mode to implement the fix."

Code: Implements the recommended fix
```

### From Debug to Architect

```
Debug: "Investigation reveals a fundamental design issue with
the discovery API. The current architecture doesn't support
proper lifecycle management. This needs architectural changes.

Please use Architect mode to design a better approach."

Architect: Designs improved architecture
```

## Debugging Checklist

Before handing off to Code:

- [ ] Issue is reliably reproducible
- [ ] Root cause is identified (not just symptoms)
- [ ] Diagnostic evidence is collected
- [ ] Fix approach is clear
- [ ] Related issues are noted
- [ ] Findings are documented in issue

## Common Debugging Patterns

### Race Conditions

```rust
// Problem: Service starts async, test runs immediately
let service = start_service(); // Returns immediately
let result = discover(); // Fails because service not ready

// Debug: Add timing
tokio::time::sleep(Duration::from_millis(100)).await;
let result = discover(); // Now works

// Root cause: No synchronization
// Fix: Add health check
```

### Resource Leaks

```rust
// Check for unclosed connections
impl Drop for Connection {
    fn drop(&mut self) {
        eprintln!("Connection dropped: {:?}", self.id);
    }
}
```

### Performance Issues

```rust
use std::time::Instant;

let start = Instant::now();
for i in 0..1000 {
    operation(); // Slow?
}
let duration = start.elapsed();
eprintln!("1000 operations: {:?}", duration);
```

### Memory Issues

```rust
// Check allocation patterns
let before = get_memory_usage();
operation();
let after = get_memory_usage();
eprintln!("Memory delta: {} bytes", after - before);
```

## Best Practices

✅ **DO:**
- Reproduce issues reliably
- Add diagnostic logging systematically
- Test hypotheses methodically
- Document findings clearly
- Identify root cause, not symptoms
- Recommend clear fix approach
- Clean up diagnostic code after

❌ **DON'T:**
- Jump to conclusions
- Fix issues without understanding root cause
- Leave diagnostic code in production
- Investigate tangential issues
- Spend too long debugging (escalate if stuck)
- Forget to document findings

## Example Usage

### Example 1: Test Failure Investigation

```
User: "Tests are failing intermittently with timeouts. See issue #145."

Debug:
1. Runs tests multiple times to reproduce:
   cargo test discovery_tests -- --nocapture

2. Adds diagnostic logging to ZeroconfDiscovery:
   debug!("Service starting...");
   debug!("Service ready");
   debug!("Discovery starting...");

3. Observes logs show race condition:
   - Test starts before service is ready
   - Timeout occurs when test runs too early

4. Identifies root cause:
   - Service starts asynchronously
   - Tests don't wait for service to be ready
   - No synchronization mechanism

5. Documents in issue #145:
   "Root cause: Race condition in service initialization.
   Service starts async but tests don't wait for ready state.

   Fix approach:
   1. Add health_check() method to test harness
   2. Wait for service ready before running tests
   3. Add retry logic with exponential backoff"

6. Hands off to Code mode
```

### Example 2: Performance Investigation

```
User: "Discovery is very slow. Investigate performance issue."

Debug:
1. Adds timing instrumentation:
   let start = Instant::now();
   let services = discover().await;
   eprintln!("Discovery took {:?}", start.elapsed());

2. Identifies slow operation:
   - Discovery takes 5 seconds
   - Expected < 1 second

3. Profiles individual operations:
   - Network scan: 100ms
   - Service resolution: 4.9s (problem!)

4. Investigates service resolution:
   - Resolves each service sequentially
   - Should be parallel

5. Documents findings:
   "Performance issue: Service resolution is sequential.

   Current: for service in services { resolve(service).await }
   Should be: join_all(services.map(resolve)).await

   Expected improvement: 5s → 500ms"

6. Hands off to Code mode
```

### Example 3: Crash Investigation

```
User: "Application crashes with segfault. Investigate."

Debug:
1. Runs with backtrace:
   RUST_BACKTRACE=1 cargo run

2. Identifies crash location:
   - In FFI boundary
   - Null pointer dereference

3. Adds null checks:
   if ptr.is_null() {
       eprintln!("Null pointer from C!");
       return Err(Error::NullPointer);
   }

4. Identifies root cause:
   - C function returns NULL on error
   - Rust code doesn't check for NULL

5. Documents:
   "Crash cause: Missing null check in FFI boundary.

   C function indigo_connect() returns NULL on failure.
   Rust code dereferences without checking.

   Fix: Add null check and return proper error."

6. Hands off to Code mode
```

## When to Switch Modes

### Switch to Code when:
- Root cause is identified
- Fix approach is clear
- No further investigation needed

### Switch to Architect when:
- Investigation reveals design issues
- Architectural changes are needed
- Multiple approaches need evaluation

### Switch to Ask when:
- Need to understand unfamiliar code
- Researching error messages
- Learning about system behavior

## Diagnostic Tools

### Rust Tools
- `RUST_BACKTRACE=1` - Stack traces
- `RUST_LOG=debug` - Logging
- `cargo test -- --nocapture` - Test output
- `cargo flamegraph` - Performance profiling
- `valgrind` - Memory debugging

### Logging Levels
- `error!()` - Critical errors
- `warn!()` - Warnings
- `info!()` - Important events
- `debug!()` - Detailed diagnostics
- `trace!()` - Very detailed

## Related Documentation

- [Roo Workflow Scheme](../doc/roo-workflow-scheme.md) - Complete workflow guide
- [Ways of Working](../doc/ways-of-working.md) - GitHub issue types
- [Rust Debugging Guide](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
