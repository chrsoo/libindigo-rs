# Documentation Organization

## Overview

Extract and organize comprehensive documentation from the plans/ directory into the doc/ directory for better accessibility and structure. The plans/ directory contains valuable technical documentation that should be more prominently available.

## Current State

- Technical documentation scattered across plans/ directory
- Some documentation in doc/ (INDI.pdf, transport_implementation.md)
- Plans contain both active planning documents and completed documentation
- No clear organization for user-facing vs developer documentation

## Goals

1. Organize documentation into clear categories (user guides, architecture, protocols, etc.)
2. Extract completed technical documentation from plans/ to doc/
3. Keep active planning documents in plans/
4. Create a documentation index for easy navigation
5. Ensure documentation is up-to-date with current implementation

## Documentation Categories

### User Documentation (doc/)

- Getting Started Guide
- API Reference (can be generated from rustdoc)
- Examples and Tutorials
- Protocol Overview

### Architecture Documentation (doc/architecture/)

- System Architecture
- Client Strategies (FFI vs Pure Rust)
- Protocol Implementation (XML/JSON)
- Transport Layer Design
- Discovery System Design

### Developer Documentation (doc/development/)

- Build Instructions (from BUILD.md)
- Testing Guide (integration test harness)
- Contributing Guidelines
- Code Organization

### Protocol Documentation (doc/protocols/)

- INDIGO Protocol Overview
- XML Protocol Details
- JSON Protocol Details
- Protocol Negotiation

## Implementation Approach

### Phase 1: Audit

- [ ] Review all files in plans/ directory
- [ ] Identify completed documentation vs active planning
- [ ] Categorize documentation by type
- [ ] Identify gaps in documentation

### Phase 2: Structure

- [ ] Create doc/ subdirectories (architecture/, development/, protocols/)
- [ ] Create doc/README.md as documentation index
- [ ] Define naming conventions for documentation files

### Phase 3: Extract and Organize

- [ ] Move completed architecture docs from plans/ to doc/architecture/
  - Transport implementation
  - Client strategies
  - Discovery system
  - Protocol implementation
- [ ] Create protocol documentation in doc/protocols/
- [ ] Move/copy BUILD.md content to doc/development/
- [ ] Create getting started guide in doc/

### Phase 4: Update and Polish

- [ ] Update extracted documentation to reflect current state
- [ ] Add cross-references between documents
- [ ] Ensure code examples are current
- [ ] Add diagrams where helpful

### Phase 5: Index

- [ ] Create comprehensive doc/README.md with links to all documentation
- [ ] Update main README.md to reference doc/ directory
- [ ] Add documentation section to CHANGES.md

## Files to Process

From plans/:

- zeroconf_discovery_architecture.md → doc/architecture/discovery.md
- json-protocol-implementation.md → doc/protocols/json-protocol.md
- integration_test_harness_architecture.md → doc/development/testing.md
- code-review-and-architecture.md → doc/architecture/overview.md

Keep in plans/:

- Active planning documents (issues.md, ci-cd-strategy.md, etc.)
- Future feature plans
- README.md (plans index)

## Success Criteria

- Clear documentation structure in doc/ directory
- All completed technical documentation extracted from plans/
- Comprehensive documentation index (doc/README.md)
- Documentation is current and accurate
- Easy navigation between related documents
- Plans/ contains only active planning documents

## Future Enhancements

- Generate API documentation from rustdoc
- Add tutorials and examples documentation
- Create architecture diagrams
- Add troubleshooting guide
