# GitHub Issues Management

## Philosophy

**GitHub Issues are the single source of truth for all actionable work items.**

GitHub Issues are for:

- Coordination of work
- All actionable tasks
- Bug reports
- Feature requests
- Work tracking

Git Hub issues are NOT for [documenting the project](documentation.md)!

## Workflow

### 1. Creating Work Items

**Always create a GitHub Issue first** before starting work using the **GitHub MCP tool**.

**Never** create tasks in:

- ❌ `doc/*.md` files
- ❌ `plans/*.md` files
- ❌ `CHANGES.md` task sections
- ❌ Local TODO files
- ❌ Code comments with TODO/FIXME

### 2. Referencing Plans

Issues should reference detailed plans, not replace them:

```markdown
## Overview
Brief description of the work

## Implementation Plan
See detailed architecture in [`plans/feature-name.md`](plans/feature-name.md)

## Acceptance Criteria
- [ ] Specific, testable criteria
```

### Classifying work items as issues

| Icon | Name | Summary |
| --- | --- | --- |
| 📍 | **Tracking** | A parent issue using task lists to manage the progress of an Epic or Feature. |
| ✨ | **Enhancement** | A request for new functionality or improvements (User Stories & Requirements). |
| 🛠️ | **Chore** | Technical tasks, refactoring, or maintenance that doesn't add direct user value. |
| 🐛 | **Bug** | A report of a functional defect, regression, or unintended behavior. |
| 🔍 | **Discussion** | A proposal for architectural changes or RFCs to gather feedback before coding. |

### Understanding hierarchical issue relationships

```text
📍 Tracking
└── ✨ Enhancement
    ├── 📄 Task List Item
    └── 📄 Task List Item
└── 🛠️ Chore
    ├── 📄 Task List Item
    └── 📄 Task List Item
└── 🐛 Bug
🔍 Discussion
```

### Using labels to organize issues

> [!NOTE] Use label names without emojis (e.g., "chore" not "🛠️ chore")

**Type**: `chore`, `bug`, `enhancement`, `documentation`, `tracking`, `discussion`
**Priority**: `priority:high`, `priority:medium`, `priority:low`
**Area**: `area:core`, `area:docs`, `area:protocol`, `area:build`, `area:tests`, `area:rs`, `area:ffi`
**Size**: `size:small`, `size:medium`, `size:large`

#### Priority

- `priority:high` - Urgent, blocking work
- `priority:medium` - Important but not urgent
- `priority:low` - Nice to have

#### Size

- `size:small` - < 1 day
- `size:medium` - 1-3 days
- `size:large` - > 3 days

#### Area

- `area:core` - Core library
- `area:docs` - Project Documentation
- `area:protocol` - Protocol implementation
- `area:build` - Build system
- `area:tests` - Testing
- `area:rs` - Pure Rust strategy for integration with the INDIGO bus
- `area:ffi` - FFI strategy for integration with INDIGO bus

### 4. Issue Lifecycle

1. **Open** - Issue created, not yet started
2. **In Progress** - Actively being worked on (assign to yourself)
3. **Review** - PR created, awaiting review
4. **Closed** - Work completed and merged

### 5. Linking Issues and PRs

Always link PRs to issues:

```markdown
Closes #123
Fixes #456
Related to #789
```

### 6. CHANGES.md Integration

`CHANGES.md` tracks **user-facing features**, not tasks:

```markdown
## [0.3.0] - Planned

### Features
- **High-level Device API**: Type-safe traits for cameras, mounts, etc. (see #12)

### Tasks
- Automate constants extraction (see #10)
- Documentation organization (see #11)
```

## Roo Assistant Guidelines

When working on this project:

### ✅ DO

1. **Use GitHub MCP tools** for all GitHub operations (issues, PRs, etc.)
2. **Check GitHub Issues first** before creating any task
3. **Create a GitHub Issue** for any new work item or plan
4. **Reference existing issues** in commit messages and PRs
5. **Update issue status** as work progresses
6. **Document what was done in issue comments** when finishing a task
7. **Work on branches** for logically grouped tasks
8. **Close issues** when work is complete
9. **Link to detailed plans** in `plans/` from issues

### ❌ DON'T

1. Create tasks in `plans/*.md` files
2. Add task sections to `CHANGES.md`
3. Create local TODO lists
4. Start work without a GitHub Issue
5. Duplicate work that already has an issue
6. Document what was done in *.md files
7. Use `gh` CLI when MCP tools provide the same functionality

## GitHub Operations

### Tool Priority

**Always use GitHub MCP tools first.** Only use `gh` CLI when:

- The MCP tool doesn't provide the needed functionality
- You need advanced CLI-specific features (e.g., complex queries, bulk operations)

### Common Operations with MCP Tools

#### Check for existing issues

```text
Use: mcp--github--list_issues or mcp--github--search_issues
```

#### Create new issue

```text
Use: mcp--github--create_issue
```

#### View issue details

```text
Use: mcp--github--get_issue
```

#### Update issue

```text
Use: mcp--github--update_issue
```

#### Add issue comment

```text
Use: mcp--github--add_issue_comment
```

### Fallback CLI Commands

Only use these when MCP tools don't provide the functionality:

#### Advanced search with complex filters

```bash
gh issue list --repo chrsoo/libindigo-rs --search "keyword" --state all --json number,title,labels
```

#### Bulk operations

```bash
gh issue list --repo chrsoo/libindigo-rs --label "needs-triage" --json number | jq -r '.[].number' | xargs -I {} gh issue edit {} --add-label "triaged"
```

## Migration from Local Tasks

If you find tasks in local files:

1. **Create GitHub Issue** for each task
2. **Link to detailed plan** if one exists
3. **Remove task from local file** or convert to reference:

   ```markdown
   See GitHub Issue #123 for implementation tracking
   ```

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
See detailed architecture in [`plans/feature-name.md`](plans/feature-name.md)

### Phase 1: [Name]
- Task 1
- Task 2

### Phase 2: [Name]
- Task 3
- Task 4

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

## Benefits

1. **Single Source of Truth**: All work tracked in one place
2. **Visibility**: Team can see what's being worked on
3. **History**: Complete audit trail of decisions
4. **Integration**: Works with GitHub Projects, milestones, etc.
5. **Notifications**: Automatic updates on issue activity
6. **Search**: Easy to find past discussions and decisions
