# Unreleased

## Highlights
- **Import aliases**: `import "path" as alias` syntax accepted. Alias is parsed and stored; qualified calls (`alias.func()`) are not yet supported.

### Features

### Diagnostics

### Fixes

### Breaking changes
- **Reserved keyword**: `as` is now a reserved keyword. Previously, `as` was a valid identifier in all positions (variables, function names, parameters). Any existing code using `as` as an identifier will fail to compile.
