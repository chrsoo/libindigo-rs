# GitHub Operations Rules

## Use gh CLI for GitHub Operations

Always use the `gh` CLI tool for GitHub operations instead of web UI or manual API calls.

### Issue Management

**Creating Issues:**
```bash
gh issue create \
  --title "Issue title" \
  --body-file /path/to/issue.md \
  --label "label1,label2,label3" \
  --milestone "milestone-name"
```

**Editing Issues:**
```bash
gh issue edit <issue-number> --milestone "milestone-name"
gh issue edit <issue-number> --add-label "new-label"
gh issue edit <issue-number> --remove-label "old-label"
```

**Listing Issues:**
```bash
gh issue list
gh issue list --label "label-name"
gh issue list --milestone "milestone-name"
gh issue view <issue-number>
```

### Label Management

**List available labels:**
```bash
gh label list
```

**Important:** Use label names without emojis (e.g., "chore" not "🛠️ chore")

### Milestone Management

**List milestones:**
```bash
gh api repos/:owner/:repo/milestones --jq '.[] | "\(.number) \(.title)"'
```

**Note:** The `gh milestone` command is not available in all gh CLI versions. Use the API method above.

### Project Labels

Available labels in this project:
- **Type:** `chore`, `bug`, `enhancement`, `documentation`, `tracking`, `discussion`
- **Priority:** `priority:high`, `priority:medium`, `priority:low`
- **Area:** `area:core`, `area:discovery`, `area:docs`, `area:ffi`, `area:protocol`, `area:testing`
- **Size:** `size:small`, `size:medium`, `size:large`

### Best Practices

1. Always check available labels with `gh label list` before creating issues
2. Use milestone names exactly as they appear in the repository
3. Prefer `--body-file` over `--body` for multi-line issue descriptions
4. Extract issue numbers from gh CLI output for tracking
5. Use the gh API for operations not directly supported by gh commands
