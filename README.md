# sh2c — A Structured Shell Language Compiler

sh2c is a prototype compiler for **sh2**, a small structured shell language. sh2 is designed to reduce common shell “footguns” by making intent explicit (structured control flow, explicit variable declaration, scoped environment/IO changes) while compiling down to a conventional shell script.

sh2c can compile the same `.sh2` source to either:

- **bash** (default; more features)
- **POSIX sh** (more portable; fewer features)

---

## Install / Build

### Build from source

```bash
git clone https://github.com/siu-mak/sh2lang.git
cd sh2lang
cargo build --workspace
```

Run the compiler:

```bash
./target/debug/sh2c --help
./target/debug/sh2do --help
```

### Install to PATH (recommended for regular use)

Install both tools to `~/.cargo/bin`:

```bash
cargo install --path sh2c --locked
cargo install --path sh2do --locked
```

Ensure `~/.cargo/bin` is on your PATH:

```bash
# Add to ~/.bashrc or ~/.zshrc:
export PATH="$HOME/.cargo/bin:$PATH"
```

Then from any directory:

```bash
sh2c --help
sh2do 'print("hello")'
```

---

## CLI Usage

```text
Usage: sh2c [flags] <script.sh2> [flags]

Flags:
  --target <bash|posix>  Select output shell dialect (default: bash)
  -o, --out <file>       Write output to file instead of stdout (auto-chmod +x)
  --check                Check syntax and semantics without emitting code
  --no-diagnostics       Disable error location reporting and traps
  --no-chmod-x           Do not set executable bit on output file
  --chmod-x              Set executable bit on output file (default)
  --emit-ast             Emit AST (debug)
  --emit-ir              Emit IR (debug)
  --emit-sh              Emit Shell (default)
  -h, --help             Print help information
```

### Compile to stdout

```bash
sh2c your_script.sh2
```

### Compile to a file (auto `chmod +x` by default)

```bash
sh2c -o your_script.sh your_script.sh2
./your_script.sh
```

### Disable `chmod +x` on output

```bash
sh2c --no-chmod-x -o your_script.sh your_script.sh2
```

### Check-only mode

```bash
sh2c --check your_script.sh2
```

### Debug outputs

```bash
sh2c --emit-ast your_script.sh2
sh2c --emit-ir  your_script.sh2
sh2c --emit-sh  your_script.sh2   # default
```

---

## Quick sh2 Example (tested-shape)

A minimal program is **imports + functions only**. The compiler-generated shell will invoke `main()`.

```sh2
func main() {
  let who = capture(run("whoami"))
  print("hello " & who)
}
```

Compile and run (bash target):

```bash
sh2c --target bash -o hello.sh hello.sh2
./hello.sh
```

---

## Language Highlights

### Program structure

- **No top-level statements.** Only `import ...` and `func ... { ... }` at file scope.
- Entry point is **`main()`**; sh2c emits a wrapper that calls it.

### Functions have named parameters

sh2 supports named parameters in function signatures:

```sh2
func greet(name, title) {
  print("Hello, " & title & " " & name)
}
```

### Newline-separated statements (no `;`)

Statements inside `{ ... }` must be separated by newlines. `;` is not a statement separator.

### `env` is reserved

`env` is a reserved keyword for environment access (`env.HOME`) and cannot be used as an identifier:

```sh2
# ✅
let env_name = "dev"

# ❌
let env = "dev"
```

### Logical operators are `&&` / `||`

Use `&&` for logical AND and `||` for logical OR:

```sh2
if exists("a") && exists("b") {
  print("both")
}

if exists("a") || exists("b") {
  print("either")
}
```

### `run(...)` is an expression

`run(...)` can be used as a statement, but also participates in boolean logic:

```sh2
run("true") && run("echo", "only if success")
run("false") || run("echo", "only if failure")
```

To allow a command to fail without aborting the script, use `allow_fail=true`:

```sh2
run("grep", "x", "missing.txt", allow_fail=true)
print("exit code: " & status())
```

### Pipelines are more than `run | run`

Pipelines connect stages with `|`. The implementation supports **mixed stages** (not just `run(...)`), as validated by pipe-block mixed-stage tests.

Common pattern:

```sh2
run("printf", "hello\n") | run("tee", "out.txt")
```

### `print(...)` is not a pipeline stage

`print(...)` and `print_err(...)` are **statements**, not pipeline stages. If you need to “print and pipe”, use `printf` + `tee`:

```sh2
run("printf", "hello\n") | run("tee", "out.txt")
```

### `case` uses `=>` and supports `glob(...)` + `_`

```sh2
func main() {
  let filename = "report.txt"
  case filename {
    glob("*.txt") => { print("text") }
    _ => { print("other") }
  }
}
```

### `try/catch`

```sh2
try {
  run("false")
} catch {
  print_err("failed: " & status())
}
```

### `sh("...")` raw shell escape hatch

`sh(expr)` executes raw shell code. It accepts **any string expression** (literal, variable, or concatenation):

```sh2
sh("echo hello")          # ✅ literal
let cmd = "echo dynamic"
sh(cmd)                   # ✅ variable
sh("echo " & cmd)         # ✅ concatenation
```

> **Warning**: `sh(expr)` is injection-prone if you interpolate untrusted input. Prefer structured `run(...)`, pipelines, and `capture(...)` instead.

`sh(expr)` uses **probe semantics**: it updates `status()` but never triggers fail-fast behavior.

---

## Targets and Portability

### `--target bash` (default)

Bash target supports the full implemented feature set, including:

- Lists and maps (`[...]`, `{...}`) plus indexing and iteration
- `with log(...) { ... }` fan-out logging (Bash-only)
- full `try_run(...)` result capture (`.stdout`, `.stderr`)

### `--target posix`

POSIX target prioritizes portability. Some features are restricted or unavailable, notably:

- Lists/maps (no arrays in POSIX sh)
- `with log(...)` (process substitution)
- potentially full `.stdout` / `.stderr` capture for `try_run(...)` (implementation-dependent)

---

## sh2do — Snippet Runner

**sh2do** is a thin wrapper around sh2c that compiles and executes sh2 snippets in one step. It's useful for quick one-liners and testing.

```bash
sh2do 'print("hello world")'
sh2do 'print(arg(1))' -- myarg
```

See [`docs/sh2do.md`](docs/sh2do.md) for full documentation.

---

## Further Documentation

- [`docs/language.md`](docs/language.md) — full language reference (syntax + semantics)
- [`docs/sh2do.md`](docs/sh2do.md) — sh2do CLI documentation
- `tests/` — fixtures and integration tests (acts as an executable spec)
