# Markdown Formatting Rules

## Document Structure

- Every document must start with a top-level `#` heading
- Headings must be followed by an empty line

## Tables

Use spaces in header divider:

```markdown
| column 1 | column 2 |
| --- | --- |
| text | text |
```

## Code Blocks

Always specify language:

```markdown
\`\`\`rust
fn main() {}
\`\`\`
```

## Lists

- Use consistent markers (all `-` or all `*`)
- Use 2-space indentation for nesting

## Links

Reference code with file paths and optional line numbers:

```markdown
See [`function_name()`](src/module.rs:42) for implementation.
```
