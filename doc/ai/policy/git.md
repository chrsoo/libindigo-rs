# Git Commit Policy

## Commit Message Format

All commit messages must follow this format:

```
<type>: <short summary (max 72 characters)>

<optional body with implementation details>
```

## Rules

### Summary Line (Required)

- **Maximum 72 characters**
- Start with a type prefix (see below)
- Use imperative mood ("Add feature" not "Added feature")
- No period at the end
- Be specific and concise

### Body (Optional)

- Separate from summary with blank line
- Wrap at 72 characters
- Explain **what** and **why**, not **how**
- Reference issues/PRs when applicable
- Can include bullet points

## Commit Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Test additions or modifications
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Maintenance tasks (dependencies, build, etc.)
- `ci`: CI/CD changes
- `style`: Code style changes (formatting, etc.)

## Examples

### Good Commits

```
feat: Add timeout handling to integration tests

- Add 5-second timeout wrappers around connection attempts
- Tests now skip gracefully when server unavailable
- Update health monitor with max_attempts limit
```

```
ci: Update GitHub Actions workflow for multi-strategy builds

Add separate jobs for pure Rust and FFI strategies.
See CI_CD_STRATEGY.md for detailed documentation.
```

```
fix: Prevent infinite retry loops in test harness
```

### Bad Commits

```
❌ Update CI/CD pipeline for pure Rust and FFI strategies with comprehensive configuration...
(Too long - exceeds 72 characters)

❌ Fixed stuff
(Too vague - what was fixed?)

❌ Added timeout handling to prevent infinite retry loops in tests and updated the health monitor to have a max_attempts limit and modified all the integration tests to use timeout wrappers
(Way too long and detailed for summary line)
```

## Implementation Details

For commits with extensive implementation details:

1. Keep summary line short and clear
2. Add brief body if needed
3. Reference separate documentation files for comprehensive details
4. Use `See <filename>` to point to detailed docs

## Rationale

- **Short summaries** make git log readable
- **Clear types** help understand change categories
- **Separate docs** keep commit history clean while preserving details
- **Consistent format** improves automation and tooling
