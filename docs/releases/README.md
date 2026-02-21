# Release notes

Release history for the **sh2lang** project.

## Releases

- [v0.1.2](v0.1.2.md) — Range loops, `which`/`glob`/`find_files`/`find0` builtins, `each_line` pipelines, job control (`spawn`/`wait`/`wait_all`), `stdin_lines`, strict variable semantics
- [v0.1.1](v0.1.1.md) — `sudo`/`confirm` builtins, strict string literals, `$"..."` interpolation, pipe blocks, `sh2do` file mode, `contains`/`contains_line` fixes, boolean encoding
- [v0.1.0](v0.1.0.md) — First public release: core language (`func`, `let`, `if`, `for`, `case`, `try`), `run`/`capture`/`sh`, Bash+POSIX codegen, `sh2c` compiler, `sh2do` runner

## How to add a new release

1. Create a new file `docs/releases/vX.Y.Z.md`.
2. Add an entry to the list above.
3. Follow the project's release guidelines.
