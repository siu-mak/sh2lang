# sh2 Language Reference (descriptive)

This document describes the **sh2** language as specified by `grammar.enbf.md`.

sh2 is designed to be a small, structured language that compiles down to `bash` or POSIX `sh`.
Where shell syntax is historically error-prone (quoting, `[` spacing rules, etc.), sh2 favors explicit
syntax nodes like `run(...)`, `with redirect { ... }`, and `exists(...)`.

> The grammar is descriptive: it captures the intended surface syntax. Implementations may land features incrementally.

---

## 1. Program structure

A program consists of:
- zero or more `import` statements
- zero or more `func` definitions
- zero or more top-level statements

```sh2
import "foo.sh2"

func main() {
  print("hi")
}
```

---

## 2. Lexical forms

### Identifiers
`identifier = letter , { letter | digit | "_" }`

### Literals
- `number`: base-10 digits (integer)
- `bool_literal`: `true` / `false`
- `string_literal`: one of:
  - normal: `"..."` (escapes defined by implementation)
  - raw: `r"..."` (no escapes; terminates at the next `"` in the grammar)
  - interpolated: `$" ... {expr} ... "` (interleaves literal text with `{ expression }`)

---

## 3. Statements

### 3.1 Bindings and assignment

#### `let`
Creates a binding in the current scope.

```sh2
let x = "hello"
```

#### `set`
Assigns to an lvalue:

- a variable: `set x = expr`
- an environment slot: `set env.PATH = expr`

```sh2
set env.PATH = env("PATH") & ":/opt/bin"
```

### 3.2 Running commands

#### `run(...)`
Runs a command. Arguments are expressions.

```sh2
run("ls", "-al")
```

#### `exec(...)`
Replaces the current process with a command.

```sh2
exec("bash")
```

### 3.3 Printing
```sh2
print("message")
print_err("error")
```

### 3.4 Control flow

#### `if / elif / else`
```sh2
if exists("a") { print("yes") }
elif exists("b") { print("maybe") }
else { print("no") }
```

#### `while`
```sh2
while status() == 0 { run("true") }
```

#### `for` over an iterable expression
```sh2
for x in args { print(x) }
```

#### `for (k, v) in map`
```sh2
let m = { "a": 1, "b": 2 }
for (k, v) in m { print(k & "=" & v) }
```

#### `try / catch`
```sh2
try { run("false") }
catch { print_err("failed") }
```

#### `case`
```sh2
case env("MODE") {
  "dev" => { print("dev") }
  glob("prod*") => { print("prod") }
  _ => { print("default") }
}
```

### 3.5 Scopes and modifiers (`with`)

`with` introduces a block whose execution is modified.

#### `with env { ... }`
```sh2
with env { HOME = "/tmp", MODE = "dev" } {
  run("env")
}
```

#### `with cwd(expr)`
```sh2
with cwd("/var/log") { run("ls") }
```

#### `with log(path, append=...)`
A logging scope (fan-out/tee behavior is target-defined).

```sh2
with log("build.log", append=true) { run("make") }
```

#### `with redirect { ... }`
Redirects standard streams.

```sh2
with redirect { stdout: file("out.txt"), stderr: stdout } {
  run("echo", "hello")
}
```

Redirect targets per grammar:
- `stdout`, `stderr`
- `file(path, append=true|false)`
- `heredoc("...")`

### 3.6 Process helpers

- `spawn { ... }` or `spawn statement`
- `subshell { ... }`
- `group { ... }`
- `wait` or `wait(expr)`
- `cd(expr)`
- `sh("...")` or `sh { ... }`
- `source(expr)`

### 3.7 Statement chaining

- `stmt && stmt`
- `stmt || stmt`

These are statement-level forms in the grammar, distinct from boolean operators in expressions.

---

## 4. Expressions

### 4.1 Operator precedence

From low to high:
1. `||`
2. `&&`
3. comparison: `== != < <= > >=`
4. concat: `&`
5. arithmetic: `+ -`
6. arithmetic: `* / %`
7. unary: `! -`

### 4.2 Primary expressions

- literals
- identifier
- parenthesized `(expr)`
- `call_expr`: `name(expr, ...)`
- `capture(expr)` where the inner is `run_call` or a pipeline
- list: `[a, b, c]`
- map: `{ "k": v, ... }`
- indexing: `x[i]`
- field access: `x.name`
- builtins (below)

---

## 5. Builtins

### Args / process
- `args` (list of args)
- `arg(n)` (single arg)
- `argc()`, `argv0()`
- `status()`
- `pid()`, `ppid()`, `uid()`, `pwd()`, `self_pid()`

### Environment
- `env(expr)`
- `env.NAME`

### Filesystem predicates
- `exists(x)`
- `is_dir(x)`
- `is_file(x)`
- `is_symlink(x)`
- `is_exec(x)`
- `is_readable(x)`
- `is_writable(x)`
- `is_non_empty(x)`

### Collection helpers
- `len(x)`
- `count(x)`
- `join(xs, sep)`
- `lines(str)`


### Interactive
- `input(prompt)`
- `confirm(prompt)`

### Misc
- `bool_str(x)` (string representation for booleans)

---

## 6. Notes on targets

sh2 compiles to either:
- `--target bash`
- `--target posix`

Some features (e.g., tee/logging, regex, associative maps) may have target-specific behavior.
