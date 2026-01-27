# Unreleased

## Added

### Pipe Blocks
Support for arbitrary statement blocks in pipelines:
- `pipe { ... } | { ... }`
- `run(...) | { ... }`
- `pipe { ... } | run(...)`
Mixed run/block stages are fully supported, with each stage running in an isolated subshell context.

### Pipeline Sudo
Pipelines now accept `sudo(...)` stages:
- `run("cmd") | sudo("cmd", n=true)`
- `pipe { ... } | sudo(...)`
`sudo` stages participate in the pipeline with correct pipefail and error handling, using the same options as standalone `sudo(...)`.

### Predicates
- Added `starts_with(text, prefix)` builtin predicate.
### Argument Access
- Added `argv()` as an alias for `args()` (returns all arguments as a list).
- Fixed `arg(n)` to avoid generating runtime calls to `argv` command in shell output.

### Capture
- Fixed `capture(..., allow_fail=true)` to correctly return captured stdout and update `status()` without aborting the script on failure.
- Clarified that `capture(..., allow_fail=true)` is only valid in `let` assignments.

### Redirects
- Fixed a bug where `inherit_stdout` or `inherit_stderr` inside a `redirect` block caused literal `\n` characters to be emitted in the generated Bash script.
- Fixed Bash codegen for `arg(expr)` with dynamic indexes to properly quote arguments passed to `__sh2_arg_by_index` using a dedicated helper to ensure safe and deterministic forms.
- Hardened `arg(expr)` validation: non-integer indices (e.g., strings, nested calls) now produce a compile-time error.
