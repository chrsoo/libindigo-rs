# Project Plans

This directory contains planning documents, architecture designs, and implementation strategies for the libindigo-rs project.

## Directory Structure

- **`active/`** - Plans for work currently in progress
- **`archive/`** - Completed plans and historical documentation
- **Root** - Standalone plans and strategies

## Active Plans

Currently, all active work is tracked in the root-level plans. Move plans to `active/` when starting implementation.

## Archived Plans

### Phase-Based Implementation

- [`phase1-complete.md`](archive/phase1-complete.md) - Foundation & Core Types
- [`phase2-complete.md`](archive/phase2-complete.md) - FFI Strategy Implementation
- [`phase3-client-implementation.md`](archive/phase3-client-implementation.md) - Client Implementation Details
- [`phase3-complete.md`](archive/phase3-complete.md) - Pure Rust Strategy Implementation
- [`phase3-json-complete.md`](archive/phase3-json-complete.md) - JSON Protocol Implementation

## Current Plans

### Architecture & Design

- [`code-review-and-architecture.md`](code-review-and-architecture.md) - Code review and architectural analysis
- [`crate-restructuring-architecture.md`](crate-restructuring-architecture.md) - Crate restructuring plan (v1)
- [`crate-restructuring-architecture-v2.md`](crate-restructuring-architecture-v2.md) - Crate restructuring plan (v2)
- [`crate-restructuring-architecture-v3.md`](crate-restructuring-architecture-v3.md) - Crate restructuring plan (v3)
- [`zeroconf_discovery_architecture.md`](zeroconf_discovery_architecture.md) - ZeroConf/Bonjour discovery API design

### Implementation Plans

- [`discovery-implementation.md`](discovery-implementation.md) - Server discovery implementation details
- [`json-protocol-implementation.md`](json-protocol-implementation.md) - JSON protocol implementation
- [`integration_test_harness_architecture.md`](integration_test_harness_architecture.md) - Test harness architecture

### Infrastructure & Operations

- [`ci-cd-strategy.md`](ci-cd-strategy.md) - CI/CD pipeline strategy and configuration
- [`integration-test-server-config.md`](integration-test-server-config.md) - Integration test server setup
- [`immediate-ci-fix.md`](immediate-ci-fix.md) - Immediate CI/CD fixes

### Issue Tracking

- [`issues.md`](issues.md) - Known issues and their resolutions

## Plan Lifecycle

1. **Create** - New plan created in root directory
2. **Activate** - Move to `active/` when starting implementation
3. **Complete** - Update status in document
4. **Archive** - Move to `archive/` when work is complete

See [`.roorules/planning.md`](../.roorules/planning.md) for detailed planning workflow guidelines.

## Creating New Plans

When creating a new plan:

1. Use descriptive kebab-case naming: `feature-name.md`
2. Follow the plan template structure (see `.roorules/planning.md`)
3. Include clear goals, architecture, and success criteria
4. Reference related code and documentation
5. Update this README when adding new plans

## Plan Categories

### Architecture Plans

Documents that define system design, component structure, and technical decisions.

### Implementation Plans

Step-by-step guides for implementing specific features or subsystems.

### Strategy Documents

High-level approaches for cross-cutting concerns (CI/CD, testing, etc.).

### Issue Documentation

Tracking and resolution of known problems and technical debt.

## Related Documentation

- [`.roorules/planning.md`](../.roorules/planning.md) - Planning workflow and best practices
- [`.roorules/git.md`](../.roorules/git.md) - Git commit conventions
- [`.roorules/markdown.md`](../.roorules/markdown.md) - Markdown formatting rules
- [`README.md`](../README.md) - Project overview
- [`BUILD.md`](../BUILD.md) - Build instructions
