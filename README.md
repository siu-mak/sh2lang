# sh2lang

<img src="images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />

This repository contains tools for **sh2**, a structured shell language designed to compile down to conventional shell scripts (bash/posix targets).

## Repo contains

- **sh2c**: The compiler implementation (Rust)
- **sh2do**: A snippet runner for quick one-liners and testing
- **editors/vscode**: Syntax highlighting extension for VS Code ([Manual Install Guide](editors/vscode/README.md))

---

## About sh2

Shell is everywhere, but it’s also easy to write a script that *looks fine* and still breaks in boring, expensive ways: a space in a filename, a missed quote, a `cd` that leaks into the rest of the script, or an error code that gets ignored.

sh2 exists to keep the “I can run anything” power of shell, while making the risky parts harder to do by accident. The idea is simple:

- **Make intent obvious** (real `if/while/case`, real variables)
- **Make side-effects local** (change env/cwd/redirects in a block, then automatically go back)
- **Make commands safe by default** (`run("cmd", "arg1", "arg2")` instead of stringly shell code)

Check for tools easily without fragile `command -v` constructs:

```sh2
if which("docker") != "" {
  run("docker", "ps")
}
```

### Example: quoting and spaces (classic shell pain)

Bash (easy to get wrong with spaces/word-splitting):

```bash
pattern="foo bar"
file="my file.txt"
grep $pattern $file   # breaks: becomes 4 args instead of 2
```

sh2 (args are always separate):

```sh2
let pattern = "foo bar"
let file = "my file.txt"
run("grep", pattern, file)
```

### Example: “did that command fail?” (and what happens next)

Bash often ends up with scattered `|| exit 1`, `$?`, or half-working `set -e` patterns.

sh2 defaults to “fail fast”, but you can explicitly allow failure and then check the exit code:

```sh2
run("grep", "x", "missing.txt", allow_fail=true)
print("exit code was " & status())

if status() != 0 {
  print("not found (but script continues)")
}
```

### Example: stop leaking `cd`, env vars, and redirects everywhere

Bash:

```bash
cd /tmp
DEBUG=1 mytool >out.log 2>&1
cd - >/dev/null
```

sh2 keeps those changes inside a block:

```sh2
with cwd("/tmp") {
  with env { DEBUG: "1" } {
    with redirect { stdout: "out.log", stderr: stdout } {
      run("mytool")
    }
  }
}
# after the block: cwd/env/redirect are back to normal
```

### Bottom line

You still end up with a regular **bash/POSIX sh script** as output, but you write the source in a way that’s easier to review, harder to accidentally break, and more predictable to run.

---

## Installation (APT – Ubuntu 22.04 / jammy)

For **Ubuntu 22.04 (jammy)**, the recommended way to install sh2lang is via the official APT repository.

```bash
# Import the sh2lang APT signing key
curl -fsSL https://siu-mak.github.io/sh2lang/sh2lang.asc \
  | sudo tee /usr/share/keyrings/sh2lang-archive-keyring.asc >/dev/null

# Add the APT repository
echo "deb [signed-by=/usr/share/keyrings/sh2lang-archive-keyring.asc] https://siu-mak.github.io/sh2lang/ jammy main" \
  | sudo tee /etc/apt/sources.list.d/sh2lang.list

# Install
sudo apt-get update
sudo apt-get install sh2lang
```

Verify the installation:

```bash
sh2c --version
sh2do --help
```

## Alternative: build from source

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

> **Note (for building from source):**
> If you plan to build sh2lang from source instead of using the APT package,
> you need a recent Rust toolchain and standard build tools.
>
> See **[Getting Started](docs/tutorials/01-getting-started.md#prerequisites)** for details.

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

## Examples

### 1. Compilation with `sh2c`

**Input (`hello.sh2`):**

```sh2
func main() {
  print("Hello from sh2")
  run("echo", "Args handled safely")
}
```

**Compile and run:**

```bash
# Compile to bash script
sh2c --target bash -o hello.sh hello.sh2

# Run it
./hello.sh
```

### 2. Snippets and Files with `sh2do`

`sh2do` compiles and runs snippets or files instantly.

```bash
# Run a file
sh2do myscript.sh2

# File with flags and arguments (arguments passed after -- are forwarded to the script)
sh2do myscript.sh2 --target posix -- arg1

# Simple print
sh2do 'print("Hello World")'

# Semicolons allow multiple statements
sh2do 'print("Start"); run("date"); print("End")'
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

> **Note**: Function calls are validated at compile time. Call undefined functions and you'll get an error with hints. Use `run("cmd", ...)` for external commands.

### Statement Separation

Statements inside `{ ... }` can be separated by newlines or semicolons (`;`). Semicolons are optional and mainly useful for one-liners.

```sh2
func main() {
  print("first"); print("second")
}
```

Semicolons are **not** allowed inside expressions (e.g. `(1; 2)` is invalid).

### `env` is reserved

`env` is a reserved keyword for environment access (`env.HOME`) and cannot be used as an identifier:

```sh2
# ✅
let env_name = "dev"

# ❌
let env = "dev"
```

### `with` environment and redirects
 
 ```sh2
 with env { DEBUG: "1" } {
   run("build")
 }
 
 # Simple redirect
 with redirect { stdout: file("out.log", append=true) } {
   run("echo", "data")
 }
 
 # Multi-sink redirect (fan-out to file + terminal)
 with redirect { stdout: [file("out.log"), inherit_stdout()] } {
   print("hello")
 }
 ```

### No Implicit Expansion
sh2 is stricter than Bash: it performs **no implicit expansion** (no Bash parameter expansion like `$foo` or `${bar}`, no globbing `*`, no splitting) in string literals or variables.
- Use `env.HOME` instead of `~`.
- Join paths with `&`.

```sh2
# Correct
with cwd(env.HOME & "/repos") { ... }

# Incorrect (treated as literal tilde)
with cwd("~/repos") { ... }
```

### Variables

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

### `capture(...)` for command output

Use `capture(...)` to get the stdout of a command as a string:

```sh2
let files = capture(run("ls", "-1"))
if status() == 0 {
  print("Files found: " & files)
}

# Nested allow_fail is also supported and equivalent to outer form
let out = capture(run("grep", "pattern", "file", allow_fail=true))
```

### `sudo(...)` for privileged commands

Execute commands with `sudo` using structured options:

```sh2
# Basic usage
sudo("systemctl", "restart", "nginx")

# With user option
sudo("ls", "/root", user="admin")

# Mixed argument ordering
sudo(user="root", "apt-get", "update")
```

Supported options: `user`, `n` (non-interactive), `k`, `prompt`, `E`, `env_keep`, `allow_fail` (statement-form only).

### Pipelines are more than `run | run`

Pipelines connect stages with `|`. The implementation supports **mixed stages** (`run(...)`, `sudo(...)`, `{ ... }`), as validated by pipe-block mixed-stage tests.

Common pattern:

```sh2
run("printf", "hello\n") | run("tee", "out.txt")

# Sudo in pipeline (use named options for flags)
run("cat", "shadow.txt") | sudo("tee", "/dev/null", n=true)
```

### `print(...)` is not a pipeline stage

`print(...)` and `print_err(...)` are **statements**, not pipeline stages. If you need to “print and pipe”, use `printf` + `tee`:

```sh2
run("printf", "hello\n") | run("tee", "out.txt")
```

### Pipe blocks
Pipelines can also include blocks using the `pipe` keyword or by mixing `run` and `{ ... }`:

```sh2
pipe {
  print("producing lines...")
  run("ls", "-1")
} | {
  run("grep", "txt")
}
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
> **Note**: `sh(...)` runs in a fresh shell context. It does **not** receive script positional parameters; use `argc()`/`arg(n)` in sh2 or `run(...)` for safe argument passing.

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

## Documentation

- **[Documentation Home](docs/index.md)** — Start here!
- **[Tutorials](docs/tutorials/index.md)** — Step-by-step guides.
- **[Articles](docs/articles/index.md)** — Deep dives and explanations.
- **[Reference](docs/language.md)** — Language syntax and semantics.
- **[CLI Reference](docs/sh2do.md)** — sh2do tool usage.
- `tests/` — fixtures and integration tests (executable spec).

## Versions

- [v0.1.1](docs/releases/v0.1.1.md) — Adds `sudo(...)`, `confirm(...)`, `$"..."` interpolation, semicolons.
- [v0.1.0](docs/releases/v0.1.0.md) — First public release of the sh2 structured shell language.



