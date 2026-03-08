# libINDIGO issues

## GitHub Issues Created

**Date**: 2026-03-08
**Total Issues Created**: 63 issues from consolidated analysis

All issues have been created in the GitHub repository: <https://github.com/chrsoo/libindigo-rs/issues>

### Summary by Priority

| Priority | Count | Issue Numbers |
|----------|-------|---------------|
| **Critical** | 6 | #14, #17, #18, #19 (bugs & features) |
| **High** | 17 | #15, #16, #20-#27, #29, #30, #32-#34, #39, #54 |
| **Medium** | 28 | #36, #37, #40-#53, #55, #56, #58, #59, #63 |
| **Low** | 12 | #31, #35, #38, #57, #60-#62 |

### Issues by Category

#### 🐛 Bugs (4 issues)

- #14: Property Conversions Should Return Result Instead of Panicking (Critical)
- #15: Fix Tokio Runtime Nesting Issue (High)
- #16: Ensure Test Isolation (High)
- #35: Potential Rust Code Issues in relm Crate (Low)

#### 🚨 Critical Features (3 issues)

- #17: Implement Device Driver API (Critical)
- #18: Implement Trait-Based Device API (Critical - Tracking)
- #19: Complete ZeroConf Backend with Full mDNS Integration (Critical)

#### ⚡ High Priority Features (7 issues)

- #20: Complete FFI Integration with C INDIGO Library
- #21: Implement BLOB Sending/Receiving in Pure Rust
- #22: Update Examples to Use New Discovery API
- #23: Move Interface Enum Generation to libindigo-ffi
- #24: Consolidate and Clarify Cargo.toml Features
- #25: Clean Up src/lib.rs Technical Debt
- #26: Implement IPv6 Support for Discovery

#### 🧪 Testing & CI/CD (10 issues)

- #15: Fix Tokio Runtime Nesting Issue (High)
- #16: Ensure Test Isolation (High)
- #27: Update and Expand Integration Test Coverage (High)
- #28: Implement Comprehensive Integration Test Harness (High - Tracking)
- #39: Integration Tests Require Running INDIGO Server (High)
- #40: Simplify FFI Build Strategy in CI/CD (Medium)
- #41: Add macOS and Windows Runners to CI/CD (Medium)
- #42: Cache INDIGO Server Binary in CI (Medium)
- #43: Implement Lightweight Mock INDIGO Server (Medium)
- #60: Implement Nightly Integration Tests (Low)

#### 📚 Documentation (8 issues)

- #29: Publish API Documentation on docs.rs (High)
- #30: Complete Documentation Organization (High)
- #31: Clarify Rust's Hyphen-to-Underscore Conversion (Low)
- #32: Comprehensive API Documentation for Device Traits (High)
- #33: User Guide for Device APIs (High)
- #34: Developer Guide for Device Types (High)
- #37: Update Test Harness Documentation (Medium)
- #38: Move Completed Plans to Archive (Low)

#### 🔧 Technical Debt (11 issues)

- #23: Move Interface Enum Generation to libindigo-ffi (High)
- #24: Consolidate and Clarify Cargo.toml Features (High)
- #25: Clean Up src/lib.rs Technical Debt (High)
- #50: Fix 42 Unused Code Warnings in relm Crate (Medium)
- #51: Manual Version Bumping for Workspace Modules (Medium)
- #53: Implement ZeroConf Callback-Based API Integration (Medium)
- #54: Remove Deprecated Code from src/lib.rs (High)
- #55: Maintain src/constants.rs (Medium)
- #56: Move Removed Helper Functions to libindigo-ffi (Medium)
- #57: Move Manual Constants to libindigo or libindigo-ffi (Low)
- #58: Update relm Crate Dependencies (Medium)
- #61: Make Transport Struct Send/Sync (Low)

#### 📋 Medium Priority Features (9 issues)

- #36: Complete JSON Protocol BLOB Support
- #44: Populate Device Metadata Fields
- #45: Implement Automatic Server Startup/Shutdown
- #46: Implement Device Discovery with Type Detection
- #47: Implement Property State Waiting with Timeouts
- #48: Implement Automatic Reconnection Logic
- #49: Implement TLS Support for Secure Connections
- #52: Implement Pure Rust mdns-sd Library Alternative
- #59: Profile and Optimize Hot Paths

#### 🌐 Platform-Specific (3 issues)

- #62: BSD Support for ZeroConf Untested (Low)
- #63: Linux/Windows ZeroConf Dependencies Complex (Medium)
- #41: Add macOS and Windows Runners to CI/CD (Medium)

### Key Tracking Issues

These are high-level tracking issues that encompass multiple sub-tasks:

1. **#18: Implement Trait-Based Device API** (Critical)
   - Covers Camera, Mount, Focuser, FilterWheel, and other device traits
   - References plan: `plans/trait-based-device-api-v3.md`

2. **#28: Implement Comprehensive Integration Test Harness** (High)
   - Covers server lifecycle, health monitoring, state management, fixtures
   - References plan: `plans/integration_test_harness_architecture.md`

### Related Plans

The following planning documents provide detailed implementation guidance:

- [`plans/trait-based-device-api-v3.md`](trait-based-device-api-v3.md) - Device API architecture
- [`plans/integration_test_harness_architecture.md`](integration_test_harness_architecture.md) - Test infrastructure
- [`plans/ci-cd-strategy.md`](ci-cd-strategy.md) - CI/CD implementation
- [`plans/documentation-organization.md`](documentation-organization.md) - Documentation structure

### Implementation Phases

Based on the consolidated analysis, issues should be addressed in the following order:

#### Phase 1: Critical Bugs & Infrastructure (Issues #14-#16, #39, #28)

Focus on safety issues and test infrastructure that blocks other work.

#### Phase 2: Core Features (Issues #17-#19, #20-#21)

Implement foundational APIs: Device Driver API, Trait-Based Device API, ZeroConf, FFI, and BLOB handling.

#### Phase 3: High Priority Items (Issues #22-#27, #29-#34, #54)

Update examples, clean up technical debt, complete documentation.

#### Phase 4: Medium Priority (Issues #36-#53, #55-#59, #63)

Features, testing improvements, CI/CD enhancements, technical debt.

#### Phase 5: Low Priority & Polish (Issues #31, #35, #38, #57, #60-#62)

Nice-to-have features, platform support, automation.

---

## Original Structural Issues

The following structural issues were documented before the GitHub issue migration:

### INDIGO server connections

The INDIGO call for connecting to an INDIGO server is asynchronous (like everything else in INDIGO), but there is no callback method indicating connection success or failure.

- The `indigo_connection_status` INDIGO API call seems to indicate that the connection is OK even when no server connection is established.
- DNS failures are not reported and the server client keeps trying a long time (forever?) even whent the name does not resolve.
- The INDIGO API does not seem to have a way of interrupting ongoing connection attempts.

**Status**: These issues should be raised with upstream INDIGO project.

### Mapping of INDIGO constants

INDIGO contains a large number of string constants for names of interfaces, well-known properties, etc. These constants are automatically mapped to Rust code by [bindgen](https://github.com/rust-lang/rust-bindgen), for example:

```rust
pub const CONNECTION_PROPERTY_NAME: &[u8; 11] = b"CONNECTION\0";
```

Ideally these constants would be wrapped to a `&str` reference and/or mapped to an enum variant to be used in safe rust code. Exactly how to be figured out. The solution very likely includes adding custom `bindgen` code to [sys/build.rs](sys/build.rs).

**Status**: Tracked in issues #55, #57 (constants maintenance and consolidation).

### Using `&str` references for C strings

Beyond the INDIGO string constants, the INDIGO C-API uses C byte buffers for storing strings on all data carriers. Ideally we would point directly to these byte buffers without copying the buffer to a Rust string.

Currently the code base copies string buffers right and left, which seems like less than ideal.

Assuming this is feasible it, this requires a very good understanding and control of ownership.

**Status**: Part of FFI integration work tracked in issue #20.

### Using the bindgen structures in client and device code

Due to the limitations of sending unsafe pointers to raw memory between threads, the `bindgen` generated INDIGO structures in the [sys](sys) crate are transformed to safe libINDIGO structures. This is wasteful, and ideally the `bindgen` code is used as the data carrier for the safe Rust code.

However, a case could be made for creating an INDIGO API in Rust that could be both be used not only for abstracting the INDIGO C-API, but also for a protocol level rewrite that parses messages in XML or JSON and is independent of the default C-implementation. The benefit of this would be better control of memory management but at the cost of a second protocol implementation that would require stringent testing to ensure compatibility and interoperability, not only with INDIGO but also with the legacy INDI protocol.

This might prove to be the best option for writing embedded Rust code that is unencumbered by legacy C-code. To be explored further...

**Status**: Architectural decision to be made as part of issues #17, #18, #20 (Device API and FFI integration).

---

## Notes

- All issues created from [`plans/github-issues-to-create.md`](github-issues-to-create.md)
- Issue #19 was adjusted to note that mDNS discovery is already working in downstream clients
- Some issues were created with slightly different priorities than originally planned based on dependencies
- Total of 63 issues created (50 from core list + additional related issues)
