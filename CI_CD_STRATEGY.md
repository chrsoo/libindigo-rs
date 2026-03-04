# CI/CD Strategy for libindigo-rs

## Overview

This document outlines the CI/CD strategy for the libindigo-rs project, which supports multiple build configurations and testing strategies.

## Build Strategies

The project supports two main implementation strategies:

### 1. Pure Rust Strategy (`rs-strategy`)

- **No C dependencies required**
- Implements INDIGO protocol entirely in Rust
- Suitable for CI/CD environments without system libraries
- Fast build times
- Recommended for most CI/CD pipelines

### 2. FFI Strategy (`ffi-strategy`)

- Requires INDIGO C library
- Uses FFI bindings to native INDIGO
- Requires system dependencies and build tools
- Longer build times due to C compilation
- Full hardware support

## CI/CD Pipeline Structure

### Job 1: Test Pure Rust Strategy (Primary)

**Purpose:** Fast feedback on pure Rust implementation

**Requirements:**

- ✅ No system dependencies
- ✅ No git submodules needed
- ✅ Fast build (~2-5 minutes)

**What it tests:**

- Pure Rust protocol implementation
- Transport layer
- Protocol negotiation
- Unit tests
- Integration tests (without live server)

**Configuration:**

```yaml
cargo build --features rs-strategy
cargo test --features rs-strategy --lib
INDIGO_TEST_SKIP_SERVER=true cargo test --features rs-strategy --test '*'
```

### Job 2: Build FFI Strategy (Secondary)

**Purpose:** Verify FFI bindings and C library integration

**Requirements:**

- ⚠️ Requires git submodules (`sys/externals/indigo`)
- ⚠️ Requires system dependencies (see below)
- ⚠️ Slower build (~10-20 minutes)

**System Dependencies:**

```bash
build-essential
autoconf, autotools-dev, libtool, cmake
libudev-dev
libavahi-compat-libdnssd-dev
libusb-1.0-0-dev
libcurl4-gnutls-dev
libgphoto2-dev
libz-dev
```

**What it tests:**

- FFI bindings generation
- C library compilation
- Basic integration (without live server)

**Configuration:**

```yaml
git submodule update --init --recursive
cargo build --workspace
cargo test --lib
```

### Job 3: Minimal Build Check

**Purpose:** Verify core library builds without optional features

**Configuration:**

```yaml
cargo build --no-default-features
```

### Job 4: Documentation Build

**Purpose:** Ensure documentation builds correctly

**Configuration:**

```yaml
cargo doc --no-deps --features rs-strategy
```

### Job 5: Multi-Version Rust Check

**Purpose:** Verify compatibility with stable, beta, and nightly Rust

**Configuration:**

```yaml
cargo check --features rs-strategy
```

## Integration Tests Strategy

### Challenge: Server Dependency

Integration tests ideally require a running INDIGO server, which is not feasible in most CI/CD environments because:

1. **Hardware Requirements:** INDIGO server typically needs astronomy hardware or simulators
2. **Complex Setup:** Requires server binary, drivers, and proper configuration
3. **Resource Intensive:** Server process consumes significant resources
4. **Timing Issues:** Server startup and initialization can be slow and unreliable

### Solution: Graceful Degradation

The test suite is designed to **gracefully skip** tests when the server is unavailable:

```rust
let addr = match common::setup_test().await {
    Ok(addr) => addr,
    Err(e) => {
        eprintln!("Skipping test - server not available: {}", e);
        return;
    }
};
```

**Key Features:**

- ✅ Tests timeout after 5-10 seconds if server unavailable
- ✅ Clear skip messages in test output
- ✅ No hanging or infinite retries
- ✅ CI/CD pipeline continues successfully

**Environment Variable:**

```bash
# Explicitly skip server startup attempts
INDIGO_TEST_SKIP_SERVER=true cargo test
```

## Caching Strategy

### INDIGO C Library Build Cache

The FFI strategy builds the INDIGO C library from source, which is time-consuming. We cache:

```yaml
- uses: actions/cache@v3
  with:
    path: |
      sys/externals/indigo/build
      target
    key: ${{ runner.os }}-indigo-${{ hashFiles('sys/externals/indigo/**') }}
```

**Benefits:**

- Reduces build time from ~20 minutes to ~5 minutes on cache hit
- Cache invalidates when INDIGO submodule changes
- Separate cache per OS

### Rust Build Cache

GitHub Actions automatically caches Rust builds via `actions-rust-lang/setup-rust-toolchain@v1`.

## Recommended CI/CD Configurations

### For Pull Requests (Fast Feedback)

```yaml
jobs:
  - test-rs-strategy  # Primary - fast, no dependencies
  - build-minimal     # Quick sanity check
```

### For Main Branch (Comprehensive)

```yaml
jobs:
  - test-rs-strategy      # Pure Rust tests
  - build-ffi-strategy    # FFI compilation check
  - build-minimal         # Minimal build
  - docs                  # Documentation
  - check-rust-versions   # Multi-version check
```

### For Releases (Full Validation)

```yaml
jobs:
  - All jobs above
  - Additional platform testing (macOS, Windows)
  - Security audit (cargo audit)
  - Dependency check (cargo outdated)
```

## Local Development Testing

### Quick Test (Pure Rust)

```bash
cargo test --features rs-strategy --lib
```

### Full Test (with server)

```bash
# Start INDIGO server first
indigo_server -p 7624 indigo_ccd_simulator

# Run tests
cargo test --features rs-strategy
```

### FFI Build Test

```bash
# Ensure submodule is initialized
git submodule update --init --recursive

# Build with FFI
cargo build --workspace
```

## Troubleshooting CI/CD Issues

### Issue: FFI Build Fails

**Symptoms:**

- `error: failed to run custom build command for 'libindigo-sys'`
- Missing system libraries

**Solutions:**

1. Verify all system dependencies are installed
2. Check git submodule is initialized: `git submodule status`
3. Try building INDIGO manually: `cd sys/externals/indigo && make`

### Issue: Tests Hang

**Symptoms:**

- CI job times out
- "Can't connect to socket" messages repeating

**Solutions:**

1. Ensure `INDIGO_TEST_SKIP_SERVER=true` is set
2. Check timeout values in test code (should be 5-10 seconds)
3. Verify test harness initialization has timeout protection

### Issue: Cache Not Working

**Symptoms:**

- FFI builds always take 15-20 minutes
- No cache hit messages in logs

**Solutions:**

1. Check cache key matches: `${{ hashFiles('sys/externals/indigo/**') }}`
2. Verify cache paths exist after build
3. Check GitHub Actions cache size limits (10GB per repo)

## Future Improvements

### Potential Enhancements

1. **Docker-based Testing**
   - Create Docker image with INDIGO server pre-installed
   - Run integration tests in container
   - Pros: Full integration testing
   - Cons: Increased complexity, slower builds

2. **Mock Server for Tests**
   - Implement lightweight mock INDIGO server in Rust
   - Use for integration tests in CI/CD
   - Pros: Fast, reliable, no dependencies
   - Cons: May not catch all real-world issues

3. **Nightly Integration Tests**
   - Separate workflow for full integration tests
   - Runs on schedule with real server
   - Pros: Comprehensive testing without slowing PR feedback
   - Cons: Delayed feedback on integration issues

4. **Platform-Specific Testing**
   - Add macOS and Windows runners
   - Test platform-specific code paths
   - Pros: Better platform coverage
   - Cons: Increased CI/CD time and cost

5. **Performance Benchmarking**
   - Add criterion benchmarks
   - Track performance over time
   - Pros: Catch performance regressions
   - Cons: Requires stable CI environment

## Conclusion

The current CI/CD strategy prioritizes:

- ✅ **Fast feedback** via pure Rust strategy
- ✅ **Reliability** through graceful test skipping
- ✅ **Flexibility** supporting multiple build configurations
- ✅ **Maintainability** with clear documentation and error messages

The pipeline is designed to work in standard GitHub Actions environments without requiring special hardware or complex setup, while still providing comprehensive testing of the pure Rust implementation.
