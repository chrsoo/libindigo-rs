# Project Documentation

## Directory Layout

- `docs/` human friendly documentation of the project not related to change
  - `architecture/` constraints on the solution design
  - `index.md` documentation entry point and overview of the project
- `plans/` documentation for supporting project change
- `README.md` for a quick overview and introduction to the project, links to `CHANGES.md` and `docs/index.md`
- `CHANGES.md` past and planned changes that links to `docs/` and `plans/`

## docs/

- Architecture documentation
- Technical specifications
- Reference material
- User documentation

## plans/

- Planned changes
- Design proposals

## README.md

## CHANGES.md

### Purpose

User-facing feature backlog and changelog organized by release version.

### Structure

- **User-facing features** → CHANGES.md (organized by version)
- **Implementation tasks** → GitHub Issues (with links to detailed `plans/`)
- **Technical documentation** → docs/ directory (architecture, design docs)

### Feature vs Task

- **Feature**: Provides value to library users (e.g., "JSON Protocol Support")
- **Task**: Internal work to deliver features (tracked in GitHub Issues)

### Workflow

1. **New Features**: Add to "Planned for X.X.X" section with GitHub Issue link
2. **New Tasks**: Create GitHub Issue, reference in CHANGES.md if user-visible
3. **Completed**: Move from "Planned" to version section (Added/Changed/Fixed/Removed)
4. **Release**: Move version from "Pending Release" to released with date

### GitHub Issues Integration

#### Creating Tasks

**Always create a GitHub Issue first** for any actionable work:

#### Referencing in CHANGES.md

Tasks should reference GitHub Issues, not detailed plans:

```markdown
## Planned for 0.X.0

### Tasks

- Brief task description ([#123](https://github.com/chrsoo/libindigo-rs/issues/123))
- Another task ([#124](https://github.com/chrsoo/libindigo-rs/issues/124))
```

#### Detailed Plans

GitHub Issues should link to detailed plans in `plans/`:

```markdown
## Implementation Plan
See detailed architecture in [`plans/feature-name.md`](plans/feature-name.md)
```

### Format

- Clear, user-understandable descriptions
- Focus on capabilities/benefits, not implementation
- Use INDIGO protocol terminology
- Present tense for unreleased, past tense for released

### Versioning

- Major (X.0.0): Breaking API changes
- Minor (0.X.0): New features, backward compatible
- Patch (0.0.X): Bug fixes, backward compatible