# Project Documentation

## Directory Layout

- `docs/` - Human-friendly project documentation
  - `architecture/` - Solution design constraints
  - `index.md` - Documentation entry point and overview
- `plans/` - Documentation supporting project change
- `README.md` - Quick overview, links to `CHANGES.md` and `docs/index.md`
- `CHANGES.md` - Past and planned changes, links to `docs/` and `plans/`

## docs/

Contains stable, reference documentation:
- Architecture documentation
- Technical specifications
- Reference material
- User documentation

## plans/

Contains change-oriented documentation:
- Planned changes
- Design proposals
- Implementation plans

See [`.roorules/planning.md`](.roorules/planning.md) for details.

## README.md

Project overview and quick start guide.

## CHANGES.md

### Purpose

User-facing feature backlog and changelog organized by release version.

### Structure

- **User-facing features** → `CHANGES.md` (organized by version)
- **Implementation tasks** → GitHub Issues (with links to `plans/`)
- **Technical documentation** → `docs/` directory

### Feature vs Task

- **Feature**: Provides value to library users (e.g., "JSON Protocol Support")
- **Task**: Internal work to deliver features (tracked in GitHub Issues)

### Workflow

1. **New Features**: Add to "Planned for X.X.X" section with GitHub Issue link
2. **New Tasks**: Create GitHub Issue, reference in `CHANGES.md` if user-visible
3. **Completed**: Move from "Planned" to version section (Added/Changed/Fixed/Removed)
4. **Release**: Move version from "Pending Release" to released with date

### Format

- Clear, user-understandable descriptions
- Focus on capabilities/benefits, not implementation
- Use INDIGO protocol terminology
- Present tense for unreleased, past tense for released

### Versioning

- Major (X.0.0): Breaking API changes
- Minor (0.X.0): New features, backward compatible
- Patch (0.0.X): Bug fixes, backward compatible

### Example

```markdown
## [0.3.0] - Planned

### Features
- **High-level Device API**: Type-safe traits for cameras, mounts, etc. ([#12](https://github.com/chrsoo/libindigo-rs/issues/12))

### Tasks
- Automate constants extraction ([#10](https://github.com/chrsoo/libindigo-rs/issues/10))
- Documentation organization ([#11](https://github.com/chrsoo/libindigo-rs/issues/11))
```
