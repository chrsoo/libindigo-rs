# Building libindigo

This document provides quick-start build instructions. For detailed information, see [`docs/build-system.md`](docs/build-system.md).

## Quick Start

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- Git with submodules support
- For FFI builds: C compiler, make, INDIGO dependencies

### Clone and Build

```bash
# Clone repository with submodules
git clone --recursive https://github.com/chrsoo/libindigo-rs
cd libindigo-rs

# Build core crate (strategy-agnostic)
cargo build -p libindigo

# Build pure Rust client (fast, ~3 seconds)
cargo build -p libindigo-rs

# Build FFI client (slower, ~30 seconds first time)
cargo build -p libindigo-ffi

# Build entire workspace
cargo build --workspace --exclude relm
```

## Build Strategies

### Pure Rust Client (Recommended for Development)

Fast builds without C compilation:

```bash
cargo build -p libindigo-rs
```

**Benefits:**

- ⚡ Fast builds (~3 seconds)
- ✅ No C compiler required
- ✅ Cross-platform
- ✅ Easy to debug

### FFI Client (For Production)

Uses C INDIGO library:

```bash
cargo build -p libindigo-sys
cargo build -p libindigo-ffi --features async
```

**Benefits:**

- ✅ Full INDIGO compatibility
- ✅ Access to all INDIGO features
- ✅ Battle-tested C implementation

## INDIGO Source

The build system supports multiple INDIGO source locations:

### 1. Git Submodule (Default)

```bash
git submodule update --init --recursive
cargo build
```

### 2. Custom Location

```bash
export INDIGO_SOURCE=/path/to/indigo
cargo build -p libindigo-sys
```

### 3. System Libraries (Linux)

If `/usr/include/indigo/indigo_version.h` exists, system libraries are used automatically.

## Version Management

### Check INDIGO Version

The build system extracts the INDIGO version at build time:

```rust
use libindigo::version;
println!("INDIGO version: {}", version::INDIGO_VERSION);
// Output: INDIGO version: 2.0.358
```

### Update Versions

When updating the INDIGO submodule:

```bash
# Update INDIGO submodule
git submodule update --remote sys/externals/indigo

# Update workspace versions
./scripts/update_version.sh

# Rebuild
cargo clean && cargo build
```

## Common Issues

### Submodule Not Initialized

```bash
git submodule update --init --recursive --depth 1
```

### Build Fails on macOS

Install Xcode Command Line Tools:

```bash
xcode-select --install
```

### Build Fails on Linux

Install INDIGO dependencies:

```bash
sudo apt-get install build-essential libudev-dev libusb-1.0-0-dev
```

## Testing

```bash
# Run all tests
cargo test --workspace --exclude relm

# Run specific crate tests
cargo test -p libindigo
cargo test -p libindigo-rs
cargo test -p libindigo-sys
```

## Documentation

```bash
# Build and open documentation
cargo doc --open --no-deps
```

## CI/CD

For CI/CD pipelines, use shallow clones:

```bash
git submodule update --init --recursive --depth 1
cargo build --workspace --exclude relm
```

## More Information

- **Build System**: [`docs/build-system.md`](docs/build-system.md)
- **Architecture**: [`docs/architecture/`](docs/architecture/)
- **Examples**: [`examples/`](examples/)
- **Issues**: [`plans/issues.md`](plans/issues.md)

## Getting Help

- GitHub Issues: <https://github.com/chrsoo/libindigo-rs/issues>
- INDIGO Documentation: <https://www.indigo-astronomy.org/>
