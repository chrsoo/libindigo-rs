# Planning Rules

## Directory Structure

```
plans/
├── README.md           # Index of all plans
├── active/             # Current work
├── archive/            # Completed work
└── standalone.md       # Not yet categorized
```

## When to Create a Plan

Create a plan for:
- New features or subsystems
- Significant architectural changes
- Complex refactoring
- Multi-step coordinated tasks

## Plan Structure

```markdown
# Plan Title

## Overview
Brief description

## Goals
Clear, measurable objectives

## Architecture/Design
Technical decisions and rationale

## Implementation Steps
1. Step 1
2. Step 2

## Testing Strategy
How to verify

## Success Criteria
- [ ] Criterion 1
- [ ] Criterion 2

## Status
[Planning | In Progress | Complete | Archived]
```

## Workflow

1. Create plan in `plans/`
2. Create GitHub **tracking issue** linking the plan
3. Work on feature branches (see [`.roorules/git.md`](.roorules/git.md))
4. Move to `plans/active/` when starting work
5. Move to `plans/archive/` when complete
6. Update status with completion date/commit

## Naming

- Use lowercase with hyphens: `feature-name.md`
- Be descriptive: `zeroconf-discovery-architecture.md`

## Commit References

Reference plans in commit **body**:

```text
feat: Implement Zeroconf discovery

See plans/zeroconf-discovery-architecture.md for details.

Closes #124
```

Update plans with commits:

```markdown
✅ Core discovery API - commit abc1234
```
