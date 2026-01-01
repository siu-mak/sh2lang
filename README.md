# sh2c (sh2)

`sh2c` is a prototype compiler for **sh2**, a small, structured shell language that compiles down to `bash` or POSIX `sh`.

This repository contains:
- the **compiler** (`sh2c`)
- the **language spec** (see `language.md`)
- the **descriptive grammar** (see `grammar.enbf.md`)

> Note: The grammar/spec describe the *target language shape*. Individual features may land incrementally.

---

## CLI

```text
Usage: sh2c [flags] <script.sh2> [flags]
Flags:
  --target <bash|posix>  Select output shell dialect (default: bash)
  --no-diagnostics       Disable error location reporting and traps
```

---

## Quick example

```sh2
import "stdlib.sh2"

func main() {
  let who = input("Name: ")
  print($"Hello {who}")

  if exists("README.md") && is_file("README.md") {
    run("ls", "-al", "README.md")
  } else {
    print_err("README.md not found")
  }
}
```

---

## Language at a glance

### Program structure
- `import "path"`
- `func name(a, b, c) { ... }`
- Optional free-standing statements (top-level execution), per grammar.

### Statements
- `let name = expr`
- `set <lvalue> = expr` where `lvalue` is `name` or `env.NAME`
- Command execution:
  - `run(expr, ...)`
  - `exec(expr, ...)`
- Output:
  - `print(expr)`
  - `print_err(expr)`
- Control flow:
  - `if expr { ... } elif expr { ... } else { ... }`
  - `while expr { ... }`
  - `for x in expr { ... }`
  - `for (k, v) in m { ... }`
  - `try { ... } catch { ... }`
  - `case expr { pattern => { ... } }`
- Scopes / modifiers:
  - `with env { KEY = expr, ... } { ... }`
  - `with cwd(expr) { ... }`
  - `with log(path, append=true|false) { ... }`
  - `with redirect { stdout: ..., stderr: ..., stdin: ... } { ... }`
- Process helpers:
  - `spawn { ... }` / `spawn statement`
  - `subshell { ... }`
  - `group { ... }`
  - pipelines: `run(...) | run(...) | run(...)`
  - statement chaining: `stmt && stmt` / `stmt || stmt`

### Expressions
- Literals: strings, numbers, booleans
- Operators (precedence high → low):
  - unary `!` `-`
  - `* / %`
  - `+ -`
  - concat `&`
  - compare `== != < <= > >=`
  - `&&`
  - `||`
- Calls: `name(expr, ...)`
- Capture: `capture(run(...) | run(...))`
- Lists: `[a, b, c]`
- Maps: `{ "k": v, "x": y }`
- Indexing: `a[i]`
- Field access: `obj.field`

### Builtins (selected)
- Args/process: `args`, `arg(n)`, `argc()`, `argv0()`, `status()`, `pid()`, `ppid()`, `uid()`, `pwd()`, `self_pid()`
- Env: `env(expr)` and `env.NAME`
- Filesystem tests: `exists(x)`, `is_dir(x)`, `is_file(x)`, `is_symlink(x)`, `is_exec(x)`, `is_readable(x)`, `is_writable(x)`, `is_non_empty(x)`
- Collections: `len(x)`, `count(x)`, `join(xs, sep)`
- Interactive: `input(prompt)`, `confirm(prompt)`
- Helpers: `bool_str(x)`

---

## Docs

- `language.md` — descriptive language reference
- `grammar.enbf.md` — EBNF grammar (descriptive)

