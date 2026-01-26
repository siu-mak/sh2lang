# The sh2 Language Reference

sh2 is a small, structured shell language designed to bring safety, clarity, and modern programming constructs to shell scripting. Scripts written in sh2 are compiled by **sh2c** into either **bash** (feature-rich) or **POSIX sh** (portable).

> **Note**: **sh2do** is a wrapper tool that compiles and executes sh2 snippets in one step. It does not change sh2 language semantics.

---

## 1. Program Structure

A program consists of:

- zero or more `import "path"` statements (must come first), and
- one or more `func ... { ... }` function definitions.

**Top-level executable statements are not allowed.** The compiler emits a shell entrypoint that invokes `main()`.

### 1.1 Imports

```sh2
import "lib/utils.sh2"
```

- Imports must appear before any function definitions.
- Imports are resolved recursively.
- Import cycles are detected and reported.
- All imported functions share a single namespace; duplicate function names are an error.

### 1.2 Functions and Parameters

Functions are defined with **named parameters**:

```sh2
func greet(name, title) {
  print("Hello, " & title & " " & name)
}
```

Parameters are bound positionally (first param receives the first argument, etc.).

The designated entry point is:

```sh2
func main() {
  run("echo", "hi")
}
```

---

## 2. Core Syntax Rules

### 2.1 Statement separation
 
 Statements are separated by **newlines** or **semicolons** (`;`). Semicolons act as optional statement terminators.
 
 ✅ Correct:
 ```sh2
 func ok() {
   let a = "x"
   let b = "y"
   print("one"); print("two")
   print("three");
 }
 ```
 
 Semicolons are not allowed inside expressions:
 ❌ Incorrect:
 ```sh2
 let x = (1; 2)
 ```

### 2.2 Reserved identifiers

`env` is a reserved keyword (used for environment access like `env.HOME`) and cannot be used as a variable or function name.

✅
```sh2
let env_name = "dev"
```

❌
```sh2
let env = "dev"
```

### 2.3 Comments

Single-line comments start with `#`.

```sh2
# comment
let x = "hello" # inline comment
```

---

## 3. Data Types and Literals

### 3.1 Strings

- Double-quoted strings support interpolation (`$name`, `${expr}`) depending on context.
- Concatenation uses `&` (requires whitespace on both sides, e.g. `x & y`).


```sh2
let name = "Alice"
print("Hello, " & name)
print("Hello, $name")
```

### 3.2 No Implicit Expansion

Unlike implicit shells (Bash/Zsh), sh2 does **not** perform implicit word splitting, globbing/wildcard expansion, or tilde expansion on string literals or variable content.

- `~` is treated as a literal character.
- `*` is treated as a literal character.
- Spaces in variables are preserved as-is.

**Recommendation**:
- Use `env.HOME` instead of `~`.
- Use `sh("ls *.txt")` or similar if you need shell expansion.

If you attempt to use `~/path` in `with cwd()` or similar, sh2 will fail at runtime and emit a helper hint advising you to use `env.HOME`.

Multiline and raw strings are supported (where implemented by your compiler target):

```sh2
let cooked = """
line 1
line 2 with \t tab
"""

let raw = r"""
this is raw
\n stays two chars
"""
```

### 3.2 Numbers

Integer literals (e.g. `0`, `42`). Arithmetic operators: `+ - * / %`.

### 3.3 Booleans

`true` and `false`.

Boolean expressions can be stored in variables and used in conditions:

```sh2
let ok = (sum == 42)
if ok {
  print("yes")
}
```

Stored booleans are represented as `"1"` (true) or `"0"` (false) internally.

> **Limitation**: Boolean variables can only be used in conditions (`if`, `while`). Using them in string contexts (e.g., `print(ok)` or `"x=" & ok`) produces a compile error. If you need string output, use `bool_str()`:
>
> ```sh2
> # ❌ Not supported:
> print(ok)
>
> # ✅ Use bool_str():
> print(bool_str(sum == 42))
> ```

- **Restriction**: The path MUST be a string literal at compile time (Model 2 restriction). Computed paths (e.g. variables, concatenation) are rejected with a compile error.
   - To use a computed path, explicitly use the canonical safe pattern with `run("sh", "-c", ...)`.
   
   ```sh2
   # ✅ Supported
   with cwd("/tmp/build") { run("make") }
   
   # ❌ Rejected (compile time)
   let d = "/tmp"
   # with cwd(d) { ... } -> Error: cwd(...) requires a string literal path.
   
   # Workaround for computed paths (safe):
   # Pattern: run("sh", "-c", script, arg0_name, arg1_path)
   # We pass "sh2" as the script name ($0), and the path as the first argument ($1).
   # Note: use "\$1" to prevent sh2 from interpolating $1 as a variable.
   run("sh", "-c", "cd \"\$1\" && ls", "sh2", d)
   ```


### 3.4 Lists (Bash-only)

```sh2
let xs = ["a", "b", "c"]
print(xs[0])
```

### 3.5 Maps (Bash-only)

```sh2
let m = { "k": "v" }
print(m["k"])
```

---

## 4. Variables and Assignment

### 4.1 Declaration: `let`

```sh2
let msg = "hello"
```

### 4.2 Reassignment: `set`

```sh2
let n = 0
set n = n + 1
```

### 4.3 Environment access

- Dot access: `env.HOME`
- Dynamic access: `env("HOME")`

```sh2
let base = env.HOME & "/sh2c/docker-rootless"
```

Environment mutation:

```sh2
set env.DEBUG = "1"
export("DEBUG")
unset("DEBUG")
```

---

## 5. Expressions and Operators

### 5.1 Operator precedence (lowest → highest)

1. `|` (pipeline)
2. `||`
3. `&&`
4. comparisons: `== != < <= > >=`
5. `&` (string concatenation)
6. `+ -`
7. `* / %`
8. unary: `!` and unary `-`
9. postfix: calls `f(...)`, indexing `x[i]`, member access `x.field`

### 5.2 Logical operators: `&&` / `||`

Use `&&` for logical AND and `||` for logical OR:

```sh2
if exists("a") && exists("b") {
  print("both")
}

if exists("a") || exists("b") {
  print("at least one")
}
```

### 5.3 Pipelines

Pipelines connect **stages** with `|`.

- They are broader than just `run(...) | run(...)`.
- Implementations include pipeline stages that may be blocks / statements in pipe contexts.

```sh2
# Block as producer
pipe {
  print("line 1")
  print("line 2")
} | run("grep", "2")

# Block as consumer
run("ls") | {
  let output = capture(input(""))
  print("Captured: " & output)
}

# Mixed run/block
pipe run("echo", "data") | { run("cat") }
```

> Note: `print(...)` is a **statement**, not a pipeline stage. To “print and pipe”, use `run("printf", ...)` OR use a `pipe { ... }` block where `print` writes to stdout of the block.

---

## 6. Command Execution

### 6.1 `run(...)` (expression)

`run(...)` executes an external command with safely separated arguments. It is an **expression**, so it can be used:

- as a standalone statement (expression statement), and
- inside boolean logic (`&&`/`||`) and conditions.

```sh2
run("echo", "hello")

run("true") && run("echo", "only if true succeeded")
run("false") || run("echo", "only if false failed")
```

By default, failures abort the script (set -e-like behavior), unless you enable `allow_fail=true`.

```sh2
run("grep", "x", "missing.txt", allow_fail=true)
print("exit code was " & status())
```

### 6.2 `exec(...)` (statement)

Replaces the current process. Execution does not continue after `exec`.

```sh2
exec("bash")
```

### 6.3 `sh(expr)` (raw shell execution)

> [!WARNING]
> **Unsafe escape hatch**: `sh(expr)` interprets `expr` as raw shell code and is **injection-prone** if you build `expr` by concatenating or interpolating untrusted input. This is intentional—it provides an escape hatch for advanced use cases, not a safe API.

Executes a shell snippet by passing it to the target shell in a child process.

**Execution model:**
- **Child shell process**: Runs via `bash -c "$cmd"` (Bash target) or `sh -c "$cmd"` (POSIX target)
- **No persistence**: Changes to working directory (`cd`), non-exported variables, or other local shell state do **not** affect subsequent statements
- **Inherits environment**: Exported environment variables are inherited by the child shell

**Probe semantics (non-fail-fast):**
- Updates `status()` with the command's exit code
- **Never** triggers fail-fast behavior or exits the script on non-zero status
- Returns control unconditionally

**Accepts any string expression:**
```sh2
sh("echo hello")
let cmd = "echo dynamic"
sh(cmd)
sh("echo " & cmd)
```

**Probe pattern** (explicit status check):
```sh2
sh("false")
if status() != 0 {
  print("Command failed as expected")
}
print("Script continues")  # Always executes
```

**Non-persistence example:**
```sh2
sh("cd /tmp")
# pwd() still returns original directory
# cd inside sh() does not affect parent script
```

#### Safe alternatives

For most use cases, prefer these safer options:
- **`run(...)`**: Argument-safe command execution with proper quoting
- **Native pipelines**: Use `|` operator for structured pipeline composition
- **String helpers**: `lines(...)`, `split(...)`, `trim()`, `replace()` for text processing
- **Upcoming helpers**: Additional safe utilities for common shell patterns

Use `sh()` only when you need raw shell syntax that cannot be expressed through safe APIs.

### 6.4 `capture(...)` (capture stdout)

`capture(...)` captures stdout from a structured command/pipeline expression.

Typical examples:

```sh2
let who = capture(run("whoami"))
let n = capture(run("printf", "a\n") | run("wc", "-l"))
```

### 6.5 `try_run(...)` → `RunResult`

Runs a command without aborting and returns a result object:

- `.status`
- `.stdout`
- `.stderr`

```sh2
let r = try_run("git", "rev-parse", "HEAD")
if r.status == 0 {
  print(r.stdout)
} else {
  print_err(r.stderr)
}
```

**Target note:** On `--target posix`, implementations may restrict or omit `.stdout` / `.stderr` capture (documented as target-dependent). `.status` is always available.

### 6.6 `sudo(...)` (privileged execution)

Structured wrapper for `sudo` command execution with type-safe options:

```sh2
# Basic usage
sudo("systemctl", "restart", "nginx")

# With user option
sudo("ls", "/root", user="admin")

# With environment preservation
sudo("env", env_keep=["PATH", "HOME"])
```

**Supported options:**
- `user` (string literal) — run as specified user (generates `-u`)
- `n` (boolean) — non-interactive mode (generates `-n`)
- `k` (boolean) — invalidate cached credentials (generates `-k`)
- `prompt` (string literal) — custom password prompt (generates `-p`)
- `E` (boolean) — preserve environment (generates `-E`)
- `env_keep` (list of string literals) — preserve specific variables (generates `--preserve-env=...`)
- `allow_fail` (boolean, statement-form only) — non-aborting execution

**Argument ordering:**
Mixed positional and named arguments are allowed:
```sh2
sudo(user="root", "ls")        # ✅
sudo("ls", user="root")        # ✅
sudo(n=true, "ls", user="root") # ✅
```

**Compile-time validation:**
- Option values must be literals:
  - `user`, `prompt`: string literals
  - `n`, `k`, `E`, `allow_fail`: boolean literals
  - `env_keep`: list of string literals
- Duplicate options are rejected
- Unknown options are rejected
- `allow_fail` in expression context is rejected with specific diagnostic

**Lowering behavior:**
- Generates stable flag ordering: `sudo -u ... -n -k -p ... -E --preserve-env=... -- cmd args...`
- Mandatory `--` separator before command arguments
- Statement-form with `allow_fail=true` behaves like `run(..., allow_fail=true)`

**Expression-form restriction:**
```sh2
# ❌ Not allowed:
let x = capture(sudo("ls", allow_fail=true))

# ✅ Use capture's allow_fail instead:
let x = capture(sudo("ls"), allow_fail=true)
```

Error message: `"allow_fail is only valid on statement-form sudo(...); use capture(sudo(...), allow_fail=true) to allow failure during capture"`



## 7. Status, Errors, and `try/catch`

### 7.1 `status()`

`status()` returns the exit code of the most recent operation and is updated by:

- `run(...)` (including `allow_fail=true`)
- `try_run(...)`
- `sh("...")`
- filesystem predicates like `exists(...)`, `is_file(...)`, etc.

### 7.2 `try { ... } catch { ... }`

If a command fails inside `try`, control transfers to `catch`. Inside `catch`, `status()` contains the failing status code.

```sh2
try {
  run("false")
  print("won't run")
} catch {
  print_err("failed: " & status())
}
```

---

## 8. Control Flow

### 8.1 `if / elif / else`

```sh2
if status() == 0 {
  print("ok")
} elif status() == 2 {
  print("special")
} else {
  print("bad")
}
```

### 8.2 `while`

```sh2
let i = 0
while i < 5 {
  print(i)
  set i = i + 1
}
```

### 8.3 `for`

List iteration (Bash-only when lists are used):

```sh2
let xs = ["a", "b", "c"]
for x in xs {
  print(x)
}
```

Map iteration (Bash-only when maps are used):

```sh2
let m = { "k": "v" }
for (k, v) in m {
  print(k & "=" & v)
}
```

### 8.4 `break` / `continue`

```sh2
let i = 0
while true {
  set i = i + 1
  if i == 3 { continue }
  if i > 5 { break }
  print(i)
}
```

### 8.5 `case`

Case arms use `=>`. Patterns include:

- string literal patterns
- `glob("pattern")`
- `_` wildcard default

```sh2
let filename = "report.txt"
case filename {
  glob("*.txt") => { print("text") }
  "README.md" => { print("readme") }
  _ => { print("other") }
}
```

---

## 9. Scoped Blocks (`with`)

### 9.1 `with env { ... } { ... }`

Verified syntax includes **colon bindings**:

```sh2
with env { DEBUG: "1", HOME: env.HOME } {
  run("env")
}
```

### 9.2 `with cwd(expr) { ... }`

```sh2
with cwd("/tmp") {
  run("pwd")
}
```

> **Note**: `cwd(...)` currently requires a string literal argument (e.g., `"/path"`). Computed paths are not yet supported. If you need a dynamic working directory, use `cd(expr)` (scoped via `subshell { ... }` if needed) or `sh("cd ...")`.

### 9.3 `with redirect { ... } { ... }`

Configure file descriptors for the scoped block. Supports single targets and **multi-sink lists** (fan-out).

**Single Targets**:

```sh2
# stdout to file (overwrite)
with redirect { stdout: file("out.log") } { ... }

# append mode
with redirect { stdout: file("out.log", append=true) } { ... }

# stderr to stdout (merge)
with redirect { stderr: to_stdout() } { ... }
```

**Multi-Sink Lists (Fan-out)**:

You can provide a list of targets to duplicate output (similar to `tee`).

```sh2
# Write to file AND keep visible on terminal
with redirect { stdout: [file("out.log"), inherit_stdout()] } { ... }

# Write to multiple files (silent on terminal)
with redirect { stdout: [file("a.log"), file("b.log")] } { ... }
```

- `inherit_stdout()` / `inherit_stderr()`: Keeps the output visible on the parent stream. If omitted from a list, the output is not shown on the terminal.
- **Legacy Keywords**: `stdout` and `stderr` can be used as synonyms for `to_stdout()` / `to_stderr()` in single-target contexts, but function-style `to_stdout()` is preferred.

**Restrictions**:

1. **Mixed Append**: A list cannot mix append modes. `[file("a", append=true), file("b")]` is invalid. All files in a multi-sink list must share the same append setting.
2. **POSIX Limitation**: Multi-sink redirects (lists with >1 target or usage of `inherit_*` with a file) are **not supported** when compiling with `--target posix`.
   - Error: *"multi-sink redirect is not supported for POSIX target; use a single redirect target or switch to --target bash"*
   - Exception: A single-element list like `[file("out.log")]` is allowed on POSIX.

### 9.4 `with log(path, append=true|false) { ... }` (Bash-only)

```sh2
with log("activity.log", append=true) {
  run("echo", "hello")
}
```

On `--target posix`, `with log` is not available.

---

## 10. Built-in Functions (selected)

### 10.1 I/O statements

- `print(expr)`
- `print_err(expr)`

> `print`/`print_err` are statements, not pipeline stages.

### 10.2 Filesystem predicates

- `exists(path)`
- `is_dir(path)`
- `is_file(path)`
- `is_symlink(path)`
- `is_exec(path)`
- `is_readable(path)`
- `is_writable(path)`
- `is_non_empty(path)`

### 10.3 Helpers (as implemented)

- string/list: `split`, `join`, `lines`, `trim`, `replace`
- regex: `matches(text, regex)`
- args: `parse_args()` and helpers
- envfiles: `load_envfile`, `save_envfile`
- JSON: `json_kv(...)`
- process/system: `pid()`, `ppid()`, `uid()`, `pwd()`, `argc()`, `argv0()`, etc.

### 10.4 Interactive Helpers

#### `confirm(prompt, default=...)` → boolean

Interactive yes/no confirmation prompt:

```sh2
if confirm("Proceed with deployment?") {
    run("deploy.sh")
}

# With default value
if confirm("Delete files?", default=false) {
    run("rm", "-rf", "data/")
}
```

**Behavior:**
- Returns `true` for yes, `false` for no
- Accepts `y`, `yes`, `Y`, `YES` as affirmative (case-insensitive)
- Accepts `n`, `no`, `N`, `NO` as negative (case-insensitive)
- Optional `default=true` or `default=false` parameter

**Non-interactive mode:**
- If `default` is provided, uses that value when stdin is not a terminal
- If `default` is not provided, fails with error in non-interactive mode

**Environment overrides:**
- `SH2_YES=1` — always return `true`
- `SH2_NO=1` — always return `false`

**Example with default:**
```sh2
# Safe for CI/automation
if confirm("Apply changes?", default=false) {
    run("apply.sh")
}
```

#### `input(prompt)` → string

Read user input from stdin:

```sh2
let name = input("Enter your name: ")
print("Hello, " & name)
```

### 10.5 String and List Utilities

#### `contains_line(text, needle)`

Boolean predicate that evaluates to `true` if `text` contains a line exactly equal to `needle`.

- **Exact match**: strict string equality (no regex/glob).
- **Portable**: Works on both Bash and POSIX targets.
- **Trailing newline**: A trailing `\n` does not imply an extra empty line.
  - `contains_line("a\n", "")` is `false`.
  - `contains_line("a\n\n", "")` is `true`.

```sh2
if contains_line(run("ls").stdout, "Makefile") { ... }
```

#### `contains(list, value)` (Bash-only)

Evaluates to `true` if `list` (array or expression evaluating to list) contains exactly `value`. Performs strict string equality check (no globbing).

```sh2
let items = ["a", "b"]
if contains(items, "b") { ... }

let text = "foo\nbar\n"
if contains(lines(text), "bar") { ... }
```

### 10.5 File I/O

#### `read_file(path)` → string

Reads the contents of a file and returns it as a string. Must be used in an expression context (cannot be used as a statement).

```sh2
let content = read_file("config.txt")
```

- **Error behavior**: If the file does not exist or cannot be read, the script exits with a non-zero status (fail-fast).
- **Newlines**: Content is returned exactly as stored, including trailing newlines.
- **Portable**: Works on both Bash and POSIX targets.

#### `write_file(path, content)`

Creates or truncates `path` and writes `content` exactly as provided. This is a **statement**, not an expression.

```sh2
write_file("output.txt", "hello")
write_file("data.txt", content & "\n")  # explicit newline
```

- **No implicit newline**: Content is written exactly; add `\n` explicitly if needed.
- **Error behavior**: If the file cannot be written (e.g., path is a directory), the script exits with a non-zero status.
- **Portable**: Works on both Bash and POSIX targets.

#### `append_file(path, content)`

Appends `content` to `path`, creating the file if it does not exist. This is a **statement**, not an expression.

```sh2
append_file("log.txt", "entry\n")
```

- **No implicit newline**: Content is appended exactly; add `\n` explicitly if needed.
- **Error behavior**: Same as `write_file`.
- **Portable**: Works on both Bash and POSIX targets.

---

## 11. Targets and Portability

### `--target bash` (default)
Supports the full implemented feature set, including lists/maps, `with log`, interactive helpers (if enabled), and full `try_run` capture.

### `--target posix`
Prioritizes portability. Bash-only features (lists/maps, `with log`, and potentially full `.stdout/.stderr` capture) are restricted.

---

### Summary

sh2 provides a structured, test-validated shell language with explicit control flow, safer command execution, predictable error handling, and dual-target compilation to bash or POSIX sh.
