# Planning and Execution Rules

## Planning Workflow

This project uses a structured planning workflow to manage complex tasks and track progress effectively.

## Directory Structure

```
plans/
├── README.md                    # Index of all plans
├── active/                      # Current work in progress
│   └── feature-name.md
├── archive/                     # Completed work
│   └── completed-feature.md
└── standalone-plan.md           # Plans not yet categorized
```

## Creating Plans

### When to Create a Plan

Create a plan document when:

- Implementing a new feature or subsystem
- Making significant architectural changes
- Working on complex refactoring
- Coordinating multi-step tasks
- Documenting implementation strategies

### Plan Document Structure

Every plan should follow this structure:

```markdown
# Plan Title

## Overview

Brief description of what this plan covers.

## Goals

- Clear, measurable objectives
- What success looks like

## Architecture/Design

Technical design decisions and rationale.

## Implementation Steps

1. Step 1 with details
2. Step 2 with details
3. ...

## Testing Strategy

How to verify the implementation works.

## Success Criteria

- [ ] Criterion 1
- [ ] Criterion 2

## Status

Current status: [Planning | In Progress | Complete | Archived]

## References

- Related files
- External documentation
- Related issues
```

### Naming Conventions

- Use lowercase with hyphens: `feature-name.md`
- Be descriptive: `zeroconf-discovery-architecture.md` not `discovery.md`
- Include type if helpful: `ci-cd-strategy.md`, `integration-test-harness.md`

## Tracking Progress

### Active Plans

Plans currently being worked on should be in `plans/active/`:

```bash
git mv plans/feature-name.md plans/active/feature-name.md
```

### Completed Plans

When work is complete, move to `plans/archive/`:

```bash
git mv plans/active/feature-name.md plans/archive/feature-name.md
```

Update the plan's status section:

```markdown
## Status

✅ Complete - Implemented in [commit hash or date]
```

### Phase-Based Planning

For large projects with multiple phases:

1. Create phase documents: `phase1-foundation.md`, `phase2-implementation.md`
2. Track completion in each phase document
3. Archive completed phases to `plans/archive/`
4. Reference phase docs in commit messages

## Documentation in Plans

### Implementation Details

Plans should document:

- **What** is being implemented
- **Why** design decisions were made
- **How** the implementation works
- **Testing** strategy and verification
- **Future** improvements or considerations

### Code References

Link to relevant code with file paths and line numbers:

```markdown
See [`ClientStrategy`](src/client/strategy.rs:15) trait definition.
```

### Commit References

Reference related commits:

```markdown
Implemented in commit abc1234 (feat: Add discovery support)
```

## Integration with Git Workflow

### Commit Messages Reference Plans

When implementing from a plan, reference it in commits:

```
feat: Add zeroconf discovery support

Implements server discovery using mDNS/DNS-SD.
See plans/zeroconf-discovery-architecture.md for details.
```

### Plans Reference Commits

Update plans with implementation commits:

```markdown
## Implementation

✅ Core discovery API - commit abc1234
✅ Zeroconf backend - commit def5678
⏳ Discovery filters - in progress
```

## Plan Index (plans/README.md)

The `plans/README.md` file serves as an index of all plans:

```markdown
# Project Plans

## Active Plans

- [Feature Name](active/feature-name.md) - Brief description

## Archived Plans

- [Completed Feature](archive/completed-feature.md) - Brief description

## Standalone Plans

- [Strategy Document](strategy-name.md) - Brief description
```

Update this index when:

- Creating new plans
- Moving plans between directories
- Archiving completed work

## Best Practices

### Keep Plans Updated

- Update status as work progresses
- Add implementation notes and learnings
- Document deviations from original plan
- Link to related commits and PRs

### Plan Granularity

- **Too broad:** "Implement entire client library"
- **Too narrow:** "Add one function"
- **Just right:** "Implement zeroconf discovery subsystem"

### Cross-Referencing

Plans should reference:

- Related plans
- Relevant code files
- External documentation
- Issues and PRs

### Archiving Criteria

Archive a plan when:

- All success criteria are met
- Implementation is complete and tested
- Documentation is updated
- Code is merged to main branch

## Examples from This Project

### Good Plan Examples

- [`plans/zeroconf_discovery_architecture.md`](../plans/zeroconf_discovery_architecture.md) - Clear architecture and implementation steps
- [`plans/integration_test_harness_architecture.md`](../plans/integration_test_harness_architecture.md) - Comprehensive testing strategy
- [`plans/ci-cd-strategy.md`](../plans/ci-cd-strategy.md) - Detailed CI/CD approach

### Phase-Based Planning

- [`plans/archive/phase1-complete.md`](../plans/archive/phase1-complete.md) - Foundation work
- [`plans/archive/phase2-complete.md`](../plans/archive/phase2-complete.md) - FFI implementation
- [`plans/archive/phase3-complete.md`](../plans/archive/phase3-complete.md) - Pure Rust implementation

## Tools and Automation

### Creating a New Plan

```bash
# Create new plan
touch plans/my-feature.md

# Move to active when starting work
git mv plans/my-feature.md plans/active/my-feature.md

# Archive when complete
git mv plans/active/my-feature.md plans/archive/my-feature.md
```

### Searching Plans

```bash
# Find plans mentioning a topic
grep -r "discovery" plans/

# List all active plans
ls plans/active/

# List archived plans
ls plans/archive/
```

## Rationale

This planning workflow provides:

- **Clarity:** Clear documentation of what's being built and why
- **Tracking:** Easy to see what's active vs. complete
- **History:** Archived plans preserve decision-making context
- **Communication:** Plans serve as documentation for team members and AI assistants
- **Organization:** Structured approach prevents planning documents from cluttering the repository
