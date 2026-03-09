# Tool Usage

## MCP Tools First

**Always prefer MCP tools over CLI commands** when available:

- **Git operations**: Use MCP Git tools (see [`.roorules/git.md`](.roorules/git.md))
- **GitHub operations**: Use MCP GitHub tools (see [`.roorules/issues.md`](.roorules/issues.md))

**Why**: MCP tools use structured parameters, avoiding multi-line CLI arguments that break Roo auto-approval.

## CLI Multi-Line Arguments

When CLI commands are necessary:

- **Never use multi-line arguments** — breaks Roo auto-approval
- **Never use heredoc** on command line (e.g., `git commit -m "$(cat <<EOF...)"`)
- **Use temp files** for multi-line content: write to `.tmp/`, pass filename to CLI

**Exception**: Heredoc is allowed in committed script files (`.sh`, etc.)

## Examples

### ❌ Bad: Multi-line CLI argument

```bash
git commit -m "feat: Add feature

- Detail 1
- Detail 2"
```

### ✅ Good: MCP tool with structured parameter

```
mcp--git--git_commit with message parameter
```

### ✅ Good: Temp file for CLI

```bash
cat > .tmp/commit-msg.txt << 'EOF'
feat: Add feature

- Detail 1
- Detail 2
EOF
git commit -F .tmp/commit-msg.txt
```
