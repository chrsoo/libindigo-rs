# Markdown Formatting Rules

## Document Structure

### Top-Level Heading Required

Every markdown document must start with a top-level heading (`#`) containing a descriptive title:

```markdown
# Document Title

Content starts here...
```

**Bad:**

```markdown
Some content without a heading...
```

### Headings Must Be Followed by Empty Line

Each heading must be followed by an empty line before content begins:

**Good:**

```markdown
## Section Title

This is the content of the section.
```

**Bad:**

```markdown
## Section Title
This is the content of the section.
```

## Table Formatting

### Compact Table Format

Tables must have spaces between bars in the header divider:

**Good:**

```markdown
| column 1 | column 2 |
| --- | --- |
| text | text |
```

**Bad:**

```markdown
| column 1 | column 2 |
|---|---|
| text | text |
```

### Table Alignment

Use colons in the divider row to specify alignment:

```markdown
| Left | Center | Right |
| :--- | :---: | ---: |
| text | text | text |
```

## Code Blocks

### Fenced Code Blocks

Always specify the language for syntax highlighting:

**Good:**

```markdown
\`\`\`rust
fn main() {
    println!("Hello, world!");
}
\`\`\`
```

**Bad:**

```markdown
\`\`\`
fn main() {
    println!("Hello, world!");
}
\`\`\`
```

### Inline Code

Use single backticks for inline code references:

```markdown
The [`function_name()`](path/to/file.rs:42) is defined here.
```

## Lists

### Consistent List Markers

Use consistent markers within a list:

**Good:**

```markdown
- Item 1
- Item 2
- Item 3
```

**Bad:**

```markdown
- Item 1
* Item 2
- Item 3
```

### Nested Lists

Use 2-space indentation for nested lists:

```markdown
- Parent item
  - Child item
  - Another child
- Another parent
```

## Links

### Reference-Style Links

For repeated links, use reference-style:

```markdown
See the [documentation][docs] for more details.
The [documentation][docs] explains everything.

[docs]: https://example.com/docs
```

### File Links with Line Numbers

When referencing code, include file path and optional line number:

```markdown
See [`function_name()`](src/module.rs:42) for implementation.
See [configuration file](config.toml) for settings.
```

## Common Errors and Fixes

### Missing Empty Lines

**Error:** No empty line after heading

```markdown
## Heading
Content immediately follows
```

**Fix:**

```markdown
## Heading

Content follows after empty line
```

### Incorrect Table Dividers

**Error:** No spaces in table divider

```markdown
| A | B |
|---|---|
| 1 | 2 |
```

**Fix:**

```markdown
| A | B |
| --- | --- |
| 1 | 2 |
```

### Missing Document Title

**Error:** Document starts without top-level heading

```markdown
This document explains...
```

**Fix:**

```markdown
# Document Title

This document explains...
```

### Inconsistent List Formatting

**Error:** Mixed list markers

```markdown
- Item 1
* Item 2
```

**Fix:**

```markdown
- Item 1
- Item 2
```

## Best Practices

1. **Use descriptive headings** - Headings should clearly indicate section content
2. **Keep lines under 80-100 characters** - Improves readability in editors
3. **Use blank lines to separate sections** - Improves visual structure
4. **Be consistent with formatting** - Choose a style and stick to it
5. **Use semantic markup** - Use appropriate elements (headings, lists, code blocks)

## Validation

Before committing markdown files:

1. Check that document starts with `#` heading
2. Verify all headings have empty line after them
3. Ensure table dividers have spaces: `| --- |`
4. Confirm code blocks specify language
5. Verify links are not broken
