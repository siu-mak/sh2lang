# Unreleased

## Added

### Expression Interpolation
- **Expression Interpolation (Limited)**: `$"..."` string literals now support evaluating expressions inside `{...}` holes.
  - Supported: `$"Sum: {1 + 2}"`, `$"Cwd: {pwd()}"`, `$"User: {name}"`.
  - **Limitation**: String literals inside holes are not yet supported (e.g., `$"X: { "value" }"` will not compile) due to lexer tokenization constraints. Use variables as a workaround: `let v = "value"; print($"X: {v}")`.
  - Escape literal braces with `\{` and `\}` (e.g., `$"Set: \{a, b\}"` outputs `Set: {a, b}`).
  - A future release will address this limitation with lexer redesign to support full expression interpolation.

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

### Security & Correctness
- **P0 Fix (Breaking Change / Correctness Fix)**: String literals (`"..."`) are now **strict literals**. They do **not** support implicit variable interpolation or Bash parameter expansion.
  - `"$foo"` and `${bar}` in string literals are preserved as literal text (e.g. `print("$foo")` prints `$foo`).
  - To use variables, use **concatenation** (`"Hello " & name`) or **explicit interpolation** (`$"Hello {name}"`).
  - This change ensures that strings like `"$5"` or `"*"` are strictly safe and never trigger unintended Bash behavior.

- **contains() Type Safety Fix**: `contains(haystack, needle)` now uses robust **static type dispatch** instead of brittle runtime probing.
    - **List Membership**: Triggered for list literals, list expressions (e.g. `split`), and tracked list variables (Bash-only).
    - **Substring Search**: Default behavior for strings and untracked variables (Portable).
    - **Improvements**: Removed `declare -p` probing that caused false negatives. Added support for `contains(split(...), ...)` via temporary variable materialization.

- **contains_line() Implementation (P0)**: `contains_line(file, needle)` now correctly reads **file contents** with exact-line matching semantics.
    - **Behavior**: Uses `grep -Fqx -e <needle> <file>` for exact-line matching within the file.
    - **POSIX Portability**: Uses `-e` flag (POSIX-compliant) instead of `--` for robust handling of needles starting with `-`.
    - **Use case**: Ideal for registry trust checks, configuration validation, or any line-oriented file operations.

- **Boolean Encoding Standardization**: Boolean variables are now consistently stored as `"true"` and `"false"` strings (previously "1"/"0").
    - **Effect**: Implicit print/string conversion for booleans is now supported (e.g. `print(true)` outputs `true`).
    - **Back-compat**: Generated conditions still use standard shell logic (`[ "$v" = "true" ]`). Users relying on internal "1"/"0" representation (undocumented) will be affected.
