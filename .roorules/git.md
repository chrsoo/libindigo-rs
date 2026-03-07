# Git Commit Rules

## Gitflow Rules

- Always branch 'feature/*' from 'develop'.
- Always branch 'release/*' from 'develop'.
- Always branch 'hotfix/*' from 'main'.
- Never merge 'feature/*' directly into 'main'.
- After finishing a release or hotfix, merge into BOTH 'main' and 'develop'.
- Use semantic versioning (vX.X.X) for tags on 'main'.

## Format

```text
<type>: <summary (max 72 characters)>

<optional body>
```

## Temporary Files

The body of commit message is created as temporary file in /tmp/[issue].md and deleted after commit is complete.

## Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `test`: Tests
- `refactor`: Code refactoring
- `perf`: Performance
- `chore`: Maintenance (dependencies, build, etc.)
- `ci`: CI/CD changes

## Rules

> [!NOTE] Use temporary files for the issue body is using `git` on the command line!

- **Summary line**: Max 72 characters, imperative mood, no period
- **Body**: Wrap at 72 characters, explain what/why not how
- **References**: Include issue numbers when applicable

## Examples

Good:

```text
feat: Add timeout handling to integration tests

- Add 5-second timeout wrappers around connection attempts
- Tests now skip gracefully when server unavailable
```

Bad:

```text
❌ Update CI/CD pipeline for pure Rust and FFI strategies with comprehensive configuration...
(Too long - exceeds 72 characters)

❌ Fixed stuff
(Too vague)
```
