# Changes and Feature Backlog

This file tracks user-facing features organized by release version. For implementation tasks and technical work, see the `plans/` directory.

## [Unreleased]

### Planned for 0.3.0

#### Features

- **High-level Trait-based Device API**: Implement a trait-based API for common device types (Camera, Mount, Focuser, FilterWheel, etc.) with their required and optional properties following INDIGO terminology. This provides type-safe, ergonomic access to device-specific functionality.

### Planned for 0.2.0

#### Tasks (see plans/ for details)

- Generate props.rs from upstream INDIGO headers as part of build process
- Replace hardcoded property name strings with generated constants throughout codebase
- Extract comprehensive documentation from plans/ to doc/ directory for better organization

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
