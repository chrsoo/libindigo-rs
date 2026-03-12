# Logging Configuration

## Overview

libindigo uses the [`tracing`](https://docs.rs/tracing) framework for structured logging throughout all crates. The logging system provides configurable output destinations and log levels with support for both stderr and file output.

## Quick Start

### Basic Setup

```rust
use libindigo::logging::{LogConfig, LogLevel};
use libindigo::client::ClientBuilder;

// Initialize logging with default settings (INFO level, stderr output)
let log_config = LogConfig::default();

let client = ClientBuilder::new()
    .with_strategy(strategy)
    .with_logging(log_config)
    .build()?;
```

### Custom Log Level

```rust
use libindigo::logging::{LogConfig, LogLevel};

let log_config = LogConfig::default()
    .with_level(LogLevel::Debug);

let client = ClientBuilder::new()
    .with_strategy(strategy)
    .with_logging(log_config)
    .build()?;
```

### File Output

```rust
use libindigo::logging::{LogConfig, LogLevel};
use std::path::PathBuf;

let log_config = LogConfig::default()
    .with_level(LogLevel::Info)
    .with_log_file(PathBuf::from("/var/log/indigo-client.log"));

let client = ClientBuilder::new()
    .with_strategy(strategy)
    .with_logging(log_config)
    .build()?;
```

## Log Levels

libindigo follows the standard tracing log level hierarchy:

| Level | Usage | Example |
|-------|-------|---------|
| **ERROR** | Application integrity compromised | `"Monitoring task panicked for 192.168.1.50:7624"` |
| **WARN** | Graceful recovery possible | `"ICMP unavailable, falling back to TCP connect"` |
| **INFO** | Meaningful user information (default) | `"Server 192.168.1.50:7624 status: Available -> Degraded"` |
| **DEBUG** | Additional troubleshooting info | `"Heartbeat check failed for 192.168.1.50: connection refused"` |
| **TRACE** | Detailed application logic tracing | `"ICMP ping sent to 192.168.1.50"` |

### When to Use Each Level

#### ERROR

Use for situations where application integrity is compromised:

- Fatal errors that prevent operation
- Unrecoverable failures
- Panics or critical bugs

#### WARN

Use for situations requiring graceful recovery:

- Fallback to alternative methods
- Deprecated feature usage
- Configuration issues with defaults applied

#### INFO (Default)

Use for meaningful user-facing information:

- Connection status changes
- Server availability transitions
- Successful operations
- Configuration changes

#### DEBUG

Use for troubleshooting and diagnostics:

- Individual operation failures
- Detailed state information
- Performance metrics
- Protocol-level details

#### TRACE

Use for detailed execution flow:

- Every network request/response
- State machine transitions
- Low-level protocol details
- Function entry/exit

## Configuration Options

### LogConfig Structure

```rust
pub struct LogConfig {
    /// The minimum log level to output (default: INFO)
    pub level: LogLevel,

    /// Optional file path for log output
    /// If set, logs are written to both stderr and the file
    pub log_file: Option<PathBuf>,
}
```

### Builder Methods

```rust
// Set log level
let config = LogConfig::default()
    .with_level(LogLevel::Debug);

// Set log file
let config = LogConfig::default()
    .with_log_file(PathBuf::from("app.log"));

// Chain multiple settings
let config = LogConfig::default()
    .with_level(LogLevel::Trace)
    .with_log_file(PathBuf::from("/var/log/indigo.log"));
```

## Environment Variable Override

The logging system respects the `RUST_LOG` environment variable, which takes precedence over programmatic configuration:

```bash
# Set log level for all libindigo crates
export RUST_LOG=libindigo=debug,libindigo_rs=debug,libindigo_ffi=debug

# Run your application
./my_indigo_app

# Or inline
RUST_LOG=trace ./my_indigo_app
```

### RUST_LOG Syntax

```bash
# Set global level
RUST_LOG=debug

# Set per-crate level
RUST_LOG=libindigo=info,libindigo_rs=debug

# Set per-module level
RUST_LOG=libindigo_rs::monitoring=trace

# Combine multiple targets
RUST_LOG=libindigo=info,libindigo_rs::monitoring=trace,libindigo_rs::client=debug
```

## Output Formats

### Stderr Output (Default)

By default, logs are written to stderr with ANSI color codes:

```
2026-03-12T06:45:23.123Z  INFO libindigo_rs::client: Connected to 192.168.1.50:7624
2026-03-12T06:45:28.456Z  WARN libindigo_rs::monitoring::heartbeat: ICMP unavailable, falling back to TCP connect
2026-03-12T06:45:33.789Z  INFO libindigo_rs::monitoring::monitor: Status changed from Available to Degraded
```

### File Output

When a log file is configured, logs are written to the file without ANSI codes:

```rust
let config = LogConfig::default()
    .with_log_file(PathBuf::from("indigo.log"));
```

File output includes:

- Timestamp (ISO 8601 UTC)
- Log level
- Target module
- Message

Both stderr and file output are written simultaneously when a log file is configured.

## Integration with ClientBuilder

The [`ClientBuilder`](../src/client/builder.rs) provides a fluent API for configuring logging:

```rust
use libindigo::client::ClientBuilder;
use libindigo::logging::{LogConfig, LogLevel};

let client = ClientBuilder::new()
    .with_strategy(strategy)
    .with_logging(
        LogConfig::default()
            .with_level(LogLevel::Debug)
            .with_log_file(PathBuf::from("client.log"))
    )
    .build()?;
```

### Initialization Timing

Logging is initialized when [`ClientBuilder::build()`](../src/client/builder.rs:165) is called. If initialization fails (e.g., cannot create log file), `build()` returns an error.

### Multiple Clients

The tracing subscriber can only be set once per process. If you create multiple clients:

1. The first client's logging configuration is applied
2. Subsequent clients' logging configurations are ignored
3. No error is returned (this is a tracing limitation)

**Best Practice**: Initialize logging once at application startup:

```rust
use libindigo::logging::{init_logging, LogConfig, LogLevel};

// At application startup
init_logging(&LogConfig::default().with_level(LogLevel::Info))?;

// Later, create clients without logging config
let client1 = ClientBuilder::new()
    .with_strategy(strategy1)
    .build()?;

let client2 = ClientBuilder::new()
    .with_strategy(strategy2)
    .build()?;
```

## Monitoring-Specific Logging

When using the [monitoring feature](monitoring.md), additional log levels are used:

| Component | Level | Example |
|-----------|-------|---------|
| HeartbeatChecker | TRACE | `"ICMP ping sent to 192.168.1.50"` |
| HeartbeatChecker | TRACE | `"ICMP ping response from 192.168.1.50: rtt=12ms"` |
| HeartbeatChecker | WARN | `"ICMP unavailable, falling back to TCP connect"` |
| HeartbeatChecker | DEBUG | `"Heartbeat check failed for 192.168.1.50: connection refused"` |
| ServerChecker | TRACE | `"TCP handshake attempt to 192.168.1.50:7624"` |
| ServerChecker | DEBUG | `"Server handshake failed: timeout after 2s"` |
| StatusTracker | TRACE | `"Window state: 4/5 successes, current=Available"` |
| ServerMonitor | INFO | `"Server 192.168.1.50:7624 status: Available -> Degraded"` |

### Recommended Monitoring Log Levels

```bash
# Production: Only status changes
RUST_LOG=libindigo_rs::monitoring=info

# Troubleshooting: Include check failures
RUST_LOG=libindigo_rs::monitoring=debug

# Deep debugging: All ping/handshake details
RUST_LOG=libindigo_rs::monitoring=trace
```

## Examples

### Production Configuration

```rust
use libindigo::logging::{LogConfig, LogLevel};
use std::path::PathBuf;

let log_config = LogConfig::default()
    .with_level(LogLevel::Info)
    .with_log_file(PathBuf::from("/var/log/indigo-client.log"));

let client = ClientBuilder::new()
    .with_strategy(strategy)
    .with_logging(log_config)
    .build()?;
```

### Development Configuration

```rust
use libindigo::logging::{LogConfig, LogLevel};

let log_config = LogConfig::default()
    .with_level(LogLevel::Debug);

let client = ClientBuilder::new()
    .with_strategy(strategy)
    .with_logging(log_config)
    .build()?;
```

### Debugging Specific Modules

```bash
# Debug only monitoring
RUST_LOG=libindigo=info,libindigo_rs::monitoring=debug ./app

# Trace monitoring, debug everything else
RUST_LOG=libindigo=debug,libindigo_rs::monitoring=trace ./app
```

### Standalone Logging Initialization

```rust
use libindigo::logging::{init_logging, LogConfig, LogLevel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging at application startup
    init_logging(&LogConfig::default()
        .with_level(LogLevel::Info)
        .with_log_file(PathBuf::from("app.log")))?;

    // Rest of application...
    Ok(())
}
```

## Troubleshooting

### No Log Output

**Problem**: No logs appear when running the application.

**Solutions**:

1. Check that logging is initialized (via `ClientBuilder` or `init_logging`)
2. Verify log level is not too restrictive (try `LogLevel::Trace`)
3. Check `RUST_LOG` environment variable isn't overriding settings

### Log File Not Created

**Problem**: Log file is not created or written.

**Solutions**:

1. Verify directory exists and is writable
2. Check file permissions
3. Ensure path is absolute or relative to working directory
4. Check for errors returned by `build()` or `init_logging()`

### Duplicate Log Messages

**Problem**: Log messages appear multiple times.

**Solutions**:

1. Ensure logging is only initialized once
2. Don't call both `init_logging()` and `ClientBuilder::with_logging()`
3. Check for multiple tracing subscriber initializations

### RUST_LOG Not Working

**Problem**: `RUST_LOG` environment variable has no effect.

**Solutions**:

1. Set `RUST_LOG` before running the application
2. Use correct syntax: `RUST_LOG=libindigo=debug`
3. Note: `RUST_LOG` overrides programmatic configuration
4. Verify the environment variable is exported: `echo $RUST_LOG`

## API Reference

### Types

- [`LogConfig`](../src/logging.rs:62) - Logging configuration
- [`LogLevel`](../src/logging.rs:18) - Log level enumeration

### Functions

- [`init_logging()`](../src/logging.rs:104) - Initialize logging system
- [`LogConfig::default()`](../src/logging.rs:71) - Create default configuration
- [`LogConfig::with_level()`](../src/logging.rs:82) - Set log level
- [`LogConfig::with_log_file()`](../src/logging.rs:88) - Set log file path

## See Also

- [Monitoring Documentation](monitoring.md) - Server monitoring and status tracking
- [Client Builder API](../src/client/builder.rs) - Client construction
- [tracing Documentation](https://docs.rs/tracing) - Underlying logging framework
- [tracing-subscriber Documentation](https://docs.rs/tracing-subscriber) - Subscriber configuration

---

**Last Updated**: 2026-03-12
**Version**: 0.2.0
