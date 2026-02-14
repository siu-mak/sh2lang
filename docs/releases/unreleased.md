# Unreleased

## Features
- **Range syntax for `for` loops**: Added inclusive range syntax `start..end` for iteration (e.g., `for i in 1..10 { ... }`). Supports both parenthesized `(1..10)` and spaced `1 .. 10` forms. Runtime dependency: requires `seq` command (part of coreutils).
- **Path Lookup**: Added `which(name)` builtin to resolve executables in `$PATH` without external dependencies. Returns the path or `""` if not found. **Exit status**: returns 0 when found, 1 when not found (allows `if which("cmd") { ... }`). **Non-aborting**: `which()` returning 1 does not abort the script. Paths may be relative if `$PATH` contains relative entries.
- **Glob Expansion**: Added `glob(pattern)` builtin (Bash-only) to safely expand filesystem glob patterns. Returns a sorted list of matched paths; empty list if no matches. Uses `compgen -G` internally (no eval). Requires Bash 4.3+.
- **Recursive File Finding**: Added `find_files(dir=".", name="*")` builtin (Bash-only) to recursively find files using `find ... -print0`. Handles "weird" filenames (spaces, newlines) safely. Returns a sorted list of paths. Requires Bash 4.3+, GNU find, and GNU sort.
- **Structured Pipeline Iteration**: Added `| each_line <var> { ... }` pipeline consumer. Executes loop in the main process (preserving variable updates) and correctly propagates upstream exit status. Replaces fragile `| while read` patterns. Bash-only (uses process substitution).
- **Job Control**: Added `spawn(cmd)`, `wait(pid)`, and `wait_all(pids)` builtins for running concurrent tasks. `spawn` starts a background job (like `&`) and returns a PID. `wait` waits for a PID and returns its exit code. `wait_all` waits for all PIDs in a list and returns the first non-zero exit code (in list order). Supports `allow_fail=true` for non-aborting waits. `spawn` and `wait` portable to both Bash and POSIX; `wait_all` fully supported on Bash, POSIX supports inline list literals only.


## Diagnostics
- **Unknown function detection**: Calling an undefined function in expression context now produces a compile error with hints to use `run(...)` for external commands or define the function.

## Fixes
- **Capture Status**: `capture(..., allow_fail=true)` now correctly preserves the command's exit status in `status()` across all targets, ensuring it isn't clobbered by internal cleanup operations. [#11]
- **`sh(...)` expression contexts**: `sh(...)` inside `capture(...)` and other expression contexts now correctly executes via `sh -c`. Previously, the command string was passed as a filename argument to `sh` rather than as code to execute.

## Docs
- Clarified `sh(...)` isolation: does not inherit `$@` / `$1`; documented using `argc()` / `arg(n)` or `run(...)` instead.
