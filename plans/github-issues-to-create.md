# GitHub Issues to Create - libindigo-rs

## Purpose

This document contains a filtered and prioritized list of GitHub issues derived from a comprehensive analysis of the libindigo project documentation. The original analysis identified 103 items; this document consolidates them into actionable issues by:

- Removing duplicates (items #87, #101, #102)
- Grouping related items into cohesive issues
- Filtering out overly vague or speculative items
- Prioritizing based on severity and impact
- Focusing on actionable, implementable work

**Total Issues to Create**: 51 (consolidated from 100 unique items)

---

## 🐛 Bugs (Critical & High Priority)

### BUG-001: Property Conversions Should Return Result Instead of Panicking

**Priority**: Critical
**Labels**: `bug`, `area:core`, `priority:critical`, `safety`

**Description**:
Property conversion functions currently panic on invalid input instead of returning proper error types. This violates Rust best practices and can crash applications unexpectedly.

**Source**: Item #7
**Files**: Core property conversion code

**Acceptance Criteria**:

- [ ] All property conversion functions return `Result<T, Error>`
- [ ] No panics in property conversion code paths
- [ ] Comprehensive error types for conversion failures
- [ ] Tests cover all error cases
- [ ] Migration guide for breaking API change

**Related Issues**: None

---

### BUG-002: Fix Tokio Runtime Nesting Issue

**Priority**: High
**Labels**: `bug`, `area:testing`, `priority:high`, `async`

**Description**:
Tests experience tokio runtime nesting issues causing failures or hangs. This affects test reliability and developer experience.

**Source**: Item #64
**Files**: Test harness, integration tests

**Acceptance Criteria**:

- [ ] No runtime nesting errors in test execution
- [ ] Tests run reliably without hangs
- [ ] Documentation explains async test patterns
- [ ] CI/CD validates fix across all test suites

**Related Issues**: BUG-003 (Test Isolation)

---

### BUG-003: Ensure Test Isolation

**Priority**: High
**Labels**: `bug`, `area:testing`, `priority:high`, `test-infrastructure`

**Description**:
Tests may interfere with each other due to shared state or server connections, leading to flaky test results.

**Source**: Item #66
**Files**: Test harness, integration tests

**Acceptance Criteria**:

- [ ] Each test has isolated server state
- [ ] No test interdependencies
- [ ] Parallel test execution works correctly
- [ ] Test cleanup is guaranteed (even on failure)
- [ ] Documentation on test isolation patterns

**Related Issues**: BUG-002 (Tokio Runtime), TEST-001 (Test Harness)

---

### BUG-004: Potential Rust Code Issues in relm Crate

**Priority**: Low
**Labels**: `bug`, `area:relm`, `priority:low`, `code-quality`

**Description**:
The relm crate has potential code quality issues beyond the 42 unused code warnings. Requires investigation and cleanup.

**Source**: Item #21
**Files**: `relm/` directory

**Acceptance Criteria**:

- [ ] Code review identifies specific issues
- [ ] Critical issues fixed
- [ ] Code quality improved
- [ ] Tests validate fixes

**Related Issues**: DEBT-002 (relm warnings)

---

## 🚨 Critical Missing Features

### FEATURE-001: Implement Device Driver API

**Priority**: Critical
**Labels**: `enhancement`, `area:core`, `priority:critical`, `device-api`

**Description**:
The device driver API is a core missing feature required for writing INDIGO device drivers in Rust. This is essential for the project\'s completeness.

**Source**: Item #6
**Files**: Core API, device modules

**Acceptance Criteria**:

- [ ] Device trait defined with all required methods
- [ ] Property registration and management
- [ ] Device lifecycle management (init, shutdown)
- [ ] Example device driver implementation
- [ ] Comprehensive documentation
- [ ] Integration tests with real INDIGO server

**Related Issues**: FEATURE-002 (Trait-Based Device API), DOC-004 (Device API docs)

---

### FEATURE-002: Implement Trait-Based Device API

**Priority**: Critical
**Labels**: `enhancement`, `area:core`, `priority:critical`, `device-api`, `tracking`

**Description**:
High-level, trait-based API for common INDIGO device types (Camera, Mount, Focuser, etc.). This provides ergonomic Rust interfaces for device interaction.

**Source**: Item #74
**Plan**: [`plans/trait-based-device-api-v3.md`](trait-based-device-api-v3.md)

**Acceptance Criteria**:

- [ ] Core Device trait implemented
- [ ] Camera trait with all standard properties
- [ ] Mount trait with positioning and tracking
- [ ] Focuser and FilterWheel traits
- [ ] Additional device traits (Dome, GPS, Guider, AO, Rotator, Aux)
- [ ] Builder patterns for device construction
- [ ] Comprehensive examples for each device type
- [ ] Full API documentation

**Related Issues**: FEATURE-001 (Device Driver API), DOC-004, DOC-005, DOC-006

---

### FEATURE-003: Complete ZeroConf Backend with Full mDNS Integration

**Priority**: Critical
**Labels**: `enhancement`, `area:discovery`, `priority:critical`, `networking`

**Description**:
ZeroConf backend requires full mDNS integration to be production-ready. Currently incomplete or partially implemented.

**Source**: Item #59
**Files**: Discovery module, ZeroConf implementation

**Acceptance Criteria**:

- [ ] Full mDNS service discovery
- [ ] Service announcement support
- [ ] IPv6 support
- [ ] Cross-platform compatibility (Linux, macOS, Windows, BSD)
- [ ] Integration tests on all platforms
- [ ] Performance benchmarks

**Related Issues**: FEATURE-010 (IPv6), PLATFORM-002 (BSD), PLATFORM-003 (Linux/Windows deps)

---

## ⚡ High Priority Enhancements

### FEATURE-004: Complete FFI Integration with C INDIGO Library

**Priority**: High
**Labels**: `enhancement`, `area:ffi`, `priority:high`, `ffi`

**Description**:
FFI implementation is currently stubbed. Complete integration with the C INDIGO library for maximum compatibility.

**Source**: Item #4
**Files**: `ffi/` crate, FFI strategy implementation

**Acceptance Criteria**:

- [ ] All FFI bindings implemented
- [ ] Async wrappers for blocking C calls
- [ ] Memory safety guarantees
- [ ] Error handling from C library
- [ ] Feature parity with pure Rust implementation
- [ ] Integration tests with C library
- [ ] Performance benchmarks vs pure Rust

**Related Issues**: FEATURE-005 (BLOB handling)

---

### FEATURE-005: Implement BLOB Sending/Receiving in Pure Rust

**Priority**: High
**Labels**: `enhancement`, `area:protocol`, `priority:high`, `blob`

**Description**:
BLOB (Binary Large Object) handling needs complete implementation in the pure Rust strategy for image transfer and large data handling.

**Source**: Item #5
**Files**: Protocol implementation, transport layer

**Acceptance Criteria**:

- [ ] BLOB sending via URL
- [ ] BLOB sending via BASE64 (XML protocol)
- [ ] BLOB receiving and validation
- [ ] Streaming support for large BLOBs
- [ ] Memory-efficient handling
- [ ] Tests with real image data
- [ ] Performance benchmarks

**Related Issues**: FEATURE-018 (JSON BLOB support)

---

### FEATURE-006: Update Examples to Use New Discovery API

**Priority**: High
**Labels**: `chore`, `area:examples`, `priority:high`, `documentation`

**Description**:
Several examples use deprecated discovery features and need updating to the new API.

**Source**: Item #9
**Files**: `examples/auto_connect.rs`, `examples/continuous_discovery.rs`

**Acceptance Criteria**:

- [ ] All examples use current API
- [ ] Examples compile without warnings
- [ ] Examples run successfully
- [ ] README updated with correct examples
- [ ] Deprecated code removed

**Related Issues**: DOC-001 (API documentation)

---

### FEATURE-007: Move Interface Enum Generation to libindigo-ffi

**Priority**: High
**Labels**: `technical-debt`, `area:ffi`, `priority:high`, `refactoring`

**Description**:
Interface enum generation currently in wrong location. Should be in libindigo-ffi crate for proper separation of concerns.

**Source**: Item #53
**Files**: FFI crate, code generation

**Acceptance Criteria**:

- [ ] Interface enum generation moved to libindigo-ffi
- [ ] Build process updated
- [ ] All tests pass
- [ ] Documentation updated
- [ ] No breaking changes to public API

**Related Issues**: DEBT-005 (Remove deprecated code)

---

### FEATURE-008: Consolidate and Clarify Cargo.toml Features

**Priority**: High
**Labels**: `technical-debt`, `area:core`, `priority:high`, `configuration`

**Description**:
Feature flags across workspace crates need consolidation and clear documentation to avoid confusion and conflicts.

**Source**: Item #56
**Files**: All `Cargo.toml` files

**Acceptance Criteria**:

- [ ] Feature flags documented in each crate
- [ ] No conflicting or redundant features
- [ ] Clear feature combinations documented
- [ ] Examples show common feature usage
- [ ] CI tests multiple feature combinations

**Related Issues**: DOC-001 (API documentation)

---

### FEATURE-009: Clean Up src/lib.rs Technical Debt

**Priority**: High
**Labels**: `technical-debt`, `area:core`, `priority:high`, `code-quality`

**Description**:
Main library file has accumulated technical debt that needs cleanup for maintainability.

**Source**: Item #57
**Files**: `src/lib.rs`

**Acceptance Criteria**:

- [ ] Code organization improved
- [ ] Deprecated items removed
- [ ] Documentation complete
- [ ] No clippy warnings
- [ ] Tests validate all functionality

**Related Issues**: DEBT-005 (Remove deprecated code)

---

### FEATURE-010: Implement IPv6 Support for Discovery

**Priority**: High
**Labels**: `enhancement`, `area:discovery`, `priority:high`, `networking`

**Description**:
Server discovery should support IPv6 addresses for modern network environments.

**Source**: Item #92
**Files**: Discovery module, mDNS implementation

**Acceptance Criteria**:

- [ ] IPv6 address discovery
- [ ] IPv6 connection support
- [ ] Dual-stack (IPv4/IPv6) support
- [ ] Tests on IPv6 networks
- [ ] Documentation on IPv6 usage

**Related Issues**: FEATURE-003 (ZeroConf)

---

## 📋 Medium Priority Enhancements

### FEATURE-011: Populate Device Metadata Fields

**Priority**: Medium
**Labels**: `enhancement`, `area:core`, `priority:medium`, `metadata`

**Description**:
Device metadata fields need to be properly populated for complete device information.

**Source**: Item #8
**Files**: Device types, property handling

**Acceptance Criteria**:

- [ ] All metadata fields identified
- [ ] Metadata populated from INDIGO properties
- [ ] Metadata accessible via API
- [ ] Tests validate metadata accuracy
- [ ] Documentation explains metadata fields

**Related Issues**: FEATURE-001 (Device Driver API)

---

### FEATURE-012: Implement Automatic Server Startup/Shutdown

**Priority**: Medium
**Labels**: `enhancement`, `area:testing`, `priority:medium`, `test-infrastructure`

**Description**:
Test harness should automatically manage INDIGO server lifecycle for integration tests.

**Source**: Item #65
**Files**: Test harness

**Acceptance Criteria**:

- [ ] Server starts automatically before tests
- [ ] Server stops automatically after tests
- [ ] Configurable server options
- [ ] Health checks before running tests
- [ ] Cleanup on test failure

**Related Issues**: TEST-001 (Test Harness), BUG-003 (Test Isolation)

---

### FEATURE-013: Implement Device Discovery with Type Detection

**Priority**: Medium
**Labels**: `enhancement`, `area:discovery`, `priority:medium`, `device-api`

**Description**:
Discovery should detect device types automatically, enabling type-specific handling.

**Source**: Item #75
**Files**: Discovery module, device API

**Acceptance Criteria**:

- [ ] Device type detection from properties
- [ ] Type information in discovery events
- [ ] Filtering by device type
- [ ] Examples showing type-based discovery
- [ ] Documentation on device types

**Related Issues**: FEATURE-002 (Trait-Based Device API)

---

### FEATURE-014: Implement Property State Waiting with Timeouts

**Priority**: Medium
**Labels**: `enhancement`, `area:core`, `priority:medium`, `async`

**Description**:
Provide async methods to wait for property state changes with configurable timeouts.

**Source**: Item #77
**Files**: Client API, property handling

**Acceptance Criteria**:

- [ ] `wait_for_property()` method with timeout
- [ ] Predicate-based waiting
- [ ] Timeout error handling
- [ ] Examples showing usage
- [ ] Tests with various timeout scenarios

**Related Issues**: FEATURE-002 (Trait-Based Device API)

---

### FEATURE-015: Implement Automatic Reconnection Logic

**Priority**: Medium
**Labels**: `enhancement`, `area:networking`, `priority:medium`, `reliability`

**Description**:
Client should automatically reconnect to server on connection loss with configurable retry logic.

**Source**: Item #94
**Files**: Transport layer, client implementation

**Acceptance Criteria**:

- [ ] Automatic reconnection on disconnect
- [ ] Configurable retry strategy (exponential backoff)
- [ ] Maximum retry limit
- [ ] Connection state events
- [ ] Tests simulate connection loss
- [ ] Documentation on reconnection behavior

**Related Issues**: None

---

### FEATURE-016: Implement TLS Support for Secure Connections

**Priority**: Medium
**Labels**: `enhancement`, `area:networking`, `priority:medium`, `security`

**Description**:
Add TLS/SSL support for encrypted connections to INDIGO servers.

**Source**: Item #97
**Files**: Transport layer

**Acceptance Criteria**:

- [ ] TLS connection support
- [ ] Certificate validation
- [ ] Optional TLS (fallback to plain TCP)
- [ ] Configuration for TLS options
- [ ] Tests with TLS server
- [ ] Documentation on TLS setup

**Related Issues**: None

---

### FEATURE-017: Implement Pure Rust mdns-sd Library Alternative

**Priority**: Medium
**Labels**: `enhancement`, `area:discovery`, `priority:medium`, `dependencies`

**Description**:
Evaluate and potentially implement alternative to current mdns-sd dependency for better cross-platform support.

**Source**: Item #52
**Files**: Discovery module dependencies

**Acceptance Criteria**:

- [ ] Research alternative mdns-sd libraries
- [ ] Evaluate performance and compatibility
- [ ] Implement or integrate alternative
- [ ] Tests on all platforms
- [ ] Migration guide if changing dependency

**Related Issues**: FEATURE-003 (ZeroConf), PLATFORM-003

---

### FEATURE-018: Complete JSON Protocol BLOB Support

**Priority**: Medium
**Labels**: `enhancement`, `area:protocol`, `priority:medium`, `json`, `blob`

**Description**:
JSON protocol BLOB support is incomplete. Needs full implementation for feature parity with XML protocol.

**Source**: Item #68
**Files**: JSON protocol implementation

**Acceptance Criteria**:

- [ ] BLOB URL support in JSON
- [ ] BLOB metadata in JSON messages
- [ ] Tests with BLOB transfers
- [ ] Documentation on JSON BLOB format
- [ ] Compatibility with INDIGO server

**Related Issues**: FEATURE-005 (BLOB handling)

---

### FEATURE-019: Implement ZeroConf Callback-Based API Integration

**Priority**: Medium
**Labels**: `technical-debt`, `area:discovery`, `priority:medium`, `api-design`

**Description**:
ZeroConf crate uses callback-based API that needs proper integration with async Rust patterns.

**Source**: Item #60
**Files**: Discovery implementation

**Acceptance Criteria**:

- [ ] Callbacks converted to async streams
- [ ] Proper error propagation
- [ ] No blocking in async context
- [ ] Tests validate async behavior
- [ ] Documentation on async patterns

**Related Issues**: FEATURE-003 (ZeroConf)

---

## 🧪 Testing & CI/CD

### TEST-001: Update and Expand Integration Test Coverage

**Priority**: High
**Labels**: `testing`, `area:testing`, `priority:high`, `test-coverage`

**Description**:
Integration tests need updates for new APIs and expansion to cover edge cases.

**Source**: Items #10, #24
**Files**: Integration tests, test harness

**Acceptance Criteria**:

- [ ] All new APIs have integration tests
- [ ] Edge cases covered (timeouts, errors, malformed data)
- [ ] Tests use current test harness
- [ ] Test documentation updated
- [ ] CI runs all integration tests

**Related Issues**: TEST-002 (Test Harness), CI-001 (Integration tests in CI)

---

### TEST-002: Implement Comprehensive Integration Test Harness

**Priority**: High
**Labels**: `testing`, `area:testing`, `priority:high`, `test-infrastructure`, `tracking`

**Description**:
Complete implementation of integration test harness for managing INDIGO server during tests.

**Source**: Items #25-43 (consolidated)
**Plan**: [`plans/integration_test_harness_architecture.md`](integration_test_harness_architecture.md)

**Acceptance Criteria**:

- [ ] Server lifecycle management (start/stop)
- [ ] Health monitoring
- [ ] State management between tests
- [ ] Parallel test execution support
- [ ] Test fixtures and helpers
- [ ] Coverage reporting integration
- [ ] Performance benchmarks
- [ ] Better error messages
- [ ] Documentation and examples

**Sub-features** (from items #28-43):

- Parallel execution (#28)
- Test fixtures (#29)
- Performance benchmarks (#30)
- Better error messages (#31)
- Docker support (#32)
- Remote server support (#33)
- Test recording/replay (#34)
- Coverage reporting (#35)
- Visual test reports (#39)
- Smart state management (#40)
- Configurable isolation levels (#41)
- Automatic retry for flaky tests (#42)
- Test dependencies (#43)

**Related Issues**: TEST-001, CI-001, BUG-003

---

### TEST-003: Implement Lightweight Mock INDIGO Server

**Priority**: Medium
**Labels**: `testing`, `area:testing`, `priority:medium`, `mocking`

**Description**:
Create lightweight mock server for unit tests that don\'t require full INDIGO server.

**Source**: Item #49
**Files**: Test utilities

**Acceptance Criteria**:

- [ ] Mock server implements core protocol
- [ ] Configurable responses
- [ ] Fast startup/shutdown
- [ ] No external dependencies
- [ ] Examples of mock usage
- [ ] Documentation

**Related Issues**: TEST-002 (Test Harness)

---

### TEST-004: Create Runnable Example Tests for Test Framework

**Priority**: Medium
**Labels**: `testing`, `area:examples`, `priority:medium`, `documentation`

**Description**:
Create runnable example tests that demonstrate the usage of the new test framework, derived from GitHub issue #64. These examples will serve as a guide for users and developers on how to write tests using the `libindigo-test` crate.

**Source**: GitHub Issue #64
**Files**: `tests/` directory, documentation

**Acceptance Criteria**:

- [ ] Clear, concise, and runnable example tests
- [ ] Examples cover core functionalities of the test framework
- [ ] Examples integrated into existing documentation or a new "examples" section
- [ ] Tests validate the examples themselves
- [ ] Documentation explains how to extend or adapt examples

**Related Issues**: TEST-002 (Test Harness), DOC-007 (Test Harness Documentation)

---

### CI-001: Integration Tests Require Running INDIGO Server

**Priority**: High
**Labels**: `ci-cd`, `area:testing`, `priority:high`, `infrastructure`

**Description**:
CI/CD pipeline needs proper setup for running integration tests with INDIGO server.

**Source**: Item #45
**Plan**: [`plans/ci-cd-strategy.md`](ci-cd-strategy.md)

**Acceptance Criteria**:

- [ ] INDIGO server available in CI environment
- [ ] Server starts before integration tests
- [ ] Tests run reliably in CI
- [ ] Graceful degradation if server unavailable
- [ ] Clear CI logs and error messages

**Related Issues**: CI-002 (FFI build), CI-003 (Platform runners)

---

### CI-002: Simplify FFI Build Strategy in CI/CD

**Priority**: Medium
**Labels**: `ci-cd`, `area:ffi`, `priority:medium`, `build`

**Description**:
FFI build strategy is complex in CI/CD. Simplify and document the build process.

**Source**: Items #44, #46, #48
**Files**: CI configuration, build scripts

**Acceptance Criteria**:

- [ ] Simplified FFI build process
- [ ] Clear build documentation
- [ ] Troubleshooting guide for build failures
- [ ] Build cache optimization
- [ ] Faster CI builds

**Related Issues**: CI-001, DOC-007 (Troubleshooting)

---

### CI-003: Add macOS and Windows Runners to CI/CD

**Priority**: Medium
**Labels**: `ci-cd`, `area:infrastructure`, `priority:medium`, `cross-platform`

**Description**:
CI/CD currently lacks macOS and Windows runners. Add for comprehensive platform testing.

**Source**: Item #51
**Files**: CI configuration

**Acceptance Criteria**:

- [ ] macOS runner configured
- [ ] Windows runner configured
- [ ] Platform-specific tests run
- [ ] Build artifacts for all platforms
- [ ] Documentation on platform differences

**Related Issues**: PLATFORM-001, PLATFORM-002, PLATFORM-003

---

### CI-004: Implement Nightly Integration Tests

**Priority**: Low
**Labels**: `ci-cd`, `area:testing`, `priority:low`, `automation`

**Description**:
Run comprehensive integration tests nightly to catch regressions early.

**Source**: Item #50
**Files**: CI configuration

**Acceptance Criteria**:

- [ ] Nightly test schedule configured
- [ ] Full test suite runs
- [ ] Results reported to team
- [ ] Failure notifications
- [ ] Test history tracking

**Related Issues**: TEST-001, CI-001

---

### CI-005: Cache INDIGO Server Binary in CI

**Priority**: Medium
**Labels**: `ci-cd`, `area:testing`, `priority:medium`, `performance`

**Description**:
Cache INDIGO server binary to speed up CI builds and test runs.

**Source**: Item #67
**Files**: CI configuration

**Acceptance Criteria**:

- [ ] Server binary cached between runs
- [ ] Cache invalidation on version change
- [ ] Faster CI execution
- [ ] Documentation on cache strategy

**Related Issues**: CI-001, CI-002

---

## 📚 Documentation

### DOC-001: Publish API Documentation on docs.rs

**Priority**: High
**Labels**: `documentation`, `area:docs`, `priority:high`, `api-docs`

**Description**:
Publish comprehensive API documentation to docs.rs for all public crates.

**Source**: Items #12, #13
**Files**: All crate documentation

**Acceptance Criteria**:

- [ ] All public APIs documented
- [ ] Examples in doc comments
- [ ] docs.rs builds successfully
- [ ] README links to docs.rs
- [ ] Documentation coverage > 90%

**Related Issues**: DOC-002, DOC-003

---

### DOC-002: Complete Documentation Organization

**Priority**: High
**Labels**: `documentation`, `area:docs`, `priority:high`, `organization`

**Description**:
Documentation is scattered and incomplete. Organize into clear structure.

**Source**: Item #62
**Plan**: [`plans/documentation-organization.md`](documentation-organization.md)

**Acceptance Criteria**:

- [ ] Architecture docs in `docs/architecture/`
- [ ] Protocol docs in `docs/protocols/`
- [ ] Development docs in `docs/development/`
- [ ] Comprehensive `docs/README.md` index
- [ ] Plans directory contains only active plans
- [ ] Completed plans moved to `plans/archive/`

**Related Issues**: DOC-008 (Move completed plans)

---

### DOC-003: Clarify Rust\'s Hyphen-to-Underscore Conversion

**Priority**: Low
**Labels**: `documentation`, `area:docs`, `priority:low`, `user-experience`

**Description**:
Document Rust\'s automatic hyphen-to-underscore conversion in crate names to prevent user confusion.

**Source**: Item #19
**Files**: README, crate documentation

**Acceptance Criteria**:

- [ ] Clear explanation in README
- [ ] Examples showing correct imports
- [ ] Common errors documented
- [ ] Troubleshooting section

**Related Issues**: DOC-001

---

### DOC-004: Comprehensive API Documentation for Device Traits

**Priority**: High
**Labels**: `documentation`, `area:docs`, `priority:high`, `device-api`

**Description**:
Device trait API needs comprehensive documentation with examples.

**Source**: Item #84
**Files**: Device trait documentation

**Acceptance Criteria**:

- [ ] All device traits documented
- [ ] Usage examples for each trait
- [ ] Property mapping explained
- [ ] Error handling documented
- [ ] Best practices guide

**Related Issues**: FEATURE-002 (Trait-Based Device API)

---

### DOC-005: User Guide for Device APIs

**Priority**: High
**Labels**: `documentation`, `area:docs`, `priority:high`, `user-guide`

**Description**:
Create user-friendly guide for using device APIs in applications.

**Source**: Item #85
**Files**: User documentation

**Acceptance Criteria**:

- [ ] Getting started guide
- [ ] Common use cases covered
- [ ] Complete examples
- [ ] Troubleshooting section
- [ ] FAQ

**Related Issues**: FEATURE-002, DOC-004

---

### DOC-006: Developer Guide for Device Types

**Priority**: High
**Labels**: `documentation`, `area:docs`, `priority:high`, `developer-guide`

**Description**:
Create developer guide for implementing new device types and drivers.

**Source**: Item #86
**Files**: Developer documentation

**Acceptance Criteria**:

- [ ] Device driver architecture explained
- [ ] Step-by-step implementation guide
- [ ] Example device driver
- [ ] Testing guidelines
- [ ] Best practices

**Related Issues**: FEATURE-001, FEATURE-002

---

### DOC-007: Update Test Harness Documentation

**Priority**: Medium
**Labels**: `documentation`, `area:docs`, `priority:medium`, `testing`

**Description**:
Test harness documentation needs updates for new features and usage patterns.

**Source**: Item #27
**Files**: Test harness documentation

**Acceptance Criteria**:

- [ ] Architecture documented
- [ ] Usage examples
- [ ] Configuration options explained
- [ ] Troubleshooting guide
- [ ] Best practices

**Related Issues**: TEST-002 (Test Harness)

---

### DOC-008: Move Completed Plans to Archive

**Priority**: Low
**Labels**: `chore`, `area:docs`, `priority:low`, `organization`

**Description**:
Move completed planning documents to archive directory for better organization.

**Source**: Item #63
**Files**: `plans/` directory

**Acceptance Criteria**:

- [ ] Completed plans identified
- [ ] Plans moved to `plans/archive/`
- [ ] README updated with archive references
- [ ] Links updated in other documents

**Related Issues**: DOC-002 (Documentation Organization)

---

## 🔧 Technical Debt

### DEBT-001: Manual Version Bumping for Workspace Modules

**Priority**: Medium
**Labels**: `technical-debt`, `area:build`, `priority:medium`, `automation`

**Description**:
Version numbers must be manually updated in each workspace crate. Automate this process.

**Source**: Item #1
**Files**: All `Cargo.toml` files, release scripts

**Acceptance Criteria**:

- [ ] Script to bump versions across workspace
- [ ] Version consistency validation
- [ ] CI checks version alignment
- [ ] Documentation on release process

**Related Issues**: None

---

### DEBT-002: Fix 42 Unused Code Warnings in relm Crate

**Priority**: Medium
**Labels**: `technical-debt`, `area:relm`, `priority:medium`, `code-quality`

**Description**:
The relm crate has 42 warnings about unused code that need to be addressed.

**Source**: Item #20
**Files**: `relm/` directory

**Acceptance Criteria**:

- [ ] All unused code warnings resolved
- [ ] Dead code removed or marked with `#[allow(dead_code)]` with justification
- [ ] No new warnings introduced
- [ ] CI enforces zero warnings

**Related Issues**: BUG-004 (relm code issues)

---

### DEBT-003: Move Removed Helper Functions to libindigo-ffi

**Priority**: Medium
**Labels**: `technical-debt`, `area:ffi`, `priority:medium`, `refactoring`

**Description**:
Helper functions removed from core should be moved to libindigo-ffi where they belong.

**Source**: Item #22
**Files**: FFI crate, helper utilities

**Acceptance Criteria**:

- [ ] Helper functions identified
- [ ] Functions moved to libindigo-ffi
- [ ] Tests updated
- [ ] Documentation updated
- [ ] No breaking changes

**Related Issues**: FEATURE-007 (Interface enum)

---

### DEBT-004: Move Manual Constants to libindigo or libindigo-ffi

**Priority**: Low
**Labels**: `technical-debt`, `area:core`, `priority:low`, `refactoring`

**Description**:
Manual constants scattered in codebase should be consolidated in appropriate crates.

**Source**: Item #23
**Files**: Constants definitions

**Acceptance Criteria**:

- [ ] Manual constants identified
- [ ] Constants moved to appropriate crate
- [ ] No duplication
- [ ] Tests validate constants
- [ ] Documentation updated

**Related Issues**: DEBT-006 (Maintain constants)

---

### DEBT-005: Remove Deprecated Code from src/lib.rs

**Priority**: High
**Labels**: `technical-debt`, `area:core`, `priority:high`, `breaking-change`

**Description**:
Remove deprecated code from main library file as part of cleanup for next major version.

**Source**: Item #54
**Files**: `src/lib.rs`

**Acceptance Criteria**:

- [ ] All deprecated items removed
- [ ] Migration guide created
- [ ] Breaking changes documented
- [ ] Tests updated
- [ ] Version bumped appropriately

**Related Issues**: FEATURE-009 (Clean up lib.rs)

---

### DEBT-006: Maintain src/constants.rs

**Priority**: Medium
**Labels**: `technical-debt`, `area:core`, `priority:medium`, `maintenance`

**Description**:
Constants file needs ongoing maintenance to stay in sync with INDIGO.

**Source**: Item #58
**Files**: `src/constants.rs`

**Acceptance Criteria**:

- [ ] Constants up to date with INDIGO
- [ ] Automated update process
- [ ] Tests validate constants
- [ ] Documentation on update process

**Related Issues**: AUTOMATION-001 (Constant updates)

---

### DEBT-007: Update relm Crate Dependencies

**Priority**: Medium
**Labels**: `technical-debt`, `area:relm`, `priority:medium`, `dependencies`

**Description**:
The relm crate has outdated dependencies that need updating.

**Source**: Item #55
**Files**: `relm/Cargo.toml`

**Acceptance Criteria**:

- [ ] Dependencies updated to latest compatible versions
- [ ] Security vulnerabilities addressed
- [ ] Tests pass with new dependencies
- [ ] Breaking changes documented

**Related Issues**: DEBT-002 (relm warnings)

---

### DEBT-008: Make Transport Struct Send/Sync

**Priority**: Low
**Labels**: `technical-debt`, `area:networking`, `priority:low`, `async`

**Description**:
Transport struct is not Send/Sync, limiting its use in async contexts.

**Source**: Item #100
**Files**: Transport implementation

**Acceptance Criteria**:

- [ ] Transport implements Send
- [ ] Transport implements Sync
- [ ] Tests validate thread safety
- [ ] Documentation updated

**Related Issues**: None

---

## 🤖 Automation & Code Generation

### AUTOMATION-001: Automated Constant Updates in CI Pipeline

**Priority**: Medium
**Labels**: `automation`, `area:build`, `priority:medium`, `ci-cd`

**Description**:
Automate extraction and updating of INDIGO constants in CI pipeline.

**Source**: Item #18
**Files**: CI configuration, build scripts

**Acceptance Criteria**:

- [ ] CI extracts constants from INDIGO headers
- [ ] Constants compared with current version
- [ ] PR created if constants changed
- [ ] Tests validate new constants
- [ ] Documentation on process

**Related Issues**: DEBT-006 (Maintain constants), CODEGEN-001

---

### CODEGEN-001: Add Compile-Time Validation of Constant Usage

**Priority**: Low
**Labels**: `automation`, `area:core`, `priority:low`, `code-generation`

**Description**:
Add compile-time checks to validate that constants are used correctly throughout the codebase.

**Source**: Item #16
**Files**: Build scripts, macro definitions

**Acceptance Criteria**:

- [ ] Compile-time validation for constant usage
- [ ] Clear error messages for misuse
- [ ] Documentation on validation rules
- [ ] Tests for validation logic

**Related Issues**: AUTOMATION-001, DEBT-006

---

### CODEGEN-002: Extract Device Interface Definitions

**Priority**: Low
**Labels**: `automation`, `area:core`, `priority:low`, `code-generation`

**Description**:
Automatically extract device interface definitions from INDIGO headers.

**Source**: Item #14
**Files**: Build scripts, code generation

**Acceptance Criteria**:

- [ ] Interface definitions extracted
- [ ] Rust types generated
- [ ] Tests validate generated code
- [ ] Documentation on generation process

**Related Issues**: FEATURE-002 (Device API), CODEGEN-003

---

### CODEGEN-003: Generate Property Type Information

**Priority**: Low
**Labels**: `automation`, `area:core`, `priority:low`, `code-generation`

**Description**:
Generate property type information from INDIGO definitions for type-safe property handling.

**Source**: Item #15
**Files**: Build scripts, type generation

**Acceptance Criteria**:

- [ ] Property types extracted
- [ ] Type-safe wrappers generated
- [ ] Tests validate types
- [ ] Documentation updated

**Related Issues**: CODEGEN-002, FEATURE-002

---

### CODEGEN-004: Auto-Generate Documentation from INDIGO Comments

**Priority**: Low
**Labels**: `automation`, `area:docs`, `priority:low`, `code-generation`

**Description**:
Automatically generate Rust documentation from INDIGO C header comments.

**Source**: Item #17
**Files**: Build scripts, documentation generation

**Acceptance Criteria**:

- [ ] Comments extracted from C headers
- [ ] Rust doc comments generated
- [ ] Documentation builds successfully
- [ ] Cross-references maintained

**Related Issues**: DOC-001 (API docs)

---

## 🌐 Platform-Specific Issues

### PLATFORM-001: BSD Support for ZeroConf Untested

**Priority**: Low
**Labels**: `platform`, `area:discovery`, `priority:low`, `bsd`

**Description**:
ZeroConf discovery has not been tested on BSD platforms. Validate and fix any issues.

**Source**: Item #61
**Files**: Discovery module, platform-specific code

**Acceptance Criteria**:

- [ ] Tests run on FreeBSD
- [ ] Tests run on OpenBSD
- [ ] Platform-specific issues fixed
- [ ] CI includes BSD testing (if feasible)
- [ ] Documentation notes BSD support

**Related Issues**: FEATURE-003 (ZeroConf), CI-003

---

### PLATFORM-002: Linux/Windows ZeroConf Dependencies Complex

**Priority**: Medium
**Labels**: `platform`, `area:discovery`, `priority:medium`, `dependencies`

**Description**:
ZeroConf dependencies on Linux and Windows are complex. Simplify and document.

**Source**: Item #103
**Files**: Discovery dependencies, platform code

**Acceptance Criteria**:

- [ ] Dependencies documented clearly
- [ ] Installation instructions for each platform
- [ ] Troubleshooting guide
- [ ] Alternative backends evaluated
- [ ] CI tests on Linux and Windows

**Related Issues**: FEATURE-003 (ZeroConf), FEATURE-017 (mdns-sd alternative)

---

## 🔬 Research & Exploration

### RESEARCH-001: Investigate slint for Embedded GUI Development

**Priority**: Low
**Labels**: `research`, `area:gui`, `priority:low`, `exploration`

**Description**:
Investigate slint framework for potential embedded GUI development with INDIGO.

**Source**: Item #2
**Files**: N/A (research task)

**Acceptance Criteria**:

- [ ] slint framework evaluated
- [ ] Proof-of-concept GUI created
- [ ] Performance benchmarks
- [ ] Comparison with alternatives
- [ ] Recommendation documented

**Related Issues**: None

---

### RESEARCH-002: Formalize Trait Design for Callback Methods

**Priority**: Low
**Labels**: `research`, `area:core`, `priority:low`, `architectural-discussion`

**Description**:
Research and formalize trait design patterns for callback methods in async Rust context.

**Source**: Item #3
**Files**: N/A (research task)

**Acceptance Criteria**:

- [ ] Current patterns analyzed
- [ ] Best practices researched
- [ ] Recommendations documented
- [ ] Prototype implementation
- [ ] Team discussion and decision

**Related Issues**: FEATURE-002 (Device API)

---

## 🚀 Performance & Optimization

### PERF-001: Profile and Optimize Hot Paths

**Priority**: Medium
**Labels**: `performance`, `area:core`, `priority:medium`, `optimization`

**Description**:
Profile application hot paths and optimize for better performance.

**Source**: Item #11
**Files**: Core library, protocol handling

**Acceptance Criteria**:

- [ ] Profiling infrastructure set up
- [ ] Hot paths identified
- [ ] Optimizations implemented
- [ ] Performance benchmarks show improvement
- [ ] Regression tests prevent slowdowns

**Related Issues**: None

---

## 🎯 Low Priority Enhancements

### ENHANCE-001: Implement WebSocket Support for JSON Protocol

**Priority**: Low
**Labels**: `enhancement`, `area:protocol`, `priority:low`, `websocket`

**Description**:
Add WebSocket transport option for JSON protocol, useful for web applications.

**Source**: Item #69
**Files**: Transport layer, protocol implementation

**Acceptance Criteria**:

- [ ] WebSocket transport implemented
- [ ] JSON protocol over WebSocket
- [ ] Tests with WebSocket server
- [ ] Examples showing usage
- [ ] Documentation

**Related Issues**: None

---

### ENHANCE-002: Implement Streaming Parser for JSON

**Priority**: Low
**Labels**: `enhancement`, `area:protocol`, `priority:low`, `performance`

**Description**:
Implement streaming JSON parser for better memory efficiency with large messages.

**Source**: Item #70
**Files**: JSON protocol parser

**Acceptance Criteria**:

- [ ] Streaming parser implemented
- [ ] Memory usage reduced
- [ ] Performance benchmarks
- [ ] Tests with large messages
- [ ] Backward compatibility maintained

**Related Issues**: PERF-001

---

### ENHANCE-003: Implement JSON Schema Validation

**Priority**: Low
**Labels**: `enhancement`, `area:protocol`, `priority:low`, `validation`

**Description**:
Add JSON schema validation for protocol messages to catch errors early.

**Source**: Item #71
**Files**: JSON protocol implementation

**Acceptance Criteria**:

- [ ] JSON schema defined
- [ ] Validation implemented
- [ ] Optional validation (performance)
- [ ] Clear error messages
- [ ] Tests for invalid messages

**Related Issues**: None

---

### ENHANCE-004: Implement Pretty-Printed JSON for Debugging

**Priority**: Low
**Labels**: `enhancement`, `area:protocol`, `priority:low`, `debugging`

**Description**:
Add option to pretty-print JSON messages for easier debugging.

**Source**: Item #72
**Files**: JSON protocol implementation

**Acceptance Criteria**:

- [ ] Pretty-print option added
- [ ] Configurable indentation
- [ ] Debug logging integration
- [ ] Examples showing usage
- [ ] Documentation

**Related Issues**: None

---

### ENHANCE-005: Implement JSON Compression

**Priority**: Low
**Labels**: `enhancement`, `area:protocol`, `priority:low`, `performance`

**Description**:
Add compression support for JSON messages to reduce bandwidth.

**Source**: Item #73
**Files**: JSON protocol, transport layer

**Acceptance Criteria**:

- [ ] Compression implemented (gzip)
- [ ] Automatic negotiation
- [ ] Performance benchmarks
- [ ] Tests with compressed messages
- [ ] Documentation

**Related Issues**: ENHANCE-006 (TCP compression)

---

### ENHANCE-006: Implement gzip Compression for TCP

**Priority**: Low
**Labels**: `enhancement`, `area:networking`, `priority:low`, `performance`

**Description**:
Add gzip compression support for TCP transport to reduce bandwidth usage.

**Source**: Item #96
**Files**: Transport layer

**Acceptance Criteria**:

- [ ] gzip compression for TCP
- [ ] Configurable compression level
- [ ] Performance benchmarks
- [ ] Tests with compression
- [ ] Documentation

**Related Issues**: ENHANCE-005 (JSON compression)

---

### ENHANCE-007: Implement Connection Pooling

**Priority**: Low
**Labels**: `enhancement`, `area:networking`, `priority:low`, `performance`

**Description**:
Implement connection pooling for efficient reuse of server connections.

**Source**: Item #95
**Files**: Client implementation, transport layer

**Acceptance Criteria**:

- [ ] Connection pool implemented
- [ ] Configurable pool size
- [ ] Connection lifecycle management
- [ ] Tests with pooling
- [ ] Performance benchmarks

**Related Issues**: FEATURE-015 (Reconnection)

---

### ENHANCE-008: Implement Metrics for TCP Transport

**Priority**: Low
**Labels**: `enhancement`, `area:networking`, `priority:low`, `observability`

**Description**:
Add metrics collection for TCP transport (bytes sent/received, latency, errors).

**Source**: Item #98
**Files**: Transport layer

**Acceptance Criteria**:

- [ ] Metrics collection implemented
- [ ] Configurable metrics backend
- [ ] Common metrics exposed
- [ ] Examples showing usage
- [ ] Documentation

**Related Issues**: None

---

### ENHANCE-009: Implement Backpressure for TCP

**Priority**: Low
**Labels**: `enhancement`, `area:networking`, `priority:low`, `reliability`

**Description**:
Implement backpressure mechanism to handle slow consumers gracefully.

**Source**: Item #99
**Files**: Transport layer

**Acceptance Criteria**:

- [ ] Backpressure mechanism implemented
- [ ] Configurable buffer sizes
- [ ] Tests with slow consumers
- [ ] Documentation on behavior

**Related Issues**: None

---

### ENHANCE-010: Implement Filtered Auto-Discovery

**Priority**: Low
**Labels**: `enhancement`, `area:discovery`, `priority:low`, `filtering`

**Description**:
Add filtering options for auto-discovery (by device type, name pattern, etc.).

**Source**: Item #89
**Files**: Discovery module

**Acceptance Criteria**:

- [ ] Filter by device type
- [ ] Filter by name pattern
- [ ] Filter by properties
- [ ] Examples showing filters
- [ ] Documentation

**Related Issues**: FEATURE-013 (Type detection)

---

### ENHANCE-011: Implement Smart Server Selection

**Priority**: Low
**Labels**: `enhancement`, `area:discovery`, `priority:low`, `intelligence`

**Description**:
Implement smart server selection based on criteria (latency, load, capabilities).

**Source**: Item #90
**Files**: Discovery module, client

**Acceptance Criteria**:

- [ ] Selection criteria defined
- [ ] Selection algorithm implemented
- [ ] Configurable preferences
- [ ] Tests with multiple servers
- [ ] Documentation

**Related Issues**: FEATURE-013

---

### ENHANCE-012: Implement Persistent Server Cache

**Priority**: Low
**Labels**: `enhancement`, `area:discovery`, `priority:low`, `caching`

**Description**:
Cache discovered servers persistently for faster reconnection.

**Source**: Item #91
**Files**: Discovery module

**Acceptance Criteria**:

- [ ] Server cache implemented
- [ ] Persistent storage (file/db)
- [ ] Cache expiration
- [ ] Tests with cache
- [ ] Documentation

**Related Issues**: FEATURE-015 (Reconnection)

---

### ENHANCE-013: Implement Service Announcement

**Priority**: Low
**Labels**: `enhancement`, `area:discovery`, `priority:low`, `networking`

**Description**:
Implement service announcement for devices to advertise themselves.

**Source**: Item #93
**Files**: Discovery module, device API

**Acceptance Criteria**:

- [ ] Service announcement implemented
- [ ] mDNS advertisement
- [ ] Configurable service info
- [ ] Tests with announcement
- [ ] Documentation

**Related Issues**: FEATURE-001 (Device Driver API)

---

## 📊 Summary

### Total Issues by Priority

| Priority | Count | Percentage |
|----------|-------|------------|
| Critical | 3 | 6% |
| High | 16 | 32% |
| Medium | 20 | 39% |
| Low | 12 | 24% |
| **Total** | **51** | **100%** |

### Issues by Category

| Category | Count | Key Focus Areas |
|----------|-------|-----------------|
| **Bugs** | 4 | Property conversions, test infrastructure, code quality |
| **Critical Features** | 3 | Device API, ZeroConf, FFI integration |
| **High Priority Features** | 7 | BLOB handling, examples, refactoring, networking |
| **Medium Priority Features** | 7 | Metadata, discovery, reconnection, security |
| **Testing & CI/CD** | 9 | Test harness, integration tests, CI infrastructure |
| **Documentation** | 8 | API docs, organization, user/dev guides |
| **Technical Debt** | 8 | Version management, warnings, refactoring |
| **Automation** | 4 | Constant updates, code generation |
| **Platform-Specific** | 2 | BSD support, dependency complexity |
| **Research** | 2 | GUI frameworks, trait design |
| **Performance** | 1 | Profiling and optimization |
| **Low Priority Enhancements** | 13 | WebSocket, compression, metrics, caching |

### Recommended Creation Order

#### Phase 1: Critical Bugs & Infrastructure (Week 1-2)

1. **BUG-001**: Property conversions (Critical - safety issue)
2. **BUG-002**: Tokio runtime nesting (High - blocks testing)
3. **BUG-003**: Test isolation (High - test reliability)
4. **CI-001**: Integration tests in CI (High - infrastructure)
5. **TEST-002**: Test harness (High - tracking issue)

#### Phase 2: Core Features (Week 3-4)

6. **FEATURE-001**: Device Driver API (Critical - core feature)
2. **FEATURE-002**: Trait-Based Device API (Critical - tracking issue)
3. **FEATURE-003**: ZeroConf backend (Critical - networking)
4. **FEATURE-004**: FFI integration (High - compatibility)
5. **FEATURE-005**: BLOB handling (High - data transfer)

#### Phase 3: High Priority Items (Week 5-6)

11. **FEATURE-006**: Update examples (High - user experience)
2. **FEATURE-007**: Interface enum generation (High - refactoring)
3. **FEATURE-008**: Consolidate features (High - configuration)
4. **FEATURE-009**: Clean up lib.rs (High - code quality)
5. **DOC-001**: API documentation (High - docs.rs)
6. **DOC-002**: Documentation organization (High - structure)
7. **TEST-001**: Integration test coverage (High - quality)

#### Phase 4: Medium Priority (Week 7-10)

18-38. All Medium priority issues across categories:
    - Features (FEATURE-011 through FEATURE-019)
    - CI/CD (CI-002, CI-003, CI-005)
    - Documentation (DOC-007)
    - Technical Debt (DEBT-001 through DEBT-007)
    - Automation (AUTOMATION-001)
    - Platform (PLATFORM-002)
    - Performance (PERF-001)
    - **TEST-004**: Create Runnable Example Tests for Test Framework

#### Phase 5: Low Priority & Enhancements (Ongoing)

39-51. All Low priority issues:
    - Remaining bugs (BUG-004)
    - Documentation (DOC-003, DOC-008)
    - Technical Debt (DEBT-004, DEBT-008)
    - Automation (CODEGEN-001 through CODEGEN-004)
    - Platform (PLATFORM-001)
    - Research (RESEARCH-001, RESEARCH-002)
    - Enhancements (ENHANCE-001 through ENHANCE-013)
    - CI/CD (CI-004)

### Items Excluded from Issue Creation

The following items were excluded as too vague, speculative, or requiring further research:

- **Item #36**: Distributed testing (too speculative)
- **Item #37**: Chaos testing (too speculative)
- **Item #38**: Automated performance regression detection (covered by PERF-001)
- **Item #78**: Device Capability Detection (covered by FEATURE-013)
- **Item #79**: Automatic Property Polling (covered by FEATURE-002)
- **Item #80**: Event-Driven Updates (covered by FEATURE-002)
- **Item #81**: Device Simulation (covered by TEST-003)
- **Item #82**: Code Generation for traits (covered by CODEGEN series)
- **Item #83**: Advanced Features for device API (too vague)
- **Item #88**: Auto-discovery and connection (covered by FEATURE-003)

### Notes on Consolidation

Several items were consolidated into single issues:

- **Items #10, #24**: Combined into TEST-001 (Integration test coverage)
- **Items #25-43**: Consolidated into TEST-002 (Test harness tracking issue)
- **Items #44, #46, #48**: Combined into CI-002 (FFI build strategy)
- **Items #60, #87**: Duplicate - single issue created
- **Items #62, #101**: Duplicate - single issue created
- **Items #13, #102**: Duplicate - single issue created

### Implementation Strategy

1. **Start with Critical bugs** - These affect safety and reliability
2. **Build infrastructure** - Test harness and CI enable faster development
3. **Implement core features** - Device API and protocols are foundational
4. **Address technical debt** - Clean up while implementing features
5. **Enhance documentation** - Keep docs current with implementation
6. **Add optimizations** - Performance and enhancements come last

### Milestone Recommendations

- **v0.4.0**: Critical bugs + Core features (Issues 1-17)
- **v0.5.0**: Medium priority features + Documentation (Issues 18-38)
- **v1.0.0**: Low priority enhancements + Polish (Issues 39-51)

---

## Next Steps

1. **Review this document** with the team
2. **Adjust priorities** based on project goals
3. **Create GitHub issues** following the recommended order
4. **Assign milestones** (v0.4.0, v0.5.0, v1.0.0)
5. **Begin implementation** starting with Phase 1

---

*Document generated: 2026-03-09*
*Source: Comprehensive analysis of libindigo project documentation*
*Total items analyzed: 103 → Consolidated to: 51 actionable issues*