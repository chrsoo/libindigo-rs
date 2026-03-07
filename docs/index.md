# libindigo Documentation

Welcome to the libindigo documentation. This directory contains comprehensive documentation for the libindigo Rust library for INDIGO protocol clients.

## Quick Links

- [GitHub Setup Instructions](github-setup-instructions.md) - Set up GitHub issue templates and workflows
- [GitHub Issues Guide](github-issues-guide.md) - Create issues from planning documents
- [Roo Workflow Scheme](roo-workflow-scheme.md) - AI-assisted development workflow
- [Ways of Working](ways-of-working.md) - GitHub issue types and processes

## Documentation Structure

### Architecture

Technical architecture and design documents:

- [Client Strategies](architecture/client-strategies.md) - FFI vs Pure Rust implementation strategies

### Protocols

INDIGO protocol documentation:

- [JSON Protocol](protocols/json-protocol.md) - JSON protocol specification and usage
- [INDI Protocol](INDI.pdf) - Original INDI protocol specification (PDF)
- [Transport Implementation](transport_implementation.md) - TCP transport layer details

### Development

Development guides and processes:

- [Build Instructions](../BUILD.md) - How to build the project
- [Contributing Guidelines](../README.md#contributing) - How to contribute
- [Testing Guide](../tests/README.md) - Running and writing tests

### Project Management

- [CHANGES.md](../CHANGES.md) - Feature backlog and changelog
- [Planning Documents](../plans/README.md) - Active planning documents
- [Roorules](.roorules/) - AI assistant rules and guidelines

## Getting Started

### For Users

1. Read the [README](../README.md) for project overview
2. Check [BUILD.md](../BUILD.md) for build instructions
3. Review [examples/](../examples/) for usage examples
4. See [Client Strategies](architecture/client-strategies.md) to choose implementation

### For Contributors

1. Read [Ways of Working](ways-of-working.md) for GitHub workflow
2. Review [Roo Workflow Scheme](roo-workflow-scheme.md) for AI-assisted development
3. Check [GitHub Issues Guide](github-issues-guide.md) for creating issues
4. Follow [GitHub Setup Instructions](github-setup-instructions.md) for repository setup

### For Maintainers

1. Review [CHANGES.md](../CHANGES.md) for release planning
2. Check [plans/](../plans/) for active planning documents
3. Use [GitHub Issues Guide](github-issues-guide.md) to create tracking issues
4. Follow [Roo Workflow Scheme](roo-workflow-scheme.md) for coordinating work

## Architecture Overview

libindigo implements a dual-strategy pattern for INDIGO client connectivity:

### Pure Rust Strategy (Default)

- Zero C dependencies at runtime
- Pure Rust implementation of INDIGO protocol
- Supports both XML and JSON protocols
- Async-first design with tokio
- Cross-platform compatibility

**Use when**: You want zero FFI dependencies, modern async Rust, or web integration.

### FFI Strategy

- Uses upstream INDIGO C library
- Full hardware driver support
- Battle-tested C implementation
- Async wrapper around synchronous FFI

**Use when**: You need hardware driver support or proven C implementation.

See [Client Strategies](architecture/client-strategies.md) for details.

## Protocol Support

### XML Protocol (Version 2.0)

- Traditional INDIGO protocol
- Full feature support
- BASE64 BLOB encoding
- Supported by both strategies

### JSON Protocol (Version 512)

- Modern JSON-based protocol
- Better web integration
- URL-referenced BLOBs only
- Supported by Pure Rust strategy

See [JSON Protocol](protocols/json-protocol.md) for details.

## Key Features

### Implemented (v0.1.2)

- ✅ Dual strategy pattern (FFI and Pure Rust)
- ✅ XML protocol support
- ✅ JSON protocol support
- ✅ Protocol negotiation
- ✅ Async client API
- ✅ Property streaming
- ✅ BLOB support
- ✅ ZeroConf/Bonjour server discovery
- ✅ Integration test harness

### Planned (v0.2.0)

- 🚧 Automated property constants generation
- 🚧 Replace hardcoded property strings
- 🚧 Documentation organization

### Planned (v0.3.0)

- 📋 Trait-based device API (Camera, Mount, Focuser, etc.)
- 📋 Property wrappers
- 📋 Device-specific methods

See [CHANGES.md](../CHANGES.md) for complete roadmap.

## Testing

### Unit Tests

```bash
# Pure Rust strategy
cargo test --features rs --lib

# FFI strategy
cargo test --features ffi-strategy --lib
```

### Integration Tests

```bash
# Pure Rust strategy (requires INDIGO server)
cargo test --features rs --test '*'

# FFI strategy (requires INDIGO server)
cargo test --features ffi-strategy --test '*'
```

See [Testing Guide](../tests/README.md) for details.

## Examples

The `examples/` directory contains working examples:

- `discover_servers.rs` - Simple server discovery
- `continuous_discovery.rs` - Continuous monitoring
- `auto_connect.rs` - Auto-discover and connect
- `discovery_with_filter.rs` - Filtered discovery

Run examples:

```bash
cargo run --example discover_servers --features rs,auto
```

## API Documentation

Generate API documentation:

```bash
cargo doc --no-deps --features rs --open
```

## Support

- **Issues**: Use GitHub issue templates (see [GitHub Setup Instructions](github-setup-instructions.md))
- **Discussions**: Use GitHub Discussions for questions
- **Documentation**: Check this directory first

## Contributing

We welcome contributions! Please:

1. Read [Ways of Working](ways-of-working.md)
2. Follow [Roo Workflow Scheme](roo-workflow-scheme.md)
3. Use appropriate issue templates
4. Write tests for new features
5. Update documentation

## License

This project is licensed under the MIT License. See [LICENSE.md](../LICENSE.md) for details.

## References

### External Documentation

- [INDIGO Website](https://www.indigo-astronomy.org/)
- [INDIGO GitHub](https://github.com/indigo-astronomy/indigo)
- [INDIGO Documentation](https://github.com/indigo-astronomy/indigo/tree/master/indigo_docs)

### Internal Documentation

- [Planning Documents](../plans/README.md)
- [Archived Plans](../plans/archive/)
- [Test Documentation](../tests/)
- [Build Documentation](../BUILD.md)

---

**Last Updated**: 2026-03-05
**Version**: 0.2.0
