# GitHub Operations

## Use gh CLI

Always use `gh` CLI for GitHub operations.

### Creating Issues

```bash
gh issue create \
  --title "Issue title" \
  --body-file /path/to/issue.md \
  --label "label1,label2" \
  --milestone "milestone-name"
```

### Project Labels

**Type**: `chore`, `bug`, `enhancement`, `documentation`, `tracking`, `discussion`
**Priority**: `priority:high`, `priority:medium`, `priority:low`
**Area**: `area:core`, `area:discovery`, `area:docs`, `area:ffi`, `area:protocol`, `area:testing`
**Size**: `size:small`, `size:medium`, `size:large`

**Note**: Use label names without emojis (e.g., "chore" not "🛠️ chore")

### Listing

```bash
gh issue list
gh issue list --label "label-name"
gh label list
```
