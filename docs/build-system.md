# Build System Architecture

This document describes the build system architecture for the libindigo workspace, including constant extraction, version management, and FFI compilation.

## Overview

The libindigo workspace uses a multi-crate build system designed to:

1. **Extract constants** from INDIGO source code without compiling it
2. **Extract version information** from INDIGO Makefile
3. **Compile FFI bindings** only when needed (for `libindigo-sys` and `libindigo-ffi`)
4. **Keep pure Rust builds fast** by avoiding unnecessary compilation

## Build Utilities Crate

The `build_utils` crate provides shared functionality for all build scripts:

### Location

- **Path**: `build_utils/`
- **Name**: `libindigo-build-utils`
- **Purpose**: Shared build-time utilities

### Functions

#### `parse_indigo_version(indigo_root: &Path) -> Result<Version>`

Extracts INDIGO version from the Makefile:

- Reads `INDIGO_VERSION` (e.g., "2.0")
- Reads `INDIGO_BUILD` (e.g., "358")
- Returns as `semver::Version` (e.g., 2.0.358)

#### `generate_version_constants(version: &Version) -> String`

Generates Rust constants for INDIGO version:

```rust
pub const INDIGO_VERSION_MAJOR: u32 = 2;
pub const INDIGO_VERSION_MINOR: u32 = 0;
pub const INDIGO_BUILD: u32 = 358;
pub const INDIGO_VERSION: &str = "2.0.358";
```

#### `format_indigo_build_metadata(version: &Version) -> String`

Formats version as SemVer build metadata:

- Input: `Version { major: 2, minor: 0, patch: 358 }`
- Output: `"INDIGO.2.0.358"`

## Build Scripts

### Root `build.rs` (libindigo crate)

**Purpose**: Extract constants and version information WITHOUT compiling INDIGO

**Process**:

1. Check if INDIGO submodule exists at `sys/externals/indigo`
2. Extract INDIGO version from Makefile
3. Generate `version.rs` with version constants
4. Extract property/item name constants from `indigo_names.h`
5. Generate `src/constants.rs` with name constants
6. For FFI builds: Generate interface bindings

**Key Features**:

- ✅ Fast builds for pure Rust strategy
- ✅ No C compilation required
- ✅ Constants extracted via regex parsing
- ✅ Version information available at compile time

**Generated Files**:

- `$OUT_DIR/version.rs` - Version constants
- `src/constants.rs` - Property/item name constants
- `$OUT_DIR/interface.rs` - Interface enum (FFI builds only)

### `sys/build.rs` (libindigo-sys crate)

**Purpose**: Compile INDIGO C library and generate FFI bindings

**Process**:

1. Determine INDIGO source location:
   - `$INDIGO_SOURCE` environment variable (if set)
   - `externals/indigo` submodule
   - System libraries at `/usr/include/indigo` (Linux only)
   - Clone from GitHub as fallback
2. Compile INDIGO using `make`
3. Extract version information
4. Generate `version.rs` with version constants
5. Generate FFI bindings with `bindgen`

**Key Features**:

- ✅ Supports multiple INDIGO source locations
- ✅ Shallow git clone for faster builds
- ✅ Version extraction from source
- ✅ Comprehensive FFI bindings

**Generated Files**:

- `$OUT_DIR/bindings.rs` - Complete FFI bindings
- `$OUT_DIR/version.rs` - Version constants

## Version Management

### SemVer Build Metadata

The workspace uses SemVer build metadata to track INDIGO versions:

```toml
# libindigo and libindigo-sys
version = "0.3.2+INDIGO.2.0.358"

# libindigo-rs and libindigo-ffi (no build metadata)
version = "0.3.2"
```

**Rationale**:

- Core crates (`libindigo`, `libindigo-sys`) include INDIGO version in metadata
- Implementation crates (`libindigo-rs`, `libindigo-ffi`) use base version only
- Build metadata doesn't affect dependency resolution

### Automatic Version Updates

Use the provided script to update versions:

```bash
./scripts/update_version.sh [base_version]
```

**What it does**:

1. Extracts INDIGO version from `sys/externals/indigo/Makefile`
2. Updates `Cargo.toml` files with correct versions
3. Maintains consistency across workspace

**Example**:

```bash
# Update to new base version 0.4.0
./scripts/update_version.sh 0.4.0

# Use current base version
./scripts/update_version.sh
```

## Build Strategies

### Pure Rust Build (Fast)

```bash
cargo build -p libindigo --features rs-strategy
```

**What happens**:

- ✅ Extracts constants from INDIGO headers (fast regex parsing)
- ✅ Extracts version from Makefile
- ✅ No C compilation
- ✅ No FFI bindings generation
- ⚡ Build time: ~3 seconds

### FFI Build (Slower)

```bash
cargo build -p libindigo-sys
cargo build -p libindigo-ffi
```

**What happens**:

- ✅ Compiles INDIGO C library with `make`
- ✅ Generates FFI bindings with `bindgen`
- ✅ Extracts version and constants
- ⏱️ Build time: ~15-30 seconds (first build)
- ⚡ Cached builds: ~3 seconds

## INDIGO Source Locations

The build system supports multiple INDIGO source locations (in priority order):

### 1. Environment Variable

```bash
export INDIGO_SOURCE=/path/to/indigo
cargo build -p libindigo-sys
```

**Use case**: Development with custom INDIGO version

### 2. Git Submodule (Default)

```bash
git submodule update --init --recursive
cargo build
```

**Use case**: Normal development and CI/CD

### 3. System Libraries (Linux only)

Automatically detected if `/usr/include/indigo/indigo_version.h` exists.

**Use case**: System-wide INDIGO installation

### 4. Automatic Clone

If no source is found, the build script clones from GitHub:

```
https://github.com/indigo-astronomy/indigo
```

**Use case**: Building from crates.io

## Constant Extraction

### Property and Item Names

Constants are extracted from `indigo_names.h`:

```c
#define CONNECTION_PROPERTY_NAME "CONNECTION"
#define CONNECTED_ITEM_NAME "CONNECTED"
```

Converted to Rust:

```rust
pub const CONNECTION_PROPERTY: &str = "CONNECTION";
pub const CONNECTED_ITEM: &str = "CONNECTED";
```

**Process**:

1. Parse header with regex: `#define (\w+)_NAME "(.+)"`
2. Remove `_NAME` suffix
3. Generate Rust constants
4. Sort alphabetically
5. Write to `src/constants.rs`

### Version Constants

Version is extracted from Makefile:

```makefile
INDIGO_VERSION = 2.0
INDIGO_BUILD = 358
```

Converted to Rust:

```rust
pub const INDIGO_VERSION_MAJOR: u32 = 2;
pub const INDIGO_VERSION_MINOR: u32 = 0;
pub const INDIGO_BUILD: u32 = 358;
pub const INDIGO_VERSION: &str = "2.0.358";
```

## Troubleshooting

### Constants not updating

```bash
# Force rebuild
cargo clean
cargo build
```

### Version mismatch

```bash
# Update submodule
git submodule update --remote sys/externals/indigo

# Update versions
./scripts/update_version.sh

# Rebuild
cargo clean && cargo build
```

### FFI build fails

```bash
# Check INDIGO source
ls sys/externals/indigo/Makefile

# Initialize submodule
git submodule update --init --recursive --depth 1

# Try manual build
cd sys/externals/indigo && make clean && make
```

## CI/CD Integration

### GitHub Actions Example

```yaml
- name: Initialize INDIGO submodule
  run: git submodule update --init --recursive --depth 1

- name: Build pure Rust
  run: cargo build -p libindigo --features rs-strategy

- name: Build FFI
  run: cargo build -p libindigo-sys
```

### Caching

Cache the INDIGO build directory:

```yaml
- uses: actions/cache@v3
  with:
    path: |
      sys/externals/indigo/build
      target
    key: ${{ runner.os }}-indigo-${{ hashFiles('sys/externals/indigo/Makefile') }}
```

## Related Issues

This build system addresses the following GitHub issues:

- **#39**: Integration Tests Require Running INDIGO Server
- **#40**: Simplify FFI Build Strategy in CI/CD
- **#44**: Populate Device Metadata Fields
- **#51**: Manual Version Bumping for Workspace Modules
- **#55**: Maintain src/constants.rs

## See Also

- [Cargo Workspace Documentation](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [SemVer Build Metadata](https://semver.org/#spec-item-10)
- [bindgen User Guide](https://rust-lang.github.io/rust-bindgen/)
