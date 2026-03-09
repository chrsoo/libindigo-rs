# Git Workflow Rules

## MCP Git Tools

**Use MCP Git tools as primary interface.** Fall back to CLI only for unsupported operations.

| Operation | MCP Tool | CLI Fallback |
| --- | --- | --- |
| Status | `mcp--git--git_status` | - |
| Log | `mcp--git--git_log` | - |
| Diff | `mcp--git--git_diff_*` | - |
| Commit | `mcp--git--git_commit` | - |
| Add | `mcp--git--git_add` | - |
| Reset | `mcp--git--git_reset` | - |
| Checkout | `mcp--git--git_checkout` | - |
| Create branch | `mcp--git--git_create_branch` | - |
| List branches | `mcp--git--git_list_branches` | - |
| Show | `mcp--git--git_show` | - |
| Push | - | `git push` |
| Pull | - | `git pull` |
| Merge | - | `git merge` |
| Tag | - | `git tag` |
| Delete branch | - | `git branch -d` |
| Delete remote | - | `git push origin --delete` |

## Gitflow Branches

- `master` - Production releases, tagged with semantic versions
- `develop` - Integration branch for features
- `feature/*` - New features, branch from `develop`
- `release/*` - Release preparation, branch from `develop`
- `hotfix/*` - Emergency fixes, branch from `master`

## Branch Rules

- **Never commit directly to `master` or `develop`**
- Always work on `feature/*` branches
- Branch from `develop`: `feature/issue-123-description`
- Merge to `develop` via PR after tests pass
- After release/hotfix, merge to BOTH `master` and `develop`

## Workflow: Executing Plans

### 1. Start Work

1. Checkout develop: `mcp--git--git_checkout` (branch: `develop`)
2. Pull latest: `git pull origin develop`
3. Create feature branch: `mcp--git--git_create_branch` (branch: `feature/issue-123-short-name`)

### 2. Implement & Commit

1. Make changes incrementally
2. Stage files: `mcp--git--git_add`
3. Commit: `mcp--git--git_commit` (message parameter handles multi-line)
4. Push: `git push -u origin feature/issue-123-short-name`

### 3. Pre-Merge Checklist

- [ ] All changes committed
- [ ] Branch pushed to origin
- [ ] `cargo test --workspace` passes
- [ ] Integration tests pass (if applicable)
- [ ] Documentation updated

### 4. Create PR

Use MCP GitHub tools (see [`.roorules/issues.md`](.roorules/issues.md)):
- Link to tracking issue: `Closes #123` or `Related to #123`
- Describe what changed and why
- Request review if needed

### 5. Merge

- Merge to `develop` only after tests pass
- Delete feature branch after merge

## Commit Message Format

```text
<type>: <summary (max 72 characters)>

<optional body>
```

**Types**: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`, `ci`

**Rules**:
- Summary: Max 72 chars, imperative mood, no period
- Body: Wrap at 72 chars, explain what/why not how
- References: Include issue numbers (`#123`)

**Examples**:

✅ Good:
```text
feat: Add timeout handling to integration tests

- Add 5-second timeout wrappers around connection attempts
- Tests now skip gracefully when server unavailable

Closes #145
```

❌ Bad:
```text
Update CI/CD pipeline for pure Rust and FFI strategies...
(Too long - exceeds 72 characters)

Fixed stuff
(Too vague)
```

## Release Workflow

### Creating a Release

1. Checkout develop: `mcp--git--git_checkout` (branch: `develop`)
2. Create release branch: `mcp--git--git_create_branch` (branch: `release/v0.3.0`)
3. Update version numbers, CHANGES.md
4. Commit: `mcp--git--git_commit` (message: `chore: Prepare v0.3.0 release`)
5. Push: `git push origin release/v0.3.0`

### Finishing a Release

1. Checkout master: `mcp--git--git_checkout` (branch: `master`)
2. Merge: `git merge --no-ff release/v0.3.0`
3. Tag: `git tag -a v0.3.0 -m "Release v0.3.0"`
4. Push: `git push origin master --tags`
5. Checkout develop: `mcp--git--git_checkout` (branch: `develop`)
6. Merge back: `git merge --no-ff release/v0.3.0`
7. Push: `git push origin develop`
8. Delete local: `git branch -d release/v0.3.0`
9. Delete remote: `git push origin --delete release/v0.3.0`

## Hotfix Workflow

1. Checkout master: `mcp--git--git_checkout` (branch: `master`)
2. Create hotfix: `mcp--git--git_create_branch` (branch: `hotfix/v0.2.1`)
3. Fix and commit: `mcp--git--git_commit`
4. Checkout master: `mcp--git--git_checkout` (branch: `master`)
5. Merge: `git merge --no-ff hotfix/v0.2.1`
6. Tag: `git tag -a v0.2.1 -m "Hotfix v0.2.1"`
7. Push: `git push origin master --tags`
8. Checkout develop: `mcp--git--git_checkout` (branch: `develop`)
9. Merge: `git merge --no-ff hotfix/v0.2.1`
10. Push: `git push origin develop`
11. Delete: `git branch -d hotfix/v0.2.1`
