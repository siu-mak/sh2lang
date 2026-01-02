# sh2c: A Structured Shell Language Compiler

`sh2c` is a prototype compiler for **sh2**, a small, structured shell language designed to bring safety and modern programming constructs to shell scripting.
`sh2c` translates `.sh2` source into either **bash** or **POSIX sh**, so you can write one script and choose the portability/performance trade-off at compile time.

## Design goals

- **Structure & safety:** explicit `run(...)` command execution, `let` declarations, structured control flow (`if`, `while`, `for`, `case`, `try/catch`).
- **Clarity:** readable, explicit syntax with fewer shell footguns.
- **Dual targets:** `--target bash` (feature-rich) and `--target posix` (portable). Some features are intentionally Bash-only.

---

## Project status

sh2c is a **robust, functional prototype** validated by an extensive test suite covering parsing, code generation, and runtime semantics.

### Implemented core capabilities

- **Variables:** `let` declaration and `set` reassignment.
- **Control flow:** `if/elif/else`, `while`, `for`, `case` (with `=>` arms), `try/catch`.
- **Command execution:** `run(...)`, pipelines (`|`), `exec(...)`, and `capture(...)` (where enabled).
- **Run controls:** `run(..., allow_fail=true)` to suppress failure propagation for a single command.
- **Run results:** `try_run(...) -> RunResult` with `.status`, `.stdout`, `.stderr`.
- **Modules:** `import "path"` with cycle detection and recursive resolution.
- **Scopes:** `with env {...}`, `with cwd(...)`, `with redirect {...}`, and `with log(...)` (**Bash-only**).
- **Builtins & stdlib helpers:** filesystem predicates, regex `matches`, arg parsing (`parse_args`), envfile helpers, JSON emitter (`json_kv`), string/file helpers (`split/join/lines/trim/replace/read_file/write_file`, etc.).

### Known gaps / limitations

- **POSIX target limitations:** features that require arrays, associative arrays, process substitution, or interactive prompts will error/panic on `--target posix`.
- **Some advanced shell forms are not native syntax:** e.g., bash process substitution like `diff <(cmd1) <(cmd2)`; use explicit `sh("...")` if absolutely needed (see limitations below).

---

## CLI usage

```bash
sh2c [flags] <script.sh2> [flags]
```

Current flags (matches `sh2c --help`):

- `--target <bash|posix>`  Select output shell dialect (default: bash)
- `-o, --out <file>`       Write output to file instead of stdout (**auto-chmod +x**)
- `--check`                Check syntax and semantics without emitting code
- `--no-diagnostics`       Disable error location reporting and traps
- `--no-chmod-x`           Do not set executable bit on output file
- `--chmod-x`              Set executable bit on output file (**default**)
- `--emit-ast`             Emit AST (debug)
- `--emit-ir`              Emit IR (debug)
- `--emit-sh`              Emit Shell (**default**)
- `-h, --help`             Print help information

### Compile to stdout

```bash
sh2c --target bash ./script.sh2
```

### Compile to a file (executable)

```bash
sh2c --target bash -o ./script.sh ./script.sh2
./script.sh
```

`-o/--out` writes the generated script to a file and sets `chmod +x` by default.
Use `--no-chmod-x` if you do not want the output file to be executable.

### Check-only

```bash
sh2c --check ./script.sh2
```

---

## Quick example (tested style)

This example demonstrates file predicates, pipelines, and status handling.

```sh2
func main() {
  let file_path = "/etc/hosts"

  if exists(file_path) {
    print("File found: " & file_path)

    # wc -l outputs: "  <N> <path>"
    # awk extracts the first field
    let line_count = capture(run("wc", "-l", file_path) | run("awk", "{print $1}"))

    print("It contains " & line_count & " lines.")
  } else {
    print_err("Error: File not found at " & file_path)
    exit(1)
  }
}
```

### Expected bash output characteristics

When targeting bash, generated output:
- includes a shebang (`#!/usr/bin/env bash`)
- uses a strict/error-tracking runtime model
- tracks status via `status()` / internal `__sh2_status`

---

## Language at a glance

For full details, see `language.md` and `grammar.enbf.md`. Below are the key points that frequently trip people up.

### Program structure (important)

- Programs are **function definitions only**.
- **No top-level statements** (the compiler-generated shell invokes `main()`).
- `env` is a **reserved keyword** (use `env_name`, etc.).

### Statement separation

- Semicolons `;` are **not** valid statement separators inside blocks.
- Use newlines between statements.

### `case` uses `=>`

Case arms use `=>` (not `->`).

```sh2
case cmd {
  "env" => { print("env") }
  _ => { print_err("unknown"); exit(2) }
}
```

### `sh("...")` is literal-only

`sh(...)` currently accepts only a **string literal**, not a concatenation or variable.

✅ `sh("echo hello")`  
❌ `sh("echo " & name)`  
❌ `sh(cmd)`

Prefer structured `run(...)` calls where possible.

### `print(...)` can’t be piped

`print(...)` is a statement, not a pipeline stage. To write to a file with piping:

```sh2
run("printf", "IMAGE_TAG=%s\n", "demo:latest") | run("tee", "/tmp/env.meta")
```

Note: `tee` echoes to stdout while writing the file.

---

## Run behavior controls

### `run(..., allow_fail=true)`

Suppress failure propagation for a single run call while still updating `status()`:

```sh2
run("false", allow_fail=true)
print("status=" & status())
```

### `try_run(...) -> RunResult`

Capture stdout/stderr/status without aborting on failure:

```sh2
let r = try_run("sh", "-lc", "echo out; echo err 1>&2; exit 7")
print("status=" & r.status)
print("stdout=" & r.stdout)
print_err("stderr=" & r.stderr)
```

---

## Logging & interactive (Bash-only)

### `with log(path, append=...) { ... }`

Bash-only fan-out logging (console + file). Errors/panics on `--target posix`.

```sh2
with log("/tmp/devctl.log", append=true) {
  print("hello")
  run("printf", "tag=%s\n", "demo:latest")
}
```

### `input(...)`, `confirm(...)`

Interactive primitives (Bash-only). They error/panic on `--target posix`.

---

## Targets & portability

### `--target bash` (default)

Supports the full implemented feature set, including:
- lists/maps (and indexing/iteration)
- `with log(...)`
- `input(...)`, `confirm(...)`
- `try_run(...) -> RunResult` with `.status/.stdout/.stderr`

### `--target posix`

Maximal portability. Some features are intentionally unsupported and will error/panic, including:
- lists/maps (no arrays in POSIX sh)
- `with log(...)` (no process substitution)
- `input(...)`, `confirm(...)` (interactive helpers)

---

## Documentation

- `language.md` — descriptive language reference with gotchas and examples
- `grammar.enbf.md` — EBNF grammar for the sh2 syntax
- `tests/` — fixtures and integration tests showing supported patterns (e.g., `try_run_success.sh2`, `try_run_fields.sh2`, `matches_basic.sh2`, etc.)
