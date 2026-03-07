# Review: Issue #6 — Refactor the RS and FFI Strategies

**Issue**: [#6 — Refactor the RS and FFI strategies](https://github.com/chrsoo/libindigo-rs/issues/6)
**Review Date**: 2026-03-06
**Status**: Review with recommendations

---

## Summary of the Proposal

Issue #6 proposes splitting the monolithic `libindigo` crate into a multi-crate workspace:

| Current | Proposed | Role |
|---------|----------|------|
| `libindigo` (single crate with features) | `libindigo-core` | SPI traits, shared types, constants |
| `src/strategies/rs/` | `libindigo-rs` (new crate in `rs/`) | Pure Rust INDIGO client/device implementation |
| `src/strategies/ffi.rs`, `async_ffi.rs` | `libindigo-ffi` (new crate in `ffi/`) | FFI-based client/device implementation |
| `libindigo-sys` | `libindigo-sys` (refactored) | Only `bindgen`-generated raw C bindings |
| — | Update script | Automate INDIGO submodule update + constant extraction |

Features `rs` and `ffi` in [`Cargo.toml`](../../Cargo.toml) would be replaced by separate crates. Both `libindigo-rs` and `libindigo-ffi` would expose `client` and `device` feature toggles and re-export the public API from `libindigo-core` while hiding the SPI.

---

## Assessment: The Direction is Right ✅

The core insight is correct: **the RS and FFI strategies are fundamentally different dependency trees and should not be entangled via feature flags in a single crate.** A user wanting the pure Rust client should never pull in `libindigo-sys`, `bindgen`, or any C toolchain. Today that works via features, but it's fragile and the [`Cargo.toml`](../../Cargo.toml) already shows the complexity:

```toml
# Current: 8 features, some undocumented, some deprecated, some "legacy"
default = ["client", "async", "std"]
rs = ["client", "quick-xml", "tokio", "base64", "serde_json", "serde/derive"]
ffi = ["client", "libindigo-sys"]
sys = ["libindigo-sys"]     # "Legacy"
blocking = []               # "Legacy"
auto = ["zeroconf"]         # FFI-based discovery mixed in
test-server = ["libindigo-sys"]
```

And [`src/lib.rs`](../../src/lib.rs) is 530 lines of deprecated code, commented-out blocks, and `#[cfg]` gates — a strong signal that the boundaries are unclear.

---

## Detailed Review of Each Task

### ✅ 1. Rename `libindigo` → `libindigo-core`

**Verdict: Correct.**

The current `libindigo` crate mixes three concerns:
1. **API types** — [`ClientStrategy`](../../src/client/strategy.rs), [`Property`](../../src/types/property.rs), [`Device`](../../src/types/device.rs), error types
2. **RS implementation** — everything under [`src/strategies/rs/`](../../src/strategies/rs/)
3. **FFI implementation** — [`src/strategies/ffi.rs`](../../src/strategies/ffi.rs), [`src/strategies/async_ffi.rs`](../../src/strategies/async_ffi.rs)
4. **Deprecated old API** — `indigo.rs`, `msg.rs`, `property.rs`, `sys.rs`, `number.rs`, etc.

`libindigo-core` would contain only #1: the SPI trait ([`ClientStrategy`](../../src/client/strategy.rs)), shared types, error types, and the generated INDIGO constants ([`src/constants.rs`](../../src/constants.rs)).

**One concern**: the name `libindigo-core` is unusual in the Rust ecosystem. Most crates use `*-core` for `no_std` versions. Consider simply keeping the name `libindigo` for the core crate since it IS the canonical API — users would still write `use libindigo::prelude::*`.

**Recommendation**: Keep the core crate named **`libindigo`** rather than `libindigo-core`. The implementation crates (`libindigo-rs`, `libindigo-ffi`) are the ones that should be suffixed, not the core API.

### ✅ 2-3. New `libindigo-rs` and `libindigo-ffi` crates

**Verdict: Correct, with naming nuance.**

Separating the RS and FFI implementations into distinct crates is the right call:

- `libindigo-rs`: Zero C dependencies. Only Rust deps (`tokio`, `quick-xml`, `serde_json`, `base64`).
- `libindigo-ffi`: Depends on `libindigo-sys`. Pulls C toolchain.

This cleanly solves the current problem where `cargo build` with `--features ffi` pulls in the entire INDIGO C build system.

**Proposed directory layout:**

```
.
├── Cargo.toml              # Workspace root
├── src/                    # libindigo (core API + types)
│   ├── lib.rs
│   ├── client/
│   │   ├── mod.rs
│   │   ├── builder.rs
│   │   └── strategy.rs     # ClientStrategy trait (SPI)
│   ├── types/
│   ├── error.rs
│   └── constants.rs        # Generated INDIGO names
├── rs/                     # libindigo-rs
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs           # Re-exports libindigo API + RS strategy
│       ├── client.rs        # RsClientStrategy
│       ├── transport.rs
│       ├── protocol.rs
│       ├── protocol_json.rs
│       └── protocol_negotiation.rs
├── ffi/                    # libindigo-ffi
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs           # Re-exports libindigo API + FFI strategy
│       ├── ffi.rs           # FfiClientStrategy
│       └── async_ffi.rs     # AsyncFfiStrategy
├── sys/                    # libindigo-sys (unchanged structure)
│   ├── Cargo.toml
│   └── ...
└── relm/                   # libindigo-relm (unchanged)
```

**Key dependency graph:**

```
libindigo-rs ──► libindigo (core)
libindigo-ffi ──► libindigo (core) + libindigo-sys
libindigo-sys ──► (C INDIGO library)
libindigo-relm ──► libindigo-rs or libindigo-ffi
```

### ✅ 4. Refactor `libindigo-sys` to only contain `bindgen`

**Verdict: Correct.**

Currently [`sys/Cargo.toml`](../../sys/Cargo.toml) has `fambox` and `enum_primitive` as dependencies, suggesting some higher-level abstractions leaked in. `libindigo-sys` should be purely:
- `bindgen` output (raw `extern "C"` functions and types)
- The C build system (`build.rs`)
- No Rust abstractions

Any Rust wrappers around the FFI should live in `libindigo-ffi`.

### ✅ 5. Script for updating INDIGO submodule + constant extraction

**Verdict: Correct, and this overlaps with [#1](https://github.com/chrsoo/libindigo-rs/issues/1).**

Currently there are **two** copies of the constants:
- [`props.rs`](../../props.rs) at root (1255 lines, uses `*_NAME` suffix, has a duplicate on line 41)
- [`src/constants.rs`](../../src/constants.rs) (1253 lines, slightly different naming convention)

This duplication must be resolved. The extraction script should:
1. Parse INDIGO C headers from `sys/externals/indigo/`
2. Generate a single canonical `constants.rs`
3. Be invoked by an update script (not `build.rs` — the constants should be checked into version control to avoid build-time header dependency for `libindigo-rs`)

### ✅ 6. Remove all feature toggles from root Cargo.toml

**Verdict: Correct.**

The root `Cargo.toml` (which becomes `libindigo` core) should have no features related to strategy selection. It's a pure API/types crate.

### ✅ 7-8. `client` and `device` features in both `libindigo-rs` and `libindigo-ffi`

**Verdict: Correct.**

This is forward-looking. Currently only client functionality exists. The `device` feature (for writing INDIGO device drivers in Rust) is a planned capability. Having the feature toggle in both implementation crates makes sense because:

- **client**: Connect to servers, enumerate devices/properties, control devices
- **device**: Register as a device on the bus, expose properties, respond to clients

Both can have RS and FFI implementations.

### ✅ 9. Re-export core API from implementation crates

**Verdict: Correct and important.**

Users should only need one dependency in their `Cargo.toml`:

```toml
# Pure Rust path:
[dependencies]
libindigo-rs = "0.3"

# FFI path:
[dependencies]
libindigo-ffi = "0.3"
```

Both crates would re-export the public types from `libindigo`:

```rust
// In libindigo-rs/src/lib.rs:
pub use libindigo::prelude::*;
pub use libindigo::client::ClientStrategy;
// ... but NOT the ClientStrategy trait directly (it's SPI)

// Expose the RS implementation
pub mod strategy;
pub use strategy::RsClientStrategy;
```

The SPI ([`ClientStrategy`](../../src/client/strategy.rs) trait) should be accessible for advanced users who want to implement custom strategies, but it shouldn't be in the default prelude.

---

## Concerns and Risks

### 1. Discovery Module Placement

The current [`discovery/`](../../src/discovery/) module uses the `zeroconf` crate which wraps Bonjour/Avahi (C FFI). This creates a problem:

- It can't live in `libindigo` core (that should be pure Rust types)
- It can't live in `libindigo-rs` (that should have zero FFI deps)
- It logically should live in `libindigo-ffi`... but server discovery is independent of the client strategy

**Recommendation**: Create a separate `libindigo-discovery` crate, or find a pure-Rust mDNS/DNS-SD library to keep it in `libindigo-rs`. The `mdns-sd` crate is a pure-Rust alternative to `zeroconf`.

### 2. `build.rs` at Root

The current root [`build.rs`](../../build.rs) generates `interface.rs` via bindgen for the `Interface` enum. This is FFI code that must move to `libindigo-ffi` or be replaced with pure Rust constants from the extraction script.

### 3. Deprecated Code Cleanup

Issue #6 says "no need to keep backwards compatibility." This means the entire deprecated section of [`src/lib.rs`](../../src/lib.rs) (lines 196-527) — including [`indigo.rs`](../../src/indigo.rs), [`msg.rs`](../../src/msg.rs), [`property.rs`](../../src/property.rs), [`sys.rs`](../../src/sys.rs), [`number.rs`](../../src/number.rs) — should be **removed**, not just deprecated.

### 4. `relm` Crate Dependency

[`relm/Cargo.toml`](../../relm/Cargo.toml) depends on `libindigo = { path = ".." }`. After refactoring, it should depend on either `libindigo-rs` or `libindigo-ffi`. Since it's a GTK app (already has native deps), `libindigo-ffi` may be appropriate, or `libindigo-rs` if pure Rust is preferred.

---

## Recommended Crate Naming

| Crate | Package name | Directory | Published? |
|-------|-------------|-----------|------------|
| Core API + types | `libindigo` | `./` (root `src/`) | Yes |
| Pure Rust impl | `libindigo-rs` | `rs/` | Yes |
| FFI impl | `libindigo-ffi` | `ffi/` | Yes |
| Raw C bindings | `libindigo-sys` | `sys/` | Yes |
| Discovery | `libindigo-discovery` | `discovery/` | Yes (or merge into rs with pure Rust mDNS) |
| GTK demo | `libindigo-relm` | `relm/` | No |

---

## Recommended Execution Order

1. **Clean up deprecated code** — Remove all deprecated modules from [`src/lib.rs`](../../src/lib.rs). This dramatically simplifies the codebase before restructuring.
2. **Extract `libindigo-core` API** — Move only types, traits (SPI), and constants to the root crate. Remove all strategy code.
3. **Create `libindigo-rs`** — Move RS strategy code. Ensure zero FFI deps. Add `client` feature (default).
4. **Create `libindigo-ffi`** — Move FFI strategy code + async wrapper. Depends on `libindigo-sys`. Add `client` feature.
5. **Refactor `libindigo-sys`** — Strip to pure bindgen output.
6. **Handle discovery** — Decide on pure Rust mDNS vs separate crate.
7. **Constants extraction script** — Automate generation from INDIGO headers into `libindigo` core.
8. **Update `relm`** — Point at new dependency.
9. **Add `device` feature stubs** — Prepare for future device driver support.

---

## Final Verdict

**The refactoring is the right direction.** The key refinement I recommend:

> Keep the core crate named **`libindigo`** (not `libindigo-core`). Users write `use libindigo::Property` for types, and add either `libindigo-rs` or `libindigo-ffi` for the implementation. This matches the Rust convention where the `-sys` / `-rs` suffixes denote implementation variants of a base crate name.

The multi-crate split solves real problems:
- ✅ Clean dependency separation (no C toolchain for RS users)
- ✅ Feature flag complexity eliminated
- ✅ Each crate has a clear, single responsibility
- ✅ Forward-compatible with `client`/`device` split
- ✅ Deprecated code gets a clean removal rather than lingering

