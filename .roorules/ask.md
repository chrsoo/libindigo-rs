# Ask Mode Rules

## Purpose

Ask mode is designed for explaining, documenting, and answering questions about the codebase. It focuses on understanding and communication rather than implementation.

## When to Use Ask Mode

Use Ask mode when:
- Need to understand existing code
- Researching error messages or patterns
- Writing user-facing documentation
- Creating examples and tutorials
- Explaining implementations
- Answering technical questions
- Reviewing code for clarity

## Primary Responsibilities

1. **Explain Code**: Describe how code works and why
2. **Write Documentation**: Create user guides and API docs
3. **Create Examples**: Write example code and tutorials
4. **Research**: Investigate existing patterns and best practices
5. **Review**: Assess code clarity and documentation quality
6. **Answer Questions**: Provide technical explanations

## Workflow Pattern

```
1. Analyze the question or documentation need
2. Research relevant code and context
3. Explain concepts clearly and accurately
4. Provide examples where helpful
5. Create or update documentation
6. Verify accuracy and completeness
```

## File Restrictions

Ask mode can edit:
- Documentation files in doc/ (*.md)
- README.md
- Example files in examples/
- Inline documentation (doc comments)
- CHANGES.md (for documentation entries)

Ask mode should NOT edit:
- Source code implementation (use Code)
- Planning documents in plans/ (use Architect)
- Test implementations (use Code)

## Documentation Best Practices

### User-Facing Documentation

```markdown
# Feature Name

## Overview
Brief description of what this feature does and why it's useful.

## Quick Start
Minimal example to get started quickly.

## Usage
Detailed usage instructions with examples.

## Configuration
Available options and how to configure them.

## Examples
Complete, runnable examples.

## Troubleshooting
Common issues and solutions.

## API Reference
Link to generated API docs.
```

### API Documentation

```rust
/// Discovers INDIGO servers on the local network using Zeroconf/mDNS.
///
/// This function performs a one-shot discovery, returning all servers
/// found within the specified timeout period.
///
/// # Arguments
///
/// * `timeout` - Maximum time to wait for server responses
///
/// # Returns
///
/// A vector of discovered servers, or an error if discovery fails.
///
/// # Examples
///
/// ```
/// use libindigo::discovery::discover_servers;
/// use std::time::Duration;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let servers = discover_servers(Duration::from_secs(5)).await?;
/// for server in servers {
///     println!("Found server: {}", server.name);
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns `DiscoveryError::Timeout` if no servers are found within
/// the timeout period.
pub async fn discover_servers(timeout: Duration) -> Result<Vec<Server>, DiscoveryError> {
    // Implementation
}
```

### Example Code

```rust
//! # Zeroconf Discovery Example
//!
//! This example demonstrates how to discover INDIGO servers on the
//! local network using Zeroconf/mDNS.

use libindigo::discovery::{discover_servers, DiscoveryConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure discovery
    let config = DiscoveryConfig::builder()
        .timeout(Duration::from_secs(5))
        .service_type("_indigo._tcp")
        .build();

    // Discover servers
    println!("Discovering INDIGO servers...");
    let servers = discover_servers(config).await?;

    // Display results
    println!("Found {} servers:", servers.len());
    for server in servers {
        println!("  - {} at {}:{}", server.name, server.host, server.port);
    }

    Ok(())
}
```

## Integration with GitHub Issues

Ask mode supports all issue types for documentation:
- **✨ Enhancement**: Document new features
- **🛠️ Chore**: Update documentation
- **🐛 Bug**: Explain fixes
- **🔍 Discussion**: Research and explain concepts

## Handoff Protocol

### From Code to Ask

```
Code: "Feature #124 is complete. Please use Ask mode to document
the Zeroconf discovery feature in the user guide and create an
example."

Ask:
1. Reviews implementation in src/discovery/zeroconf_impl.rs
2. Adds API documentation to code
3. Creates example in examples/zeroconf_discovery.rs
4. Updates README.md with feature description
5. Updates CHANGES.md
6. Closes documentation task
```

### From Ask to Architect

```
Ask: "I've analyzed the discovery API. The DiscoveryStrategy trait
provides a clean extension point. Please use Architect mode to
design how Zeroconf discovery will implement this trait."

Architect: Creates design based on research findings
```

### From Any Mode to Ask

```
User: "Before implementing #124, I need to understand the existing
discovery API. Please explain how src/discovery/api.rs works."

Ask:
1. Analyzes src/discovery/api.rs
2. Explains DiscoveryStrategy trait
3. Shows how it's used in the codebase
4. Provides examples of existing implementations
5. Recommends approach for new implementation
```

## Documentation Checklist

Before completing documentation:

- [ ] Explanation is clear and accurate
- [ ] Examples are complete and runnable
- [ ] Code snippets are tested
- [ ] Links are valid
- [ ] Terminology is consistent
- [ ] Audience level is appropriate
- [ ] Common questions are addressed
- [ ] Related documentation is linked

## Best Practices

✅ **DO:**
- Write for your audience (users vs developers)
- Provide complete, runnable examples
- Explain the "why" not just the "what"
- Use consistent terminology
- Link to related documentation
- Test code examples
- Keep documentation up-to-date
- Use clear, simple language

❌ **DON'T:**
- Assume prior knowledge
- Use jargon without explanation
- Provide incomplete examples
- Skip error handling in examples
- Forget to update docs when code changes
- Write overly technical explanations for users
- Leave broken links
- Duplicate information

## Example Usage

### Example 1: Documenting New Feature

```
User: "Please document the Zeroconf discovery feature."

Ask:
1. Reviews implementation:
   - src/discovery/zeroconf_impl.rs
   - src/discovery/api.rs
   - tests/discovery_tests.rs

2. Adds API documentation:
   /// Discovers INDIGO servers using Zeroconf/mDNS.
   ///
   /// # Examples
   /// ```
   /// let servers = discover_servers(timeout).await?;
   /// ```

3. Creates example file examples/zeroconf_discovery.rs:
   - Complete, runnable example
   - Shows configuration options
   - Demonstrates error handling

4. Updates README.md:
   ## Features
   - **Zeroconf Discovery**: Automatic server detection
     using mDNS/Bonjour

5. Updates CHANGES.md:
   - ZeroConf/Bonjour Server Discovery: Automatic
     detection of INDIGO servers on local network

6. Closes documentation task
```

### Example 2: Explaining Existing Code

```
User: "How does the discovery API work?"

Ask:
1. Analyzes src/discovery/api.rs

2. Explains structure:
   "The discovery API is built around the DiscoveryStrategy
   trait, which defines how different discovery methods work.

   Key components:
   - DiscoveryStrategy trait: Defines discovery interface
   - DiscoveryConfig: Configuration for discovery
   - DiscoveredServer: Represents a found server

   The trait has two main methods:
   - discover_once(): One-shot discovery
   - discover_continuous(): Continuous monitoring

   Implementations:
   - ZeroconfDiscovery: Uses mDNS/Bonjour
   - ManualDiscovery: Uses configured addresses"

3. Shows usage example:
   ```rust
   let strategy = ZeroconfDiscovery::new(config);
   let servers = strategy.discover_once().await?;
   ```

4. Recommends next steps based on user's goal
```

### Example 3: Creating Tutorial

```
User: "Create a getting started guide for the discovery feature."

Ask:
1. Creates doc/guides/discovery-getting-started.md

2. Structures tutorial:
   # Getting Started with Discovery

   ## Installation
   Add to Cargo.toml...

   ## Quick Start
   Minimal example...

   ## Configuration
   Available options...

   ## Advanced Usage
   Continuous discovery...

   ## Troubleshooting
   Common issues...

3. Includes complete examples:
   - Basic discovery
   - With configuration
   - Continuous monitoring
   - Error handling

4. Tests all examples to ensure they work

5. Links from README.md and main documentation
```

## Code Explanation Patterns

### Explaining Architecture

```
The discovery system uses a strategy pattern:

1. DiscoveryStrategy trait defines the interface
2. Concrete strategies implement specific methods:
   - ZeroconfDiscovery: mDNS/Bonjour
   - ManualDiscovery: Configured addresses
3. Client uses strategies through the trait

This allows:
- Easy addition of new discovery methods
- Runtime selection of strategy
- Testing with mock strategies
```

### Explaining Algorithms

```
The protocol negotiation works as follows:

1. Client connects and sends supported protocols
2. Server responds with chosen protocol
3. If JSON is supported by both, use JSON
4. Otherwise, fall back to XML
5. If no common protocol, connection fails

This ensures:
- Modern servers use efficient JSON
- Legacy servers still work with XML
- Clear error if incompatible
```

### Explaining Design Decisions

```
We use trait objects (Box<dyn Trait>) instead of generics
for the discovery API because:

Advantages:
- Simpler API for users
- Runtime strategy selection
- Smaller binary size

Trade-offs:
- Small runtime overhead
- Dynamic dispatch instead of monomorphization

This is appropriate because discovery is not performance-
critical and API simplicity is more important.
```

## Documentation Types

### User Documentation
- Getting started guides
- Feature overviews
- Configuration guides
- Troubleshooting guides
- Migration guides

### Developer Documentation
- Architecture overviews
- API references
- Contributing guides
- Design decisions
- Implementation notes

### Examples
- Quick start examples
- Feature demonstrations
- Integration examples
- Best practices
- Common patterns

## Related Documentation

- [Roo Workflow Scheme](../doc/roo-workflow-scheme.md) - Complete workflow guide
- [Ways of Working](../doc/ways-of-working.md) - GitHub issue types
- [Rust Documentation Guidelines](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html)
