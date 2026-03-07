# GitHub Issues Management

## Philosophy

**GitHub Issues are the single source of truth for all actionable work items.**

Use for:
- All actionable tasks
- Bug reports
- Feature requests
- Work tracking

**NOT for**: Project documentation (use `docs/` instead)

## Workflow

### 1. Creating Work Items

**Always create a GitHub Issue first** using the **GitHub MCP tool**.

**Never** create tasks in:
- ❌ `docs/*.md` files
- ❌ `plans/*.md` files
- ❌ `CHANGES.md` task sections
- ❌ Local TODO files
- ❌ Code comments with TODO/FIXME

### 2. Referencing Plans

Issues reference plans for implementation details:

```markdown
## Overview
Brief description

## Implementation Plan
See [`plans/feature-name.md`](plans/feature-name.md)

## Acceptance Criteria
- [ ] Specific, testable criteria
```

### 3. Issue Types

| Icon | Name | Use |
| --- | --- | --- |
| 📍 | **Tracking** | Parent issue with task lists for Epics/Features |
| ✨ | **Enhancement** | New functionality or improvements |
| 🛠️ | **Chore** | Technical tasks, refactoring, maintenance |
| 🐛 | **Bug** | Functional defects or regressions |
| 🔍 | **Discussion** | Architectural proposals or RFCs |

### 4. Labels

> [!NOTE] Use label names without emojis (e.g., "chore" not "🛠️ chore")

**Type**: `chore`, `bug`, `enhancement`, `documentation`, `tracking`, `discussion`

**Priority**: `priority:high` (urgent), `priority:medium` (important), `priority:low` (nice to have)

**Size**: `size:small` (< 1 day), `size:medium` (1-3 days), `size:large` (> 3 days)

**Area**: `area:core`, `area:docs`, `area:protocol`, `area:build`, `area:tests`, `area:rs`, `area:ffi`

### 5. Issue Lifecycle

1. **Open** - Created, not started
2. **In Progress** - Actively worked on (assign to yourself)
3. **Review** - PR created, awaiting review
4. **Closed** - Work completed and merged

### 6. Linking Issues and PRs

```markdown
Closes #123
Fixes #456
Related to #789
```

## Roo Assistant Guidelines

### ✅ DO

1. **Use GitHub MCP tools** for all GitHub operations
2. **Check issues first** before creating tasks
3. **Create issue** for any new work
4. **Reference issues** in commits and PRs
5. **Update status** as work progresses
6. **Work on feature branches** (never directly on `develop` or `master`)
7. **Document completion** in issue comments
8. **Close issues** when work is complete

### ❌ DON'T

1. Create tasks in `plans/*.md` or `docs/*.md`
2. Add task sections to `CHANGES.md`
3. Create local TODO lists
4. Start work without a GitHub Issue
5. Duplicate work that has an issue
6. Use `gh` CLI when MCP tools work
7. Commit directly to `master` or `develop`

## GitHub MCP Tools

**Always use MCP tools first.** Use `gh` CLI only for:
- Advanced queries with complex filters
- Bulk operations not supported by MCP

### Common Operations

**Check existing issues**: `mcp--github--list_issues` or `mcp--github--search_issues`

**Create issue**: `mcp--github--create_issue`

**View details**: `mcp--github--get_issue`

**Update issue**: `mcp--github--update_issue`

**Add comment**: `mcp--github--add_issue_comment`

## Example Issue Template

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
