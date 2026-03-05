# Code Mode Rules

## Purpose

Code mode is designed for writing, modifying, and refactoring code. It focuses on implementation following established plans and designs.

## When to Use Code Mode

Use Code mode when:
- Implementing **✨ Enhancement** issues (after design is complete)
- Implementing **🛠️ Chore** issues (simple to moderate complexity)
- Implementing **🐛 Bug** fixes (after root cause is identified)
- Writing tests
- Updating documentation
- Making code changes based on clear requirements

## Primary Responsibilities

1. **Implement Features**: Write code according to design plans
2. **Write Tests**: Create unit and integration tests
3. **Refactor Code**: Improve code structure and quality
4. **Fix Bugs**: Implement fixes for identified issues
5. **Update Documentation**: Maintain inline code documentation
6. **Commit Changes**: Create clear, atomic commits

## Workflow Pattern

```
1. Review implementation plan or acceptance criteria
2. Implement one criterion at a time
3. Write tests for new functionality
4. Run tests frequently
5. Commit working code regularly
6. Update inline documentation
7. Switch to Debug if issues arise
8. Close issue when complete
```

## File Restrictions

Code mode can edit:
- All source code files (*.rs)
- Configuration files (*.toml, *.yml, *.yaml, *.json)
- Test files
- Build scripts (build.rs)
- Examples
- README.md and inline documentation

Code mode should NOT edit:
- Planning documents in plans/ (use Architect)
- Comprehensive documentation in doc/ (use Ask for major updates)

## Implementation Best Practices

### Code Quality

✅ **DO:**
- Follow Rust idioms and conventions
- Write clear, self-documenting code
- Add comments for complex logic
- Use meaningful variable and function names
- Handle errors properly
- Write tests for new functionality
- Keep functions focused and small
- Use type system for safety

❌ **DON'T:**
- Skip error handling
- Write untested code
- Ignore compiler warnings
- Use unsafe code without justification
- Create overly complex solutions
- Forget to update documentation
- Make unrelated changes in same commit

### Testing Strategy

```rust
// Unit tests for individual functions
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specific_behavior() {
        // Arrange
        let input = setup_test_data();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected_value);
    }
}

// Integration tests in tests/ directory
#[tokio::test]
async fn test_feature_integration() {
    // Test complete feature workflow
}
```

### Commit Messages

Follow conventional commit format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `refactor`: Code refactoring
- `test`: Adding tests
- `docs`: Documentation updates
- `chore`: Maintenance tasks

Example:
```
feat(discovery): implement Zeroconf discovery strategy

- Add ZeroconfDiscovery struct implementing DiscoveryStrategy
- Integrate mdns-sd crate for service discovery
- Add unit tests for discovery logic

Closes #124
```

## Integration with GitHub Issues

Code mode works with:
- **✨ Enhancement**: Implement after Architect designs
- **🛠️ Chore**: Implement directly or after Architect plans
- **🐛 Bug**: Implement fix after Debug identifies root cause

## Handoff Protocol

### From Architect to Code

```
Architect: "Design complete for enhancement #124. See
plans/discovery-implementation.md. Please implement the
ZeroconfDiscovery struct in src/discovery/zeroconf_impl.rs."

Code:
1. Reviews implementation plan
2. Implements according to design
3. Writes tests
4. Commits changes
5. Closes issue
```

### From Code to Debug

```
Code: "Implementation of #124 is failing integration tests with
a connection timeout. Please use Debug mode to investigate why
the Zeroconf service discovery is timing out."

Debug: Investigates and identifies root cause
```

### From Debug to Code

```
Debug: "Root cause identified: Zeroconf service wasn't being
properly initialized in test harness. Fix needed in
tests/harness/server.rs to call start_service() before running
discovery tests."

Code:
1. Implements the fix
2. Verifies tests pass
3. Commits fix
```

### From Code to Ask

```
Code: "Feature #124 is complete. Please use Ask mode to document
the Zeroconf discovery feature in the user guide and create an
example."

Ask: Creates documentation and examples
```

## Implementation Checklist

Before closing an issue:

- [ ] All acceptance criteria are met
- [ ] Tests are written and passing
- [ ] Code follows project conventions
- [ ] Error handling is proper
- [ ] Inline documentation is updated
- [ ] No compiler warnings
- [ ] Changes are committed with clear messages
- [ ] Related issues are referenced

## When to Switch Modes

### Switch to Debug when:
- Tests fail with unclear errors
- Unexpected runtime behavior occurs
- Performance issues need investigation
- Implementation reveals hidden bugs

### Switch to Ask when:
- Need to understand unfamiliar code
- Researching error messages or patterns
- Creating user-facing documentation
- Explaining implementation to others

### Switch to Architect when:
- Implementation reveals design issues
- Need to evaluate alternative approaches
- Architectural changes are required
- Design assumptions are invalid

## Example Usage

### Example 1: Implementing Enhancement

```
User: "Please implement Zeroconf discovery according to the plan."

Code:
1. Reviews plans/discovery-implementation.md
2. Adds mdns-sd dependency to Cargo.toml
3. Creates src/discovery/zeroconf_impl.rs:
   - Implements ZeroconfDiscovery struct
   - Implements DiscoveryStrategy trait
   - Adds error handling
4. Writes unit tests in same file
5. Updates src/discovery/mod.rs to export new module
6. Runs tests: cargo test
7. Commits: "feat(discovery): implement Zeroconf discovery"
8. Closes issue #124
```

### Example 2: Fixing Bug

```
User: "Please implement the fix for the race condition."

Code:
1. Reviews Debug findings in issue #145
2. Opens tests/harness/server.rs
3. Adds health_check() method:
   async fn health_check(&self) -> Result<()> {
       // Wait for service to be ready
   }
4. Updates test setup to call health_check()
5. Adds retry logic with exponential backoff
6. Runs tests 100 times to verify fix
7. Commits: "fix(tests): add health check to prevent race condition"
8. Closes issue #145
```

### Example 3: Simple Chore

```
User: "Update dependencies to latest versions."

Code:
1. Runs: cargo update
2. Reviews Cargo.lock changes
3. Runs full test suite: cargo test
4. Checks for deprecation warnings
5. Updates any code if needed
6. Commits: "chore(deps): update dependencies to latest versions"
7. Closes chore issue
```

## Error Handling Patterns

### Result Type

```rust
use crate::error::Error;

pub fn operation() -> Result<Value, Error> {
    let data = fetch_data()
        .map_err(|e| Error::FetchFailed(e.to_string()))?;

    process_data(data)
}
```

### Custom Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Connection timeout after {0}s")]
    Timeout(u64),

    #[error("Discovery failed: {0}")]
    DiscoveryFailed(#[from] std::io::Error),
}
```

## Performance Considerations

- Use appropriate data structures (Vec, HashMap, BTreeMap)
- Avoid unnecessary allocations
- Use references when possible
- Consider async/await for I/O operations
- Profile before optimizing
- Document performance-critical sections

## Safety Considerations

- Minimize unsafe code
- Document safety invariants
- Use type system for correctness
- Validate inputs at boundaries
- Handle all error cases
- Avoid panics in library code

## Related Documentation

- [Roo Workflow Scheme](../doc/roo-workflow-scheme.md) - Complete workflow guide
- [Ways of Working](../doc/ways-of-working.md) - GitHub issue types
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) - Rust best practices
