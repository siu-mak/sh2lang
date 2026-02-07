<a href="https://github.com/siu-mak/sh2lang">
  <img src="images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# sh2do — sh2 Snippet Runner

[sh2do](https://github.com/siu-mak/sh2lang) is a thin CLI wrapper around sh2c that compiles and executes sh2 snippets or files in one step. It wraps snippets into `func main() { ... }` (if needed), invokes sh2c, and runs the generated shell script.

> **New to sh2?** Start with the **[Tutorials](tutorials/index.md)**, specifically **[Getting Started](tutorials/01-getting-started.md)**.

## Install

From the workspace root:

```bash
cargo build
```

During development:

```bash
target/debug/sh2do --help
```

## Usage

### Run a file

```bash
sh2do script.sh2
```

> **Note**: sh2do invokes the shell as `bash -- script`, so script paths starting with `-` (e.g., `-script.sh2`) are safely handled and not treated as shell options.

### Run an inline snippet

```bash
sh2do 'print("hello world")'
```

### Snippet from stdin

```bash
echo 'print("hello")' | sh2do -
```

### With flags and arguments
 
 Flags can be placed **before or after** the script argument, but must appear before `--`.
 
 ```bash
 sh2do script.sh2 --target posix -- arg1 arg2
 sh2do --target posix script.sh2 -- arg1 arg2
 sh2do 'print(arg(1))' --target posix -- myarg
 ```
 
 > **Note**: sh2do requires exactly one positional argument (the file path or snippet). Directories are rejected.

## Flags

### `-e, --emit`
(File mode only) Compile, emit the generated shell script next to the source file (e.g. `script.sh` for `script.sh2`), and then execute it. It is **not** valid for inline snippets.

```bash
sh2do script.sh2 --emit
# Creates script.sh and runs it
```

### `-o <path>`
Compile to the specified output path and execute it.

```bash
sh2do script.sh2 -o custom.sh
```

### `--emit-sh`
Compile and emit the generated shell script to stdout **without** executing it. (Legacy alias: `--no-exec`)

```bash
sh2do 'print("hi")' --emit-sh
```

### `--target <bash|posix>`
Select the target shell dialect. Default is `bash`.

```bash
sh2do script.sh2 --target posix
```

### `--shell <bash|sh>`
Override the runtime shell used to execute the script.
Default mapping:
- `--target bash` -> `bash`
- `--target posix` -> `sh`

### `-h, --help`
Show help text and exit.

## Arguments Passthrough

Everything after `--` is passed verbatim to the executed script (via the interpreter's arguments). These arguments are accessible via `arg(n)` and `argc()` in your sh2 script.

```bash
sh2do script.sh2 -- hello
# Output: hello
```

Arguments are ignored in `--emit-sh` output mode (no execution).

## Exit Status

### Compile errors
If sh2c fails to compile, sh2do exits with sh2c's exit code and forwards stderr unchanged.

### Runtime errors
If the generated script executes and fails, sh2do exits with the script's exit code.

### Success
Exit code 0 indicates successful compilation and execution.

## Examples

### Run a file

```bash
sh2do mytool.sh2
```

### Emit and run

```bash
sh2do mytool.sh2 --emit
ls mytool.sh # exists
```

### Basic inline execution

```bash
sh2do 'print("hello world")'
```

### Using arguments

```bash
sh2do 'print("Hello, " & arg(1))' -- Alice
```

### Emit shell without execution (stdout)

```bash
sh2do 'print("test")' --emit-sh > script.sh
```

### POSIX target

```bash
sh2do script.sh2 --target posix
```

## Non-Goals

sh2do is intentionally minimal. For full control over compilation options (like IR emission or AST inspection), use `sh2c` directly.

---
# Docs

The GitHub repo is here:  
**[https://github.com/siu-mak/sh2lang](https://github.com/siu-mak/sh2lang)**

## Further Documentation

- [`docs/language.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/language.md) — full language reference (syntax + semantics)
- [`docs/sh2do.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/sh2do.md) — sh2do CLI documentation
- `tests/` — fixtures and integration tests (acts as an executable spec)

---

## Common Next Reads

*   **[Error Handling](articles/features/15-error-handling.md)**: Patterns for robust scripts.
*   **[No Implicit Expansion](articles/features/13-no-implicit-expansion.md)**: Why strings are strict literals.
*   **[sudo Builtin](articles/features/11-sudo-builtin.md)**: Safe privilege escalation.
