# Troubleshooting Guide

## Common Issues

### "unresolved module or unlinked crate `libindigo`" Error

**Error Message:**

```
error[E0432]: unresolved import `libindigo`
  --> src/main.rs:1:5
   |
1  | use libindigo::{Client, ClientBuilder};
   |     ^^^^^^^^^ use of undeclared crate or module `libindigo`
```

**Root Cause:**

This is the most common issue when using `libindigo-rs`. The crate is named `libindigo-rs` (with hyphen) in Cargo.toml, but Rust automatically converts hyphens to underscores in crate names for imports.

**Solution:**

Change your import statements from `libindigo::` to `libindigo_rs::` (with underscore):

```rust
// ❌ WRONG - causes "unresolved module" error
use libindigo::{Client, ClientBuilder, RsClientStrategy};

// ✅ CORRECT - use underscore in imports
use libindigo_rs::{Client, ClientBuilder, RsClientStrategy};
```

**Why does this happen?**

This is a Rust language convention. Rust package names (in Cargo.toml) can contain hyphens for readability, but module names in Rust code cannot contain hyphens. Therefore, Rust automatically converts hyphens to underscores when you import a crate.

Examples:

- `tokio-util` → `use tokio_util::`
- `serde-json` → `use serde_json::`
- `libindigo-rs` → `use libindigo_rs::`

### Correct Import Patterns

Here are the correct ways to import from `libindigo-rs`:

```rust
// Import specific items
use libindigo_rs::{Client, ClientBuilder, RsClientStrategy};

// Import with alias
use libindigo_rs as indigo;

// Import nested modules
use libindigo_rs::discovery::{DiscoveryBuilder, DiscoveryEvent};

// Import types
use libindigo_rs::types::{Property, PropertyType, PropertyValue};

// Import protocol negotiation
use libindigo_rs::{ProtocolNegotiator, ProtocolType};
```

## Migration from Old API

If you're migrating from the old monolithic `libindigo` crate (version 0.1.x), here's what you need to change:

### 1. Update Cargo.toml

```diff
[dependencies]
- libindigo = { version = "0.1", features = ["rs"] }
+ libindigo-rs = "0.3"
  tokio = { version = "1.35", features = ["full"] }
```

### 2. Update Import Statements

```diff
- use libindigo::strategies::RsClientStrategy;
- use libindigo::client::ClientBuilder;
+ use libindigo_rs::{RsClientStrategy, ClientBuilder};
```

### 3. Update Discovery Imports (if using discovery feature)

```diff
- use libindigo::discovery::{DiscoveryBuilder, DiscoveryEvent};
+ use libindigo_rs::discovery::{DiscoveryBuilder, DiscoveryEvent};
```

### 4. No Code Changes Required

The API remains the same, so your actual code logic doesn't need to change:

```rust
// This code works the same in both versions
let strategy = RsClientStrategy::new();
let mut client = ClientBuilder::new()
    .with_strategy(strategy)
    .build();

client.connect("localhost:7624").await?;
```

## Quick Reference

| Cargo.toml | Import Statement |
|------------|------------------|
| `libindigo-rs = "0.3"` | `use libindigo_rs::...` |
| `libindigo-ffi = "0.3"` | `use libindigo_ffi::...` |
| `libindigo-sys = "0.3"` | `use libindigo_sys::...` |

## Still Having Issues?

If you're still experiencing problems:

1. **Check your Cargo.toml**: Ensure you have `libindigo-rs` (not `libindigo`) in your dependencies
2. **Clean and rebuild**: Run `cargo clean && cargo build`
3. **Check Rust version**: This crate requires Rust 2021 edition or later
4. **Review examples**: See the [`examples/`](../examples/) directory for working code
5. **Open an issue**: If the problem persists, [open an issue on GitHub](https://github.com/chrsoo/libindigo-rs/issues)

## Additional Resources

- [Main README](README.md) - Getting started guide
- [Root README](../README.md) - Project overview and architecture
- [Examples](../examples/) - Working code examples
- [INDIGO Documentation](https://www.indigo-astronomy.org/) - INDIGO protocol documentation
