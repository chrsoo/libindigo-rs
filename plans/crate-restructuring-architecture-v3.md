# Crate Restructuring Architecture Plan v3 (Final)

## Executive Summary

This document proposes a restructuring of the libindigo project to achieve:

1. **`libindigo`** remains the main crate in the root directory
2. **`sys/`** subdirectory contains `libindigo-sys` crate (FFI strategy)
3. **`rs/`** subdirectory contains `libindigo-rs` crate (pure Rust strategy)
4. **Feature-based selection** between `rs` and `ffi` strategies
5. **Zero FFI dependencies** when using `rs` feature

## Directory Structure

```
libindigo/                     # Root = libindigo crate
├── Cargo.toml                 # [package] name = "libindigo"
├── build.rs
├── src/
│   ├── lib.rs
│   ├── constants.rs           # Pre-generated INDIGO constants
│   ├── error.rs
│   ├── client/
│   │   ├── mod.rs
│   │   ├── builder.rs
│   │   ├── client.rs
│   │   └── strategy.rs        # ClientStrategy trait (SPI)
│   ├── types/
│   │   ├── property.rs
│   │   ├── value.rs
│   │   └── device.rs
│   └── device/                # Future: Device API
│       └── mod.rs
├── sys/                       # libindigo-sys crate
│   ├── Cargo.toml             # [package] name = "libindigo-sys"
│   ├── build.rs               # Builds INDIGO C library
│   ├── src/
│   │   ├── lib.rs
│   │   ├── strategy.rs        # FfiClientStrategy
│   │   ├── async_strategy.rs  # AsyncFfiStrategy
│   │   └── bindings.rs        # Generated FFI bindings
│   └── externals/
│       └── indigo/            # Git submodule
├── rs/                        # libindigo-rs crate (NEW)
│   ├── Cargo.toml             # [package] name = "libindigo-rs"
│   ├── src/
│   │   ├── lib.rs
│   │   ├── strategy.rs        # RsClientStrategy
│   │   ├── transport.rs
│   │   ├── protocol.rs
│   │   ├── protocol_json.rs
│   │   └── protocol_negotiation.rs
│   └── tests/
├── relm/                      # GUI application (unchanged)
│   └── Cargo.toml
└── tests/                     # Integration tests
```

## Workspace Configuration

**Root `Cargo.toml`**:

```toml
[workspace]
members = [".", "sys", "rs", "relm"]
resolver = "2"

[package]
name = "libindigo"
version = "0.2.0"
edition = "2021"
# ... metadata

[dependencies]
# Core dependencies (always present)
serde = { version = "1.0", default-features = false }
tokio = { version = "1.35", features = ["rt-multi-thread", "sync", "macros", "time", "net", "io-util"], optional = true }
async-trait = "0.1"
thiserror = "1.0"
chrono = "0.4"
# ... other core deps

# Strategy implementations (optional, path dependencies)
libindigo-rs = { version = "0.2", path = "rs", optional = true }
libindigo-sys = { version = "0.2", path = "sys", optional = true }

[features]
default = ["client", "rs"]

# API features
client = ["tokio"]
device = []  # Future: Device API

# Strategy features
rs = ["libindigo-rs"]
ffi = ["libindigo-sys"]

# Other features
async = ["tokio"]
std = []
```

## Crate Details

### 1. `libindigo` (Root Crate)

**Location**: Root directory
**Cargo.toml**: `./Cargo.toml` with `[package] name = "libindigo"`

**Purpose**:

- Client API (`Client`, `ClientBuilder`)
- `ClientStrategy` trait (SPI)
- Core types (`Property`, `PropertyType`, etc.)
- INDIGO constants (pre-generated)
- Error types

**Key Files**:

- `src/constants.rs` - Pre-generated from `props.rs`
- `src/client/strategy.rs` - `ClientStrategy` trait definition
- `src/client/builder.rs` - Strategy selection logic

**Build Script** (`build.rs`):

```rust
fn main() -> std::io::Result<()> {
    // No generation needed - constants are pre-generated in src/constants.rs
    println!("cargo:rerun-if-changed=src/constants.rs");
    Ok(())
}
```

**Strategy Selection** (`src/client/builder.rs`):

```rust
impl ClientBuilder {
    /// Use pure Rust strategy (requires 'rs' feature)
    #[cfg(feature = "rs")]
    pub fn with_rs_strategy(mut self) -> Self {
        self.strategy = Some(Box::new(libindigo_rs::RsClientStrategy::new()));
        self
    }

    /// Use FFI strategy (requires 'ffi' feature)
    #[cfg(feature = "ffi")]
    pub fn with_ffi_strategy(mut self) -> Self {
        self.strategy = Some(Box::new(libindigo_sys::FfiClientStrategy::new()));
        self
    }

    /// Use custom strategy (always available)
    pub fn with_strategy(mut self, strategy: Box<dyn ClientStrategy>) -> Self {
        self.strategy = Some(strategy);
        self
    }

    pub fn build(self) -> Result<Client> {
        let strategy = self.strategy.ok_or_else(|| {
            IndigoError::Configuration(
                "No ClientStrategy provided. Either:\n\
                 1. Enable 'rs' feature: libindigo = { version = \"0.2\", features = [\"rs\"] }\n\
                 2. Enable 'ffi' feature: libindigo = { version = \"0.2\", features = [\"ffi\"] }\n\
                 3. Provide custom strategy: builder.with_strategy(my_strategy)\n\
                 4. Add external SPI crate as dependency".to_string()
            )
        })?;

        Ok(Client::new(strategy))
    }
}

// Auto-select strategy if only one feature is enabled
impl Default for ClientBuilder {
    fn default() -> Self {
        let mut builder = Self::new();

        #[cfg(all(feature = "rs", not(feature = "ffi")))]
        {
            builder = builder.with_rs_strategy();
        }

        #[cfg(all(feature = "ffi", not(feature = "rs")))]
        {
            builder = builder.with_ffi_strategy();
        }

        builder
    }
}
```

### 2. `libindigo-sys` (FFI Strategy)

**Location**: `sys/` subdirectory
**Cargo.toml**: `sys/Cargo.toml` with `[package] name = "libindigo-sys"`

**Purpose**: FFI-based implementation using upstream INDIGO C library

**Dependencies** (`sys/Cargo.toml`):

```toml
[package]
name = "libindigo-sys"
version = "0.2.0"
edition = "2021"

[dependencies]
libindigo = { version = "0.2", path = "..", default-features = false, features = ["client"] }
tokio = { version = "1.35", features = ["rt-multi-thread", "sync", "macros"], optional = true }
async-trait = "0.1"
# ... FFI-related deps

[build-dependencies]
bindgen = "0.71"
cc = "1.0"
# ... build deps

[features]
default = []
async = ["tokio"]
```

**Build Script** (`sys/build.rs`):

- Handles INDIGO C library compilation
- Git submodule detection and initialization
- Generates FFI bindings
- Links against compiled C library

**Key Point**: This is the **existing** `sys/` directory, just with updated `Cargo.toml` to:

1. Set `name = "libindigo-sys"`
2. Depend on parent `libindigo` crate
3. Implement `libindigo::ClientStrategy`

### 3. `libindigo-rs` (Pure Rust Strategy)

**Location**: `rs/` subdirectory (NEW)
**Cargo.toml**: `rs/Cargo.toml` with `[package] name = "libindigo-rs"`

**Purpose**: Pure Rust implementation with zero FFI dependencies

**Dependencies** (`rs/Cargo.toml`):

```toml
[package]
name = "libindigo-rs"
version = "0.2.0"
edition = "2021"

[dependencies]
libindigo = { version = "0.2", path = "..", default-features = false, features = ["client"] }
tokio = { version = "1.35", features = ["rt-multi-thread", "sync", "macros", "time", "net", "io-util"] }
quick-xml = "0.31"
serde_json = "1.0"
base64 = "0.21"
async-trait = "0.1"
# ... other pure Rust deps (NO FFI)

[features]
default = ["xml", "json"]
xml = ["quick-xml"]
json = ["serde_json"]
```

**Key Files**:

- `rs/src/strategy.rs` - `RsClientStrategy` implementation
- `rs/src/transport.rs` - TCP transport layer
- `rs/src/protocol.rs` - XML protocol parser
- `rs/src/protocol_json.rs` - JSON protocol parser
- `rs/src/protocol_negotiation.rs` - Protocol negotiation

**Source**: Move from `src/strategies/rs/` to `rs/src/`

## Migration Steps

### Phase 1: Create `rs/` Subdirectory (Day 1)

**Step 1.1**: Create directory structure

```bash
mkdir -p rs/src
mkdir -p rs/tests
```

**Step 1.2**: Create `rs/Cargo.toml`

```toml
[package]
name = "libindigo-rs"
version = "0.2.0"
edition = "2021"
description = "Pure Rust INDIGO client strategy"
license = "MIT"

[dependencies]
libindigo = { version = "0.2", path = "..", default-features = false, features = ["client"] }
tokio = { version = "1.35", features = ["rt-multi-thread", "sync", "macros", "time", "net", "io-util"] }
quick-xml = "0.31"
serde_json = "1.0"
base64 = "0.21"
async-trait = "0.1"
thiserror = "1.0"
tracing = "0.1"
futures = "0.3"

[features]
default = ["xml", "json"]
xml = ["quick-xml"]
json = ["serde_json"]
```

**Step 1.3**: Move strategy code

```bash
# Move pure Rust strategy implementation
cp -r src/strategies/rs/* rs/src/
# Or: mv src/strategies/rs/* rs/src/

# Update imports in rs/src/*.rs
# Change: use crate::strategies::rs::*
# To: use libindigo::*
```

**Step 1.4**: Update `rs/src/lib.rs`

```rust
//! Pure Rust INDIGO client strategy implementation

mod strategy;
mod transport;
mod protocol;
mod protocol_json;
mod protocol_negotiation;

pub use strategy::RsClientStrategy;
```

**Step 1.5**: Update workspace `Cargo.toml`

```toml
[workspace]
members = [".", "sys", "rs", "relm"]  # Add "rs"
```

### Phase 2: Update `sys/` Crate (Day 1)

**Step 2.1**: Update `sys/Cargo.toml`

```toml
[package]
name = "libindigo-sys"  # Ensure this is set
version = "0.2.0"
edition = "2021"

[dependencies]
libindigo = { version = "0.2", path = "..", default-features = false, features = ["client"] }
# ... rest of dependencies
```

**Step 2.2**: Update `sys/src/lib.rs`

```rust
//! FFI-based INDIGO client strategy implementation

mod strategy;
mod async_strategy;

pub use strategy::FfiClientStrategy;
pub use async_strategy::AsyncFfiStrategy;

// Re-export FFI bindings if needed
pub use bindings::*;
```

### Phase 3: Update Root `libindigo` Crate (Day 2)

**Step 3.1**: Copy constants to source

```bash
cp props.rs src/constants.rs
```

**Step 3.2**: Update `src/lib.rs`

```rust
// Include pre-generated constants
pub mod constants {
    include!("constants.rs");
}

// Re-export for convenience
pub use constants as name;

// Conditional compilation for interface bindings (FFI only)
#[cfg(any(feature = "ffi", feature = "sys"))]
include!(concat!(env!("OUT_DIR"), "/interface.rs"));
```

**Step 3.3**: Update `build.rs`

```rust
fn main() -> std::io::Result<()> {
    // Only generate interface bindings for FFI builds
    let has_ffi = std::env::var("CARGO_FEATURE_FFI").is_ok()
        || std::env::var("CARGO_FEATURE_SYS").is_ok();

    if !has_ffi {
        // For pure Rust builds, create empty interface file
        let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
        let interface_path = out_dir.join("interface.rs");
        std::fs::write(interface_path, "// No INDIGO interface for pure Rust build\n")?;
        return Ok(());
    }

    // For FFI builds, we don't generate anything here
    // The sys/ crate handles all FFI generation
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let interface_path = out_dir.join("interface.rs");
    std::fs::write(interface_path, "// Interface bindings in libindigo-sys\n")?;

    Ok(())
}
```

**Step 3.4**: Update root `Cargo.toml`

```toml
[dependencies]
# Add strategy dependencies
libindigo-rs = { version = "0.2", path = "rs", optional = true }
libindigo-sys = { version = "0.2", path = "sys", optional = true }

[features]
default = ["client", "rs"]
client = ["tokio"]
device = []
rs = ["libindigo-rs"]
ffi = ["libindigo-sys"]
# Remove "auto" from default if it causes issues
```

**Step 3.5**: Update `src/client/builder.rs`

- Add `with_rs_strategy()` method
- Add `with_ffi_strategy()` method
- Update `build()` error handling
- Update `Default` impl

### Phase 4: Update CI/CD (Day 2-3)

**Step 4.1**: Update `.github/workflows/rust.yml`

```yaml
jobs:
  test-rs:
    name: Test Pure Rust Strategy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      # No submodule checkout!

      - name: Install Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Install avahi
        run: sudo apt-get install -y libavahi-client-dev

      - name: Test libindigo with rs feature
        run: cargo test --no-default-features --features client,rs

      - name: Test libindigo-rs crate
        run: cargo test -p libindigo-rs

  test-ffi:
    name: Test FFI Strategy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive

      - name: Install Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Install system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            build-essential \
            libudev-dev \
            libusb-1.0-0-dev \
            libavahi-client-dev \
            libglib2.0-dev \
            libgobject-2.0-dev

      - name: Test libindigo with ffi feature
        run: cargo test --no-default-features --features client,ffi

      - name: Test libindigo-sys crate
        run: cargo test -p libindigo-sys
```

### Phase 5: Testing (Day 3)

**Step 5.1**: Test pure Rust build

```bash
# Should work without INDIGO submodule
cargo build --no-default-features --features client,rs
cargo test --no-default-features --features client,rs
```

**Step 5.2**: Test FFI build

```bash
# Requires INDIGO submodule
git submodule update --init --recursive
cargo build --no-default-features --features client,ffi
cargo test --no-default-features --features client,ffi
```

**Step 5.3**: Test no strategy (should fail gracefully)

```bash
cargo build --no-default-features --features client
# Should compile successfully

# But running should give error:
cargo run --example client_example
# Error: No ClientStrategy provided...
```

**Step 5.4**: Test workspace build

```bash
# Should build everything
cargo build --workspace
```

## Usage Examples

### Example 1: Pure Rust Client (Default)

```rust
// Cargo.toml
[dependencies]
libindigo = "0.2"  # defaults to ["client", "rs"]

// main.rs
use libindigo::{Client, ClientBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClientBuilder::default().build()?;
    client.connect("localhost:7624").await?;
    Ok(())
}
```

### Example 2: FFI Client

```rust
// Cargo.toml
[dependencies]
libindigo = { version = "0.2", default-features = false, features = ["client", "ffi"] }

// main.rs
use libindigo::{Client, ClientBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClientBuilder::default().build()?;
    client.connect("localhost:7624").await?;
    Ok(())
}
```

### Example 3: Explicit Strategy Selection

```rust
// Cargo.toml
[dependencies]
libindigo = { version = "0.2", features = ["rs", "ffi"] }

// main.rs
use libindigo::{Client, ClientBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Explicitly choose RS
    let client = ClientBuilder::new()
        .with_rs_strategy()
        .build()?;

    // Or explicitly choose FFI
    let client = ClientBuilder::new()
        .with_ffi_strategy()
        .build()?;

    client.connect("localhost:7624").await?;
    Ok(())
}
```

## Benefits

✅ **Clean directory structure** - Root is main crate, subdirs are strategy crates
✅ **Workspace members** - `sys/` and `rs/` are proper workspace members
✅ **Crate names in Cargo.toml** - Each crate defines its own name
✅ **Zero FFI for pure Rust** - `rs` feature has no C dependencies
✅ **Feature-based selection** - Simple, idiomatic Rust
✅ **Clear error messages** - Missing strategy gives helpful guidance
✅ **Backward compatible paths** - `sys/` stays where it is

## Timeline

- **Day 1**: Create `rs/` directory, update `sys/` crate name
- **Day 2**: Update root crate, move constants, update features
- **Day 3**: Update CI/CD, test all configurations
- **Total**: 3 days for complete migration

## Next Steps

1. **Create `rs/` directory** and move pure Rust strategy code
2. **Update `sys/Cargo.toml`** to set `name = "libindigo-sys"`
3. **Copy `props.rs`** to `src/constants.rs`
4. **Update root `Cargo.toml`** with features and dependencies
5. **Update `src/client/builder.rs`** with strategy selection
6. **Test locally** before pushing
7. **Update CI/CD** workflows
8. **Commit and push**

---

**Document Version**: 3.0 (Final)
**Date**: 2026-03-04
**Author**: Architecture Planning
**Status**: Proposed (Final revision per directory structure requirements)
