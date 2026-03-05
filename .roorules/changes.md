# CHANGES.md Management

## Purpose

User-facing feature backlog and changelog organized by release version.

## Structure

- **User-facing features** → CHANGES.md (organized by version)
- **Implementation tasks** → plans/ directory (detailed technical documents)

## Feature vs Task

- **Feature**: Provides value to library users (e.g., "JSON Protocol Support")
- **Task**: Internal work to deliver features (e.g., "Refactor transport layer")

## Workflow

1. **New Features**: Add to "Planned for X.X.X" section
2. **New Tasks**: Brief entry in CHANGES.md + detailed plan in plans/
3. **Completed**: Move from "Planned" to version section (Added/Changed/Fixed/Removed)
4. **Release**: Move version from "Pending Release" to released with date

## Format

- Clear, user-understandable descriptions
- Focus on capabilities/benefits, not implementation
- Use INDIGO protocol terminology
- Present tense for unreleased, past tense for released

## Versioning

- Major (X.0.0): Breaking API changes
- Minor (0.X.0): New features, backward compatible
- Patch (0.0.X): Bug fixes, backward compatible
