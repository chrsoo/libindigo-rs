# GitHub Actions Workflows

## Overview

This directory contains GitHub Actions workflows for continuous integration and testing of the libindigo-rs project.

## Workflows

### `rust.yml` - Main CI/CD Pipeline

The main workflow runs on every push and pull request to `main`/`master` branches.

#### Jobs

1. **test-rs-strategy** (Primary, ~3-5 min)
   - Tests pure Rust implementation
   - No system dependencies required
   - Fast feedback for PRs
   - Runs unit and integration tests

2. **build-ffi-strategy** (Secondary, ~10-20 min)
   - Builds FFI bindings to C library
   - Requires system dependencies
   - Verifies C library integration
   - Uses caching to speed up builds

3. **build-minimal** (~2 min)
   - Verifies core library builds without optional features
   - Ensures minimal dependency footprint

4. **docs** (~3-5 min)
   - Builds documentation
   - Ensures doc comments are valid

5. **check-rust-versions** (~2-3 min each)
   - Tests against stable, beta, and nightly Rust
   - Ensures forward compatibility

## Running Locally

### Quick Test (Pure Rust)

```bash
cargo test --features rs-strategy --lib
```

### Full Build (FFI)

```bash
git submodule update --init --recursive
cargo build --workspace
```

### Integration Tests

```bash
# Without server (tests will skip gracefully)
INDIGO_TEST_SKIP_SERVER=true cargo test --features rs-strategy

# With server (requires INDIGO server running)
cargo test --features rs-strategy
```

## Configuration

### Environment Variables

- `INDIGO_TEST_SKIP_SERVER=true` - Skip server startup in tests
- `INDIGO_SOURCE` - Path to INDIGO source for FFI builds
- `CARGO_TERM_COLOR=always` - Enable colored output

### Caching

The workflow caches:

- INDIGO C library build artifacts
- Rust build artifacts (via actions-rust-lang/setup-rust-toolchain)

Cache key: `${{ runner.os }}-indigo-${{ hashFiles('sys/externals/indigo/**') }}`

## Troubleshooting

### Tests Timeout

- Ensure `INDIGO_TEST_SKIP_SERVER=true` is set
- Check test timeout values (should be 5-10 seconds)

### FFI Build Fails

- Verify system dependencies are installed
- Check git submodule is initialized
- Review build logs for specific errors

### Cache Issues

- Cache may be invalidated if INDIGO submodule changes
- GitHub Actions has 10GB cache limit per repository

## See Also

- [CI/CD Strategy Documentation](../../CI_CD_STRATEGY.md) - Comprehensive CI/CD strategy
- [Test Harness Documentation](../../tests/HARNESS_IMPLEMENTATION.md) - Test infrastructure
- [Pure Rust Tests README](../../tests/README_PURE_RUST_TESTS.md) - Test suite overview
