# GitHub Issues Management

## Philosophy

**GitHub Issues are the single source of truth for all actionable work items.**

Use for: All actionable tasks, bug reports, feature requests, work tracking

**NOT for**: Project documentation (use `docs/` instead)

## MCP GitHub Tools

**Always use MCP GitHub tools.** Never use `gh` CLI.

| Operation | MCP Tool |
| --- | --- |
| List issues | `mcp--github--list_issues` |
| Search issues | `mcp--github--search_issues` |
| Create issue | `mcp--github--create_issue` |
| View details | `mcp--github--get_issue` |
| Update issue | `mcp--github--update_issue` |
| Add comment | `mcp--github--add_issue_comment` |
| Create PR | `mcp--github--create_pull_request` |

## Workflow

### 1. Creating Work Items

**Always create a GitHub Issue first** using `mcp--github--create_issue`.

**Never** create tasks in:
- ❌ `docs/*.md` files
- ❌ `plans/*.md` files
- ❌ `CHANGES.md` task sections
- ❌ Local TODO files
- ❌ Code comments with TODO/FIXME

### 2. Issue Types

| Icon | Name | Use |
| --- | --- | --- |
| 📍 | **Tracking** | Parent issue with task lists for Epics/Features |
| ✨ | **Enhancement** | New functionality or improvements |
| 🛠️ | **Chore** | Technical tasks, refactoring, maintenance |
| 🐛 | **Bug** | Functional defects or regressions |
| 🔍 | **Discussion** | Architectural proposals or RFCs |

### 3. Labels

> [!NOTE] Use label names without emojis (e.g., "chore" not "🛠️ chore")

**Type**: `chore`, `bug`, `enhancement`, `documentation`, `tracking`, `discussion`

**Priority**: `priority:high` (urgent), `priority:medium` (important), `priority:low` (nice to have)

**Size**: `size:small` (< 1 day), `size:medium` (1-3 days), `size:large` (> 3 days)

**Area**: `area:core`, `area:docs`, `area:protocol`, `area:build`, `area:tests`, `area:rs`, `area:ffi`

### 4. Issue Lifecycle

1. **Open** - Created, not started
2. **In Progress** - Actively worked on (assign to yourself)
3. **Review** - PR created, awaiting review
4. **Closed** - Work completed and merged

### 5. Linking Issues and PRs

```markdown
Closes #123
Fixes #456
Related to #789
```

## Roo Assistant Guidelines

### ✅ DO

1. **Use MCP GitHub tools** for all GitHub operations
2. **Check issues first** before creating tasks (`mcp--github--search_issues`)
3. **Create issue** for any new work (`mcp--github--create_issue`)
4. **Reference issues** in commits and PRs
5. **Update status** as work progresses (`mcp--github--update_issue`)
6. **Work on feature branches** (never directly on `develop` or `master`)
7. **Document completion** in issue comments (`mcp--github--add_issue_comment`)
8. **Close issues** when work is complete

### ❌ DON'T

1. Create tasks in `plans/*.md` or `docs/*.md`
2. Add task sections to `CHANGES.md`
3. Create local TODO lists
4. Start work without a GitHub Issue
5. Duplicate work that has an issue
6. Use `gh` CLI (MCP tools only)
7. Commit directly to `master` or `develop`

## Issue Template

```markdown
## Overview
Brief description of what needs to be done

## Background
Why this work is needed

## Goals
1. Specific goal 1
2. Specific goal 2

## Implementation Plan
See [`plans/feature-name.md`](plans/feature-name.md)

## Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Tests pass
- [ ] Documentation updated

## Related
- Part of v0.X.0 release
- Depends on #XXX
- Related to #YYY
```
