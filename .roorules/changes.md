# CHANGES.md Management Rules

## Purpose

The CHANGES.md file serves as a user-facing feature backlog and changelog. It tracks what features are available in each release and what's planned for future releases.

## Structure

- **User-facing features** go in CHANGES.md organized by release version
- **Implementation tasks** go in the plans/ directory as detailed technical documents
- CHANGES.md should reference plans/ for task details but not duplicate them

## Feature vs Task

- **Feature**: Something that provides value to library users (e.g., "JSON Protocol Support", "Server Discovery API")
- **Task**: Internal work needed to deliver features (e.g., "Refactor transport layer", "Add unit tests")

## Workflow

1. **New Features**: Add to the "Planned for X.X.X" section under "Features"
2. **New Tasks**: Add brief entry under "Tasks (see plans/ for details)" and create detailed plan in plans/ directory
3. **Completed Work**: Move from "Planned" to the version section under "Added", "Changed", "Fixed", or "Removed"
4. **Release**: When releasing, move the version from "Pending Release" to released with date

## Format Guidelines

- Use clear, concise descriptions that users can understand
- Focus on capabilities and benefits, not implementation details
- Group related features together
- Use consistent terminology matching INDIGO protocol documentation
- Keep entries in present tense for unreleased, past tense for released

## Version Numbering

Follow semantic versioning:
- Major (X.0.0): Breaking API changes
- Minor (0.X.0): New features, backward compatible
- Patch (0.0.X): Bug fixes, backward compatible

## Examples

Good feature entry:
- "ZeroConf/Bonjour Server Discovery: Automatic detection of INDIGO servers on local network"

Good task entry:
- "Extract well-known Device and Property names from upstream INDIGO"

Bad (too technical for CHANGES.md):
- "Refactor protocol parser to use nom combinators for better performance"
