# The sh2 Language Reference

sh2 is a small, structured shell language designed to bring safety, clarity, and modern programming constructs to shell scripting. Scripts written in sh2 are compiled by **sh2c** into either **bash** (feature-rich) or **POSIX sh** (portable).

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

Inside `{ ... }` blocks, statements are separated by **newlines**. Semicolons (`;`) are not valid statement separators.

✅ Correct:
```sh2
func ok() {
  let a = "x"
  let b = "y"
}
```

❌ Incorrect:
```sh2
func bad() {
  let a = "x"; let b = "y"
}
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
- Concatenation uses `&`.

```sh2
let name = "Alice"
print("Hello, " & name)
print("Hello, $name")
```

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
2. `or`
3. `and`
4. comparisons: `== != < <= > >=`
5. `&` (string concatenation)
6. `+ -`
7. `* / %`
8. unary: `!` and unary `-`
9. postfix: calls `f(...)`, indexing `x[i]`, member access `x.field`

### 5.2 Logical operators: `and` / `or`

Use `and` / `or` (textual operators), not `&&` / `||`.

```sh2
if exists("a") and exists("b") {
  print("both")
}

if exists("a") or exists("b") {
  print("at least one")
}
```

### 5.3 Pipelines

Pipelines connect **stages** with `|`.

- They are broader than just `run(...) | run(...)`.
- Implementations include pipeline stages that may be blocks / statements in pipe contexts (as verified by pipe-block mixed-stage tests).

Common pattern:

```sh2
run("printf", "hello\n") | run("tee", "out.txt")
```

> Note: `print(...)` is a **statement**, not a pipeline stage. To “print and pipe”, use `run("printf", ...)`.

---

## 6. Command Execution

### 6.1 `run(...)` (expression)

`run(...)` executes an external command with safely separated arguments. It is an **expression**, so it can be used:

- as a standalone statement (expression statement), and
- inside boolean logic (`and`/`or`) and conditions.

```sh2
run("echo", "hello")

run("true") and run("echo", "only if true succeeded")
run("false") or run("echo", "only if false failed")
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

### 6.3 `sh("...")` (raw shell)

Executes a raw shell snippet. **It only accepts a string literal** (not concatenation / variables).

✅
```sh2
sh("echo hello")
```

`sh(...)` acts as a **probe**: it updates `status()` but does **not** automatically abort the script on failure (unlike `run(...)`).

❌
```sh2
let cmd = "echo hello"
sh(cmd)
sh("echo " & cmd)
```

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

---

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

### 9.3 `with redirect { ... } { ... }`

Redirection supports common patterns including append and stream merge (see `syntax_io` tests). Typical intent:

```sh2
# stdout to file
with redirect { stdout: "out.log" } { run("echo", "hi") }

# append
with redirect { stdout: file("out.log", append=true) } { run("echo", "more") }

# stderr → stdout
with redirect { stderr: stdout } { run("sh", "-c", "echo err 1>&2") }
```

> Exact redirect target surface forms are target-dependent; use the forms validated in your fixtures (`with_redirect_stdout_append`, `with_redirect_stderr_to_stdout`, etc.).

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

---

## 11. Targets and Portability

### `--target bash` (default)
Supports the full implemented feature set, including lists/maps, `with log`, interactive helpers (if enabled), and full `try_run` capture.

### `--target posix`
Prioritizes portability. Bash-only features (lists/maps, `with log`, and potentially full `.stdout/.stderr` capture) are restricted.

---

### Summary

sh2 provides a structured, test-validated shell language with explicit control flow, safer command execution, predictable error handling, and dual-target compilation to bash or POSIX sh.
