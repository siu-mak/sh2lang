# Unreleased

## Features
- **Range syntax for `for` loops**: Added inclusive range syntax `start..end` for iteration (e.g., `for i in 1..10 { ... }`). Supports both parenthesized `(1..10)` and spaced `1 .. 10` forms. Runtime dependency: requires `seq` command (part of coreutils).

## Docs
- Clarified `sh(...)` isolation: does not inherit `$@` / `$1`; documented using `argc()` / `arg(n)` or `run(...)` instead.
