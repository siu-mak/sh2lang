# Unreleased

## Highlights
- **Import aliases**: `import "path" as alias` with qualified calls: `alias.func()`. Mangled wrappers emitted lazily (only when referenced).

### Features

### Diagnostics
- **Suggestions**: "did you mean …?" hints for unknown import aliases and unknown qualified function names

### Editors
- **VS Code**: `as` keyword highlighting in import statements

### Fixes
- Test harness: exec fixtures now fail fast when missing expected output files.

### Breaking changes
- **Reserved keyword**: `as` is now a reserved keyword. Previously, `as` was a valid identifier in all positions (variables, function names, parameters). Any existing code using `as` as an identifier will fail to compile.
