# Immediate CI/CD Fix Plan

## Context

While the [crate restructuring architecture](./crate-restructuring-architecture.md) is the long-term solution, we need immediate fixes to get CI/CD passing.

## Current CI/CD Status

**Passing (5/7)**:

- ✅ Build Minimal (No Optional Features)
- ✅ Build Documentation
- ✅ Check Rust beta
- ✅ Check Rust nightly
- ✅ Build Minimal (with warnings)

**Failing (2/7)**:

- ❌ Test Pure Rust Strategy - Missing `INFO_PROPERTY` constant
- ❌ Build FFI Strategy - glib-sys build failure

## Immediate Fix: Option A (Quick & Dirty)

### Fix 1: Embed Constants in Source Code

Instead of generating constants at build time, embed them directly in source:

**File**: `src/constants.rs` (new file)

```rust
// INDIGO Protocol Constants
// Generated from INDIGO 2.0.300
// Source: indigo_names.h

pub const INFO_PROPERTY: &str = "INFO";
pub const INFO_DEVICE_INTERFACE_ITEM: &str = "DEVICE_INTERFACE";
pub const CONNECTION_PROPERTY: &str = "CONNECTION";
// ... copy all constants from props.rs
```

**File**: `src/lib.rs` (modify)

```rust
// Replace this:
pub mod name {
    include!(concat!(env!("OUT_DIR"), "/name.rs"));
}

// With this:
#[cfg(any(feature = "ffi", feature = "sys", feature = "auto"))]
pub mod name {
    include!(concat!(env!("OUT_DIR"), "/name.rs"));
}

#[cfg(not(any(feature = "ffi", feature = "sys", feature = "auto")))]
pub mod name {
    include!("constants.rs");
}
```

**File**: `build.rs` (already fixed)

- Already conditional on FFI features ✅

### Fix 2: Remove `auto` from Default Features

**File**: `Cargo.toml`

```toml
# Change this:
default = ["async", "ffi", "sys", "std", "auto"]

# To this:
default = ["async", "ffi", "sys", "std"]
# auto = ["zeroconf"]  # Keep as optional feature
```

This removes the glib/gobject dependency from default builds.

### Fix 3: Install glib-dev in FFI Workflow

**File**: `.github/workflows/rust.yml`

```yaml
- name: Install system dependencies
  run: |
    sudo apt-get update
    sudo apt-get install -y \
      build-essential \
      autoconf \
      libtool \
      cmake \
      libudev-dev \
      libavahi-compat-libdnssd-dev \
      libavahi-client-dev \
      libglib2.0-dev \        # ADD THIS
      libgobject-2.0-dev \    # ADD THIS
      libusb-1.0-0-dev \
      libcurl4-gnutls-dev \
      libgphoto2-dev \
      libz-dev \
      git \
      curl \
      patchelf
```

## Immediate Fix: Option B (Cleaner)

### Use Pre-generated Constants for All Builds

**Step 1**: Copy `props.rs` → `src/constants.rs`

```bash
cp props.rs src/constants.rs
```

**Step 2**: Update `src/lib.rs`

```rust
// Remove conditional compilation
pub mod name {
    include!("constants.rs");
}

// Remove this:
// include!(concat!(env!("OUT_DIR"), "/interface.rs"));
```

**Step 3**: Update `build.rs`

```rust
fn main() -> std::io::Result<()> {
    // Only generate interface bindings for FFI builds
    let has_ffi = env::var("CARGO_FEATURE_FFI_STRATEGY").is_ok()
        || env::var("CARGO_FEATURE_SYS").is_ok();

    if !has_ffi {
        // For pure Rust builds, create empty interface file
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let interface_path = out_dir.join("interface.rs");
        std::fs::write(interface_path, "// No INDIGO interface for pure Rust build\n")?;
        return Ok(());
    }

    // FFI build logic (existing code)
    // ...
}
```

**Step 4**: Update `src/lib.rs` for interface

```rust
#[cfg(any(feature = "ffi", feature = "sys"))]
include!(concat!(env!("OUT_DIR"), "/interface.rs"));
```

**Step 5**: Remove `auto` from default features (same as Option A)

## Recommended Approach: Option B

**Pros**:

- ✅ Simpler - constants always available
- ✅ No conditional compilation complexity
- ✅ Works for both FFI and pure Rust
- ✅ Easy to update constants (just edit `src/constants.rs`)
- ✅ Aligns with long-term architecture plan

**Cons**:

- ⚠️ Constants might drift from INDIGO headers
- ⚠️ Need manual update when INDIGO releases new version

**Mitigation**:

- Add CI job to regenerate and compare constants
- Document update process
- Version constants with INDIGO version

## Implementation Steps (Option B)

### Step 1: Create Constants File

```bash
cp props.rs src/constants.rs
git add src/constants.rs
```

### Step 2: Update src/lib.rs

```rust
// Around line 183-195
pub mod name {
    include!("constants.rs");
}

// Around line 181 - make conditional
#[cfg(any(feature = "ffi", feature = "sys"))]
include!(concat!(env!("OUT_DIR"), "/interface.rs"));
```

### Step 3: Update build.rs

Already done in commit `6606589` ✅

### Step 4: Update Cargo.toml

```toml
[features]
default = ["async", "ffi", "sys", "std"]  # Remove "auto"
```

### Step 5: Update CI Workflow

Add glib development packages to FFI build job.

### Step 6: Test Locally

```bash
# Test pure Rust build
cargo build --no-default-features --features rs

# Test FFI build
cargo build --features ffi

# Test default build
cargo build
```

### Step 7: Commit and Push

```bash
git add -A
git commit -m "fix: Use pre-generated constants for all builds

- Copy props.rs to src/constants.rs
- Make interface.rs generation conditional on FFI features
- Remove auto from default features to avoid glib dependency
- Update CI to install glib-dev for FFI builds"
git push
```

## Expected Results

After implementing Option B:

**Pure Rust Strategy**:

- ✅ Builds without INDIGO submodule
- ✅ No glib/gobject dependencies
- ✅ Constants available from `src/constants.rs`
- ✅ Fast compilation

**FFI Strategy**:

- ✅ Builds with INDIGO submodule
- ✅ Interface bindings generated
- ✅ Constants available from `src/constants.rs`
- ✅ Full C library integration

**CI/CD**:

- ✅ All 7 jobs should pass
- ✅ Pure Rust jobs fast (no C compilation)
- ✅ FFI jobs comprehensive (full build)

## Timeline

- **Immediate**: 1-2 hours to implement
- **Testing**: 1 hour local testing
- **CI/CD**: 10-15 minutes per run
- **Total**: ~4 hours to complete and verify

## Long-term Path

This immediate fix is compatible with the long-term restructuring plan:

1. **Now**: Use `src/constants.rs` in monolithic crate
2. **Phase 1**: Move `src/constants.rs` → `indigo-core/src/constants.rs`
3. **Phase 2+**: Continue with crate restructuring

No throwaway work - the constants file will be reused in the new architecture.

---

**Recommendation**: Implement Option B immediately to unblock CI/CD, then proceed with full restructuring when ready.
