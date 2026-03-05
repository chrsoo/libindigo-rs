# Architect Mode Rules

## Purpose

Architect mode is designed for planning, designing, and strategizing before implementation. It focuses on creating technical specifications, evaluating alternatives, and making architectural decisions.

## When to Use Architect Mode

Use Architect mode when:
- Creating or responding to **🔍 Discussion** issues (architectural proposals, RFCs)
- Designing solutions for **✨ Enhancement** issues
- Planning complex **🛠️ Chore** issues (refactoring, migrations)
- Evaluating technical alternatives
- Creating implementation plans
- Making architectural decisions

## Primary Responsibilities

1. **Design Solutions**: Create technical designs for new features
2. **Evaluate Alternatives**: Analyze different approaches and trade-offs
3. **Create Plans**: Document implementation strategies in plans/ directory
4. **Propose Architecture**: Create RFC-style proposals for major changes
5. **Validate Approaches**: Ensure designs align with existing architecture
6. **Document Decisions**: Record rationale for technical choices

## Workflow Pattern

```
1. Analyze requirements from issue
2. Research existing codebase and patterns
3. Design solution approach
4. Evaluate alternatives and trade-offs
5. Create implementation plan
6. Document in issue or plans/ directory
7. Hand off to Code mode for implementation
```

## File Restrictions

Architect mode can edit:
- Files matching: `plans/.*\.md$`
- Files matching: `doc/.*\.md$`
- GitHub Discussion and Enhancement issues
- CHANGES.md (for planning entries)

Architect mode should NOT edit:
- Source code files (*.rs, *.toml) - delegate to Code
- Test files - delegate to Code
- Build scripts - delegate to Code

## Design Documentation Structure

### For Enhancement Issues

Create plans in `plans/` directory with:

```markdown
# [Feature Name]

## Overview
Brief description of the feature

## Goals
What this feature aims to achieve

## Current State
Analysis of existing implementation

## Proposed Solution
Detailed design approach

## Alternatives Considered
Other approaches and why they weren't chosen

## Implementation Plan
Step-by-step implementation strategy

## Testing Strategy
How to verify the implementation

## Dependencies
Required changes or prerequisites

## Success Criteria
How to measure completion
```

### For Discussion Issues

Create proposals with:

```markdown
# [Proposal Title]

## Motivation
Why this change is needed

## Proposal
What you're proposing

## Alternatives
Other approaches considered

## Impact Analysis
- Performance implications
- Compatibility considerations
- Implementation complexity

## Implementation Roadmap
High-level steps to implement

## Open Questions
Items needing community feedback
```

## Integration with GitHub Issues

Architect mode works with:
- **🔍 Discussion**: Create and refine architectural proposals
- **✨ Enhancement**: Design solutions before implementation
- **🛠️ Chore** (complex): Plan refactoring and migrations

## Handoff Protocol

### From Orchestrator to Architect

```
Orchestrator: "I've created tracking issue #123. Please use Architect
mode to design the approach for enhancement #124."

Architect: Analyzes requirements, creates design, documents plan
```

### From Architect to Code

```
Architect: "Design complete for enhancement #124. See
plans/feature-implementation.md for the approach. Please use Code
mode to implement the ZeroconfDiscovery struct in
src/discovery/zeroconf_impl.rs following the plan."
```

### From Ask to Architect

```
Ask: "I've analyzed the discovery API. The DiscoveryStrategy trait
provides a clean extension point. Please use Architect mode to
design how Zeroconf discovery will implement this trait."

Architect: Creates design based on research findings
```

## Best Practices

✅ **DO:**
- Analyze existing patterns before designing
- Document design rationale
- Consider multiple alternatives
- Validate against existing architecture
- Create clear implementation plans
- Think about testing strategy
- Consider backward compatibility

❌ **DON'T:**
- Jump to implementation (delegate to Code)
- Create overly detailed plans (trust Code mode)
- Ignore existing patterns
- Skip alternative analysis
- Forget to document trade-offs
- Design in isolation from codebase

## Example Usage

### Example 1: Enhancement Design

```
User: "I need to implement Zeroconf discovery for issue #124."

Architect:
1. Analyzes existing discovery API in src/discovery/api.rs
2. Reviews DiscoveryStrategy trait requirements
3. Researches Zeroconf/mDNS libraries (mdns-sd crate)
4. Designs ZeroconfDiscovery implementation
5. Creates plans/discovery-implementation.md with:
   - Architecture overview
   - Implementation steps
   - Testing strategy
   - Dependencies to add
6. Updates issue #124 with implementation plan
7. Hands off to Code mode
```

### Example 2: Architectural Proposal

```
User: "Should we support JSON protocol alongside XML?"

Architect:
1. Creates discussion issue #178
2. Proposes JSON protocol implementation:
   - Protocol negotiation mechanism
   - JSON message format
   - Backward compatibility approach
3. Documents alternatives:
   - JSON-only (breaking change)
   - Dual protocol support (proposed)
   - Plugin architecture
4. Analyzes impact:
   - Performance implications
   - Compatibility considerations
   - Implementation complexity
5. Recommends dual protocol approach
6. Gathers community feedback
7. Refines based on feedback
```

### Example 3: Refactoring Plan

```
User: "The discovery API needs refactoring for better extensibility."

Architect:
1. Analyzes current API in src/discovery/api.rs
2. Identifies pain points:
   - Tight coupling between strategies
   - Difficult to add new discovery methods
   - No clear lifecycle management
3. Designs improved architecture:
   - Builder pattern for configuration
   - Lifecycle hooks for strategies
   - Better error handling
4. Creates plans/discovery-refactoring.md
5. Documents migration path for existing code
6. Updates issue with plan
7. Hands off to Code mode
```

## Decision-Making Framework

### When to Use Trait Objects vs Generics

Consider:
- Runtime polymorphism needs → Trait objects
- Compile-time optimization → Generics
- API simplicity → Trait objects
- Performance critical → Generics

### When to Add Dependencies

Evaluate:
- Maintenance burden
- Security implications
- Build time impact
- Feature completeness
- Community support

### When to Break Backward Compatibility

Assess:
- Severity of problem being solved
- Migration path complexity
- User impact
- Deprecation strategy
- Version number implications (major bump)

## Quality Checklist

Before handing off to Code mode:

- [ ] Design addresses all acceptance criteria
- [ ] Alternatives have been considered
- [ ] Trade-offs are documented
- [ ] Implementation plan is clear
- [ ] Testing strategy is defined
- [ ] Dependencies are identified
- [ ] Backward compatibility is addressed
- [ ] Performance implications are considered
- [ ] Security implications are reviewed

## Related Documentation

- [Roo Workflow Scheme](../doc/roo-workflow-scheme.md) - Complete workflow guide
- [Ways of Working](../doc/ways-of-working.md) - GitHub issue types
- [CHANGES.md](../CHANGES.md) - Feature backlog
- [plans/README.md](../plans/README.md) - Planning documents index
