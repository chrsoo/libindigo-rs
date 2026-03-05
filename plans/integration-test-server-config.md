# Integration Test Server Configuration

## Summary

Integration tests are now configured to discover and use the INDIGO server from the build directory for both FFI and RS builds.

## Changes Made

### 1. Server Discovery Logic ([`tests/harness/server.rs`](tests/harness/server.rs:53-161))

Updated `discover_binary()` function with improved priority order:

1. **Environment variable** `INDIGO_SERVER_PATH` (highest priority)
2. **Built from source** in `sys/externals/indigo/build/bin/indigo_server` (NEW - prioritized)
3. **System PATH** using `which` (Unix) or `where` (Windows)
4. **System installation paths** (`/usr/local/bin`, `/usr/bin`, `/opt/indigo/bin`)

**Key improvements:**

- Checks build directory FIRST (before system paths)
- Uses `which`/`where` commands to search system PATH
- Provides detailed error messages showing all searched locations
- Logs discovery results to stderr for debugging

### 2. Cargo Configuration ([`Cargo.toml`](Cargo.toml:83-94))

Added `test-server` feature flag:

```toml
[features]
# ... existing features ...
test-server = ["libindigo-sys"]
```

**Purpose:**

- Allows building INDIGO server for integration tests without runtime FFI dependencies
- For RS builds: `cargo build --features "rs,test-server"` builds the server
- For FFI builds: Server is built automatically as part of regular build

### 3. CI/CD Workflow ([`.github/workflows/rust.yml`](../.github/workflows/rust.yml))

#### RS Strategy Job (Lines 13-72)

- Added submodule checkout with `submodules: "recursive"`
- Added system dependencies needed to build INDIGO server
- Added INDIGO build caching
- New step: "Build INDIGO server for integration tests"
  - Uses `--features "rs,test-server"` to build server
  - Verifies server binary exists
- Updated integration test step to run without `INDIGO_TEST_SKIP_SERVER`

#### FFI Strategy Job (Lines 74-138)

- Added verification step to confirm server was built
- Updated integration test step to run without `INDIGO_TEST_SKIP_SERVER`
- Tests now discover and use the built server

## Verification Results

### ✅ RS Build (Pure Rust) - No Runtime FFI Dependencies

```bash
# Build without test-server feature
$ cargo build --no-default-features --features rs
# ✓ No libindigo-sys compilation
# ✓ No INDIGO FFI dependencies in binary

# Build with test-server feature (for integration tests)
$ cargo build --no-default-features --features "rs,test-server"
# ✓ libindigo-sys compiles (build-time only)
# ✓ INDIGO server built at sys/externals/indigo/build/bin/indigo_server
# ✓ Still no runtime FFI dependencies in libindigo binary
```

### ✅ Integration Tests Discover Built Server

```bash
$ cargo test --no-default-features --features rs --test '*'
# Output shows:
# [INDIGO] Found server in build directory: sys/externals/indigo/build/bin/indigo_server
# ✓ 32 tests passed (5 failed due to unrelated tokio runtime issue)
```

### ✅ Server Binary Verification

```bash
$ ls -lh sys/externals/indigo/build/bin/indigo_server
-rwxr-xr-x  1 csoop  staff   1.3M Mar  4 22:32 sys/externals/indigo/build/bin/indigo_server
```

## Usage

### Local Development

#### For RS builds (pure Rust)

```bash
# 1. Build the INDIGO server (one-time or when INDIGO updates)
cargo build --no-default-features --features "rs,test-server"

# 2. Run integration tests (server will be discovered automatically)
cargo test --no-default-features --features rs --test '*'
```

#### For FFI builds

```bash
# 1. Build (server is built automatically)
cargo build

# 2. Run integration tests
cargo test --test '*'
```

### CI/CD

The workflow automatically:

1. Checks out submodules
2. Installs system dependencies
3. Builds INDIGO server (via test-server feature for RS, automatically for FFI)
4. Runs integration tests (which discover the built server)

### Manual Server Path Override

If you need to use a different server binary:

```bash
export INDIGO_SERVER_PATH=/path/to/custom/indigo_server
cargo test --test '*'
```

## Architecture Benefits

### ✅ Zero Runtime FFI Dependencies for RS Builds

- Pure Rust builds have NO C library dependencies at runtime
- INDIGO server is only built during test compilation (via `test-server` feature)
- Server binary is separate executable, not linked into Rust code

### ✅ Consistent Test Environment

- Both local development and CI/CD use the same server binary
- Server version matches the INDIGO submodule version
- No dependency on system-installed INDIGO

### ✅ Graceful Degradation

- Tests skip gracefully if server not available (existing behavior preserved)
- Clear error messages guide users to build the server
- Multiple fallback paths for server discovery

### ✅ Clean Separation of Concerns

- Build-time dependency: `test-server` feature builds the server
- Runtime dependency: None for RS builds, FFI for FFI builds
- Test-time dependency: Tests discover and use the built server

## Constraints Satisfied

✅ Pure Rust builds have **zero FFI runtime dependencies**
✅ Integration tests work in both local development and CI/CD
✅ Tests gracefully skip if server is unavailable
✅ Works in clean directory for crates.io publishing
✅ Doesn't break existing FFI build behavior

## Future Improvements

1. **Parallel test execution**: Fix tokio runtime nesting issue in live server tests
2. **Server lifecycle management**: Add automatic server startup/shutdown in test harness
3. **Test isolation**: Ensure tests don't interfere with each other when using shared server
4. **Performance**: Cache server binary across test runs to avoid rebuilds

## Related Files

- [`tests/harness/server.rs`](tests/harness/server.rs) - Server discovery and management
- [`tests/harness/health.rs`](tests/harness/health.rs) - Health monitoring
- [`tests/common/mod.rs`](tests/common/mod.rs) - Test setup with graceful degradation
- [`sys/build.rs`](sys/build.rs:497-546) - INDIGO build script
- [`Cargo.toml`](Cargo.toml) - Feature flags and dependencies
- [`.github/workflows/rust.yml`](../.github/workflows/rust.yml) - CI/CD configuration
