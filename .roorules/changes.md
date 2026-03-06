# CHANGES.md Management

## Purpose

User-facing feature backlog and changelog organized by release version.

## Structure

- **User-facing features** → CHANGES.md (organized by version)
- **Implementation tasks** → GitHub Issues (with links to detailed plans)
- **Technical documentation** → plans/ directory (architecture, design docs)

## Feature vs Task

- **Feature**: Provides value to library users (e.g., "JSON Protocol Support")
- **Task**: Internal work to deliver features (tracked in GitHub Issues)

## Workflow

1. **New Features**: Add to "Planned for X.X.X" section with GitHub Issue link
2. **New Tasks**: Create GitHub Issue, reference in CHANGES.md if user-visible
3. **Completed**: Move from "Planned" to version section (Added/Changed/Fixed/Removed)
4. **Release**: Move version from "Pending Release" to released with date

## GitHub Issues Integration

### Creating Tasks

**Always create a GitHub Issue first** for any actionable work:

```bash
gh issue create --title "Task title" --body "Description" --label "enhancement"
```

### Referencing in CHANGES.md

Tasks should reference GitHub Issues, not detailed plans:

```markdown
### Planned for 0.X.0

#### Tasks

- Brief task description ([#123](https://github.com/chrsoo/libindigo-rs/issues/123))
- Another task ([#124](https://github.com/chrsoo/libindigo-rs/issues/124))
```

### Detailed Plans

GitHub Issues should link to detailed plans in `plans/`:

```markdown
## Implementation Plan
See detailed architecture in [`plans/feature-name.md`](plans/feature-name.md)
```

## Format

- Clear, user-understandable descriptions
- Focus on capabilities/benefits, not implementation
- Use INDIGO protocol terminology
- Present tense for unreleased, past tense for released

## Versioning

- Major (X.0.0): Breaking API changes
- Minor (0.X.0): New features, backward compatible
- Patch (0.0.X): Bug fixes, backward compatible
