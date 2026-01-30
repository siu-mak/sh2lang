# Unreleased

## Breaking Changes

- **String Interpolation Safety**: Unbound variables in string interpolations (e.g. `$FOO`, `${BAR}`) now remain as string literals instead of expanding to empty strings or environment variables. This prevents accidental injection of Bash variables. To use an environment variable, explicitly use `env.FOO` (though inside strings `env.FOO` is not currently interpolated; use concatenation: `"Value: " & env.FOO`).

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
- Added support for nested `allow_fail` option directly on `run` calls within `capture` (e.g. `capture(run(..., allow_fail=true))`), which is hoisted to the capture behavior.
- Fixed bash codegen so `capture(run(...), allow_fail=true)` preserves `status()` (non-zero exit codes no longer clobbered).
- Clarified that `capture(..., allow_fail=true)` is only valid in `let` assignments.
- Implemented **Named Argument Policy (Hardening)**: `name=value` arguments are now strictly limited to builtins (`run`, `sudo`, `sh`, `capture`, `confirm`). General function calls are restricted to positional arguments, with clear diagnostics for violations.
- Implemented **Strict Command Word Model for `$(...)`**: Command substitution now strictly interprets its content as command words. Generic function call shorthand `$(func())` is preserved via parser-level flattening to `func`, but named options are rejected within `$(...)`.
- **Sudo Hardening**: Refactored `sudo(...)` lowering to unconditional inject the `--` separator before command arguments in all contexts (including command substitution), preventing flag injection attacks. Flattened `sudo` arguments in the parser to ensure consistent behavior without double-quoting.

- Fixed Bash codegen for `arg(expr)` with dynamic indexes to properly quote arguments passed to `__sh2_arg_by_index` using a dedicated helper to ensure safe and deterministic forms.
- Hardened `arg(expr)` validation: non-integer indices (e.g., strings, nested calls) now produce a compile-time error.
