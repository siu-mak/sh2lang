# Unreleased

## Features
- **Range syntax for `for` loops**: Added inclusive range syntax `start..end` for iteration (e.g., `for i in 1..10 { ... }`). Supports both parenthesized `(1..10)` and spaced `1 .. 10` forms. Runtime dependency: requires `seq` command (part of coreutils).
- **Path Lookup**: Added `which(name)` builtin to resolve executables in `$PATH` without external dependencies. Returns the path or `""` if not found. **Exit status**: returns 0 when found, 1 when not found (allows `if which("cmd") { ... }`). **Non-aborting**: `which()` returning 1 does not abort the script. Paths may be relative if `$PATH` contains relative entries.
- **Glob Expansion**: Added `glob(pattern)` builtin (Bash-only) to safely expand filesystem glob patterns. Returns a sorted list of matched paths; empty list if no matches. Uses `compgen -G` internally (no eval). Requires Bash 4.3+.
- **Recursive File Finding**: Added `find_files(dir=".", name="*")` builtin (Bash-only) to recursively find files using `find ... -print0`. Handles "weird" filenames (spaces, newlines) safely. Returns a sorted list of paths. Requires Bash 4.3+, GNU find, and GNU sort.
- **Structured Pipeline Iteration**: Added `| each_line <var> { ... }` pipeline consumer. Executes loop in the main process (preserving variable updates) and correctly propagates upstream exit status. Replaces fragile `| while read` patterns. Bash-only (uses process substitution).
- **Job Control**: Added `spawn(cmd)`, `wait(pid)`, and `wait_all(pids)` builtins for running concurrent tasks. `spawn` starts a background job (like `&`) and returns a PID. `wait` waits for a PID and returns its exit code. `wait_all` waits for all PIDs in a list and returns the first non-zero exit code (in list order). Supports `allow_fail=true` for non-aborting waits. `spawn` and `wait` portable to both Bash and POSIX; `wait_all` fully supported on Bash, POSIX supports inline list literals only.
- **Stdin Line Iteration**: Added `stdin_lines()` iterator for `for` loops (e.g. `for line in stdin_lines() { ... }`). Correctly handles whitespace, raw lines, and empty input. Fully supported on both Bash and POSIX targets. Compliant with Policy A variable semantics.
- **Streaming File Discovery**: Added `find0(dir=".", name=?, type=?, maxdepth=?)` iterator for `for` loops (Bash-only). Uses NUL-delimited `find` for safe handling of filenames with spaces, newlines, and special characters. Results are deterministically sorted. Options are compile-time validated (`type` must be `"f"` or `"d"`, `maxdepth` must be a non-negative integer literal).


## Diagnostics
- **Unknown function detection**: Calling an undefined function in expression context now produces a compile error with hints to use `run(...)` for external commands or define the function.

## Fixes
- **Capture Status**: `capture(..., allow_fail=true)` now correctly preserves the command's exit status in `status()` across all targets, ensuring it isn't clobbered by internal cleanup operations. [#11]
- **`sh(...)` expression contexts**: `sh(...)` inside `capture(...)` and other expression contexts now correctly executes via `sh -c`. Previously, the command string was passed as a filename argument to `sh` rather than as code to execute.

## Docs
- **`sh()` Argument Forwarding**: Added `args=args()` (or `args=argv()`) option to `sh(...)` to explicitly forward parent script positional parameters to the child shell process. By default, `sh(...)` starts with empty arguments to prevent accidental leakage.
- Clarified `sh(...)` isolation: does not inherit `$@` / `$1`; documented using `argc()` / `arg(n)` or `run(...)` instead.
- **Structured primitives over `sh("...")`**: Docs now prefer `glob()`, `find0()`, `spawn()`, `stdin_lines()`, and structured pipelines over `sh("...")` where available. Remaining `sh(...)` usages are justified with `# sh(...) because:` comments. CI enforcement test added.


## Infrastructure
- **Unified GitHub Pages deployment**: The APT repository URL has moved from `https://siu-mak.github.io/sh2lang/` to `https://siu-mak.github.io/sh2lang/apt/`. Update your APT source line and GPG key URL accordingly. Ubuntu 24.04 (noble) is now supported alongside 22.04 (jammy).

## Breaking Changes
- **Hardened `arg(i)`**: `arg(i)` now enforces strict runtime validation for variable indices. It aborts the script with a fatal error if the index is non-numeric, `< 1`, or `> argc()`. Previously, invalid or out-of-bounds indices might have returned empty strings or behaved unpredictably depending on the target shell. This change ensures safety against injection and logic errors.
- **Strict Variable Semantics**: Variables must now be declared with `let` before use or assignment. `let` declarations allow shadowing only in disjoint branches (e.g., `if true { let x=1 } else { let x=2 }`), preventing accidental re-declaration errors. Diagnostic improvements now provide hints for `let` vs `set` usage.
- **Binder Refinements**: Fixed an issue where disjoint branch declarations were incorrectly flagged as redeclarations. `each_line` loop variables now correctly preserve their value if the loop does not run (0-iterations) and the variable was previously set.
