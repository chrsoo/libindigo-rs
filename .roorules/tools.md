# tool usage

## command line tools

### avoid multi-line arguments

- Avoid multi-line arguments as it breaks Roo auto approval.
- Avoid heredoc on the command line except when part of a script file.
- Long text arguments should be passed as temporary files in `.tmp/`.
