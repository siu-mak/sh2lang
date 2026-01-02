# The sh2 Language Reference

sh2 is a small, structured shell language compiled by **sh2c** into either **bash** or **POSIX sh**.
It aims to make shell intent explicit (control flow, pipelines, env/cwd scoping, status handling) and reduce common shell footguns.

This document reflects the **implemented** language behavior as of the incremental `devctl` development steps (Steps 1–8) plus the implemented
features you listed (run behavior controls, modules, try_run, logging, interactive, lists/maps, and stdlib helpers).

---

## 1. Program structure

### 1.1 Imports

You can import modules at the top of a file:

```sh2
import "path/to/module.sh2"
import "lib/envfile.sh2"
```

Notes:
- Imports must appear **before** function definitions.
- The loader resolves imports recursively, detects cycles, and supports common dependency shapes (including diamond dependencies).
- Imported functions share a single namespace (name collisions are errors).

### 1.2 Functions

Functions are defined with `func`:

```sh2
func greet(name) {
  print("hello " & name)
}
```

### 1.3 Entry point

Programs are **function definitions only** (no top-level statements). The generated shell invokes `main()` as the entry point:

```sh2
func main() {
  print("ok")
}
```

---

## 2. Statement separation (important)

Inside `{ ... }` blocks, statements are separated by **newlines**.

- Semicolons `;` are **not** valid statement separators inside blocks.

✅

```sh2
func main() {
  print("a")
  print("b")
}
```

❌

```sh2
func main() {
  print("a"); print("b")
}
```

---

## 3. Identifiers and reserved keywords

### 3.1 `env` is reserved

`env` is reserved for environment access (e.g., `env.HOME`) and cannot be used as a variable name.

Use a different identifier:

```sh2
func main() {
  let env_name = "ok"
  print(env_name)
}
```

---

## 4. Data types and literals

### 4.1 Strings

```sh2
let s = "hello"
let p = env.HOME & "/sh2c/docker-rootless"
```

Concatenation uses `&`.

### 4.2 Numbers

```sh2
let n = 42
```

### 4.3 Booleans

```sh2
let ok = true
```

### 4.4 Lists (Bash only)

Lists are written with `[...]`.

```sh2
let xs = ["a", "b", "c"]
print(xs[0])
```

- **Bash target:** supported
- **POSIX target:** not supported (errors/panics due to lack of arrays)

### 4.5 Maps (Bash only)

Maps are written with `{ "k": v, ... }`.

```sh2
let m = { "user": "herbert", "role": "admin" }
print(m["user"])
```

- **Bash target:** supported (associative arrays / structured lowering)
- **POSIX target:** not supported (errors/panics)

---

## 5. Variables and assignment

### 5.1 `let`

Declare a new local variable:

```sh2
let name = "world"
```

### 5.2 `set`

Assign to an existing lvalue:

```sh2
func main() {
  let x = 1
  set x = x + 1
  print("x=" & x)
}
```

### 5.3 Environment access

Read env vars with dot access:

```sh2
let base = env.HOME & "/sh2c/docker-rootless"
```

Dynamic env lookup is also available via `env("NAME")` when you need the variable name to be computed:

```sh2
let key = "HOME"
let home = env(key)
```

---

## 6. Running commands

### 6.1 `run(...)`

Run an external command:

```sh2
run("printf", "hello %s\n", "world")
```

### 6.2 `run(..., allow_fail=true)`

`allow_fail=true` prevents the compiler from emitting failure-propagation checks for **that specific run**.
The command may fail without aborting the script, and `status()` is still updated.

```sh2
func main() {
  run("false", allow_fail=true)
  print("still running, status=" & status())
}
```

### 6.3 Pipelines

Pipelines connect `run(...)` stages:

```sh2
run("printf", "hello\n") | run("tee", "/tmp/out.txt")
```

Notes:
- `tee` echoes to stdout while writing to the file.
- Pipeline error propagation is target-dependent (bash can be pipefail-like; POSIX may be limited depending on implementation).

### 6.4 `exec(...)`

Replace the current process with the given command (no continuation):

```sh2
func main() {
  exec("bash", "-lc", "echo replaced")
}
```

### 6.5 `sh("...")` (literal-only)

`sh(...)` runs a shell snippet, but currently it accepts **only a string literal**:

✅

```sh2
sh("echo hello")
```

❌

```sh2
func main() {
  let cmd = "echo hello"
  # sh(cmd)              # not allowed
  # sh("echo " & "hi")   # not allowed
}
```

Recommended alternatives:
- Prefer structured `run(...)` calls.
- If you truly need `sh`, keep it literal and pass dynamic values via files/env or by composing `run(...)` plumbing.

---

## 7. Printing and piping

### 7.1 `print(...)` and `print_err(...)`

`print(expr)` writes to stdout; `print_err(expr)` writes to stderr.

These are **statements**, not pipeline stages, so they cannot be piped.

Instead of piping `print`, use `printf | tee`:

```sh2
func main() {
  let path = "/tmp/out.txt"
  run("printf", "hello %s\n", "world") | run("tee", path)
}
```

---

## 8. Control flow

### 8.1 `if` / `elif` / `else`

```sh2
func main() {
  run("true")
  if status() == 0 {
    print("ok")
  } else {
    print_err("failed")
  }
}
```

### 8.2 `case` (arm arrow is `=>`)

Case arms use `=>` (not `->`).

```sh2
func main() {
  let cmd = "env"
  case cmd {
    "env" => { print("env") }
    _ => { print_err("unknown"); exit(2) }
  }
}
```

### 8.3 Loops

#### `while`

```sh2
func main() {
  let i = 0
  while i < 3 {
    print("i=" & i)
    set i = i + 1
  }
}
```

#### `for` over lists (Bash only)

```sh2
func main() {
  let xs = ["a", "b", "c"]
  for x in xs {
    print(x)
  }
}
```

#### `for (k, v)` over maps (Bash only)

```sh2
func main() {
  let m = { "a": "1", "b": "2" }
  for (k, v) in m {
    print(k & "=" & v)
  }
}
```

POSIX notes:
- list/map iteration is restricted because lists/maps are not supported on `--target posix`.

### 8.4 `try { ... } catch { ... }`

`try/catch` is implemented. If a command in `try` fails under the language’s error model, the `catch` block runs.
`status()` in the catch block reflects the failure.

```sh2
func main() {
  try {
    run("false")
    print("unreachable")
  } catch {
    print_err("failed, status=" & status())
  }
}
```

---

## 9. Status tracking

### 9.1 `status()`

`status()` returns the last tracked exit status and updates after:
- `run(...)` (including `allow_fail=true`)
- `try_run(...)`
- `sh("...")`
- filesystem predicates (e.g. `exists/is_dir/is_file`)

---

## 10. `try_run(...) -> RunResult`

`try_run(...)` is implemented and **does not abort on failure**. It captures:
- `stdout`
- `stderr`
- exit `status`

The result supports field access:
- `r.status`
- `r.stdout`
- `r.stderr`

```sh2
func main() {
  let r = try_run("sh", "-lc", "echo out; echo err 1>&2; exit 7")

  print("status=" & r.status)
  print("stdout=" & r.stdout)
  print_err("stderr=" & r.stderr)
}
```

Fixtures confirming behavior include:
- `try_run_success.sh2`
- `try_run_fields.sh2`

---

## 11. Scoped logging: `with log(...) { ... }` (Bash only)

`with log(path, append=true|false) { ... }` fans out output to both console and a log file.
Implementation uses Bash process substitution (e.g., `exec > >(tee ...)` patterns).

```sh2
func main() {
  with log("/tmp/devctl.log", append=true) {
    print("hello")
    run("printf", "tag=%s\n", "demo:latest")
  }
}
```

Target notes:
- **Bash target:** supported
- **POSIX target:** errors/panics (no process substitution)

---

## 12. Interactive primitives (Bash only)

### 12.1 `input(prompt)`

```sh2
func main() {
  let name = input("Name: ")
  print("hi " & name)
}
```

### 12.2 `confirm(prompt)`

```sh2
func main() {
  if confirm("Proceed?") {
    print("ok")
  } else {
    print_err("cancelled")
    exit(1)
  }
}
```

Target notes:
- **Bash target:** supported
- **POSIX target:** errors/panics

---

## 13. Builtins and helpers

### 13.1 Filesystem predicates

Return booleans:

- `exists(x)`
- `is_dir(x)`
- `is_file(x)`
- `is_symlink(x)`
- `is_exec(x)`
- `is_readable(x)`
- `is_writable(x)`
- `is_non_empty(x)`

### 13.2 Regex helper: `matches(text, regex)`

Implemented (see tests like `syntax_matches.rs` and fixtures like `matches_basic.sh2`).

```sh2
func main() {
  if matches("12345", "^[0-9]+$") {
    print("digits")
  }
}
```

### 13.3 Arg parsing: `parse_args(...)`

`parse_args` is implemented (see `syntax_parse_args.rs`). Document your project’s exact return shape (flags/positionals) in this section if needed.

### 13.4 Envfile helpers

Implemented:
- `load_envfile(path)`
- `save_envfile(path, data)`

### 13.5 JSON helper

Implemented:
- `json_kv(...)` for emitting JSON objects from key-value pairs.

### 13.6 String and file helpers

Implemented helpers include:
- `split`, `join`, `lines`
- `trim`, `replace`
- `read_file`, `write_file`

---

## 14. Target differences and portability

### 14.1 `--target bash` (default)

Supports Bash-only features:
- lists / maps and their iteration
- `with log(...)`
- `input(...)`, `confirm(...)`
- `try_run(...) -> RunResult` with `.stdout/.stderr/.status`

### 14.2 `--target posix`

Strictly portable output, with some features disabled.

Currently **not supported** on POSIX target (errors/panics):
- lists and maps
- `with log(...)`
- `input(...)`, `confirm(...)`

---

## Appendix A: Known-good cookbook patterns

### A.1 Root check (portable pattern)

```sh2
func main() {
  sh("test \"$(id -u)\" -eq 0")
  if status() != 0 {
    print_err("must run as root")
    exit(1)
  }
}
```

### A.2 Write file content (simple)

```sh2
func main() {
  let path = env.HOME & "/env.meta"
  run("printf", "IMAGE_TAG=%s\n", "demo:latest") | run("tee", path)
}
```
