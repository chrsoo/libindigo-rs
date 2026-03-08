# Changes and Feature Backlog

This file tracks user-facing features organized by release version.

**For implementation tasks**: See [GitHub Issues](https://github.com/chrsoo/libindigo-rs/issues)
**For technical documentation**: See [`plans/`](plans/) directory

## [Unreleased]

### Planned for 0.3.0

#### Features

- **High-level Trait-based Device API**: Type-safe traits for common device types (Camera, Mount, Focuser, FilterWheel, etc.) with required and optional properties following INDIGO terminology ([#4](https://github.com/chrsoo/libindigo-rs/issues/4), [#12](https://github.com/chrsoo/libindigo-rs/issues/12))

### Planned for 0.2.0

#### Tasks

- Automate INDIGO constants extraction from headers ([#1](https://github.com/chrsoo/libindigo-rs/issues/1), [#10](https://github.com/chrsoo/libindigo-rs/issues/10))
- Replace hardcoded property strings with constants ([#2](https://github.com/chrsoo/libindigo-rs/issues/2))
- Complete documentation organization ([#3](https://github.com/chrsoo/libindigo-rs/issues/3), [#11](https://github.com/chrsoo/libindigo-rs/issues/11))

## [0.3.1] - 2026-03-08

### Fixed

- **Critical Property Streaming Bug**: Fixed bug in `property_receiver()` where the method consumed the channel receiver, preventing multiple concurrent subscribers from receiving property updates

### Added

- **Multiple Concurrent Subscribers**: New `subscribe_properties()` method that supports multiple concurrent subscribers to property updates using broadcast channels
- **Property Streaming Example**: Added comprehensive example in `examples/property_streaming.rs` demonstrating both single and multiple subscriber patterns
- **Documentation**: Updated `rs/README.md` with detailed property streaming guide and best practices
- **Issue Tracking**: Added 63 GitHub issues tracking known issues, planned features, and technical debt

### Deprecated

- **property_receiver() Method**: Deprecated in favor of `subscribe_properties()` which supports multiple concurrent subscribers. The old method will be removed in v0.4.0

## [0.2.0] - Pending Release

### Added

- **Pure Rust INDIGO Client Strategy**: Complete zero-FFI client implementation providing a fully cross-platform solution without C dependencies
- **XML Protocol Support**: Full INDIGO XML protocol parser and serializer supporting all message and property types including BLOB data
- **JSON Protocol Support**: Complete INDIGO JSON protocol implementation with automatic negotiation, prioritizing JSON for INDIGO 2.0+ servers with 20-30% faster parsing
- **Protocol Negotiation**: Automatic protocol version negotiation with fallback from JSON to XML for legacy server compatibility
- **TCP Transport Layer**: Robust connection management with message framing and error handling
- **ZeroConf/Bonjour Server Discovery**: Automatic detection of INDIGO servers on local network with both one-shot and continuous discovery modes
- **Discovery Event Streaming**: Real-time monitoring of server availability changes on the network
- **Property Event Streams**: Asynchronous channels for real-time property updates from devices
- **Async FFI Strategy**: Asynchronous wrapper around synchronous FFI calls using tokio::spawn_blocking
- **Builder Patterns**: Ergonomic construction for Client, DiscoveryConfig, and complex types
- **Comprehensive Testing**: Unit tests, protocol compliance tests (120+ JSON protocol tests), and integration test harness

### Changed

- Established idiomatic Rust API patterns throughout the codebase
- Improved error handling with detailed error types

## [0.1.2] - Initial Refactoring

### Added

- Foundation types for Property, Device, and values (Text, Number, Switch, Light, Blob)
- ClientStrategy trait for pluggable client implementations
- Basic FFI bindings to INDIGO C library
- Core project structure and build system
