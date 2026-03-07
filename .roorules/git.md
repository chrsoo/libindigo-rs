# Git Workflow Rules

## Gitflow Branches

- `master` - Production releases only, tagged with semantic versions
- `develop` - Integration branch for features
- `feature/*` - New features, branch from `develop`
- `release/*` - Release preparation, branch from `develop`
- `hotfix/*` - Emergency fixes, branch from `master`

## Branch Rules

- **Never commit directly to `master` or `develop`**
- Always work on `feature/*` branches
- Branch from `develop` for new work: `feature/issue-123-description`
- Merge to `develop` via PR after tests pass
- After release/hotfix, merge to BOTH `master` and `develop`

## Workflow: Executing Plans

### 1. Start Work

```bash
git checkout develop
git pull origin develop
git checkout -b feature/issue-123-short-name
```

### 2. Implement & Commit

- Make changes incrementally
- Commit with clear messages (see format below)
- Push branch: `git push -u origin feature/issue-123-short-name`

### 3. Pre-Merge Checklist

Before creating PR:

- [ ] All changes committed
- [ ] Branch pushed to origin
- [ ] `cargo test --workspace` passes
- [ ] Integration tests pass (if applicable)
- [ ] Documentation updated

### 4. Create PR

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

### Types

- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation only
- `test` - Tests only
- `refactor` - Code refactoring
- `perf` - Performance improvement
- `chore` - Maintenance (dependencies, build, etc.)
- `ci` - CI/CD changes

### Rules

- **Summary**: Max 72 chars, imperative mood, no period
- **Body**: Wrap at 72 chars, explain what/why not how
- **References**: Include issue numbers (`#123`)

### Examples

Good:

```text
feat: Add timeout handling to integration tests

- Add 5-second timeout wrappers around connection attempts
- Tests now skip gracefully when server unavailable

Closes #145
```

Bad:

```text
❌ Update CI/CD pipeline for pure Rust and FFI strategies...
(Too long - exceeds 72 characters)

❌ Fixed stuff
(Too vague)
```

## Release Workflow

### Creating a Release

```bash
git checkout develop
git checkout -b release/v0.3.0
# Update version numbers, CHANGES.md
git commit -m "chore: Prepare v0.3.0 release"
git push origin release/v0.3.0
```

### Finishing a Release

```bash
# Merge to master
git checkout master
git merge --no-ff release/v0.3.0
git tag -a v0.3.0 -m "Release v0.3.0"
git push origin master --tags

# Merge back to develop
git checkout develop
git merge --no-ff release/v0.3.0
git push origin develop

# Delete release branch
git branch -d release/v0.3.0
git push origin --delete release/v0.3.0
```

## Hotfix Workflow

```bash
# Create hotfix from master
git checkout master
git checkout -b hotfix/v0.2.1

# Fix and commit
git commit -m "fix: Critical bug in discovery module"

# Merge to master
git checkout master
git merge --no-ff hotfix/v0.2.1
git tag -a v0.2.1 -m "Hotfix v0.2.1"
git push origin master --tags

# Merge to develop
git checkout develop
git merge --no-ff hotfix/v0.2.1
git push origin develop

# Delete hotfix branch
git branch -d hotfix/v0.2.1
```
