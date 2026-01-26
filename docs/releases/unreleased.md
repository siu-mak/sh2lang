# Unreleased

## Added

### Pipe Blocks
Support for arbitrary statement blocks in pipelines:
- `pipe { ... } | { ... }`
- `run(...) | { ... }`
- `pipe { ... } | run(...)`
Mixed run/block stages are fully supported, with each stage running in an isolated subshell context.
