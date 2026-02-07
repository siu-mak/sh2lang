# Unreleased

## Features
- **Range syntax for `for` loops**: Added inclusive range syntax `start..end` for iteration (e.g., `for i in 1..10 { ... }`). Supports both parenthesized `(1..10)` and spaced `1 .. 10` forms. Runtime dependency: requires `seq` command (part of coreutils).
- **Path Lookup**: Added `which(name)` builtin to resolve executables in `$PATH` without external dependencies. Returns the path or `""` if not found. **Exit status**: returns 0 when found, 1 when not found (allows `if which("cmd") { ... }`). **Non-aborting**: `which()` returning 1 does not abort the script. Paths may be relative if `$PATH` contains relative entries.


## Diagnostics
- **Unknown function detection**: Calling an undefined function in expression context now produces a compile error with hints to use `run(...)` for external commands or define the function.

## Docs
- Clarified `sh(...)` isolation: does not inherit `$@` / `$1`; documented using `argc()` / `arg(n)` or `run(...)` instead.
