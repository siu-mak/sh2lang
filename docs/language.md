# sh2 language reference (devctl Steps 1–8)

This reference documents the language behavior currently relied upon by the incremental **devctl** development steps (Steps 1–8). It intentionally calls out **gotchas** where the syntax differs from “typical shell”.

---

## 1. Program structure

- A `.sh2` file contains **function definitions only**.
- **Top-level statements are not allowed.**
- The compiler-generated shell invokes `main()`.

```sh2
func main() {
  print("ok")
}
```

---

## 2. Statement separation

Inside blocks, statements are separated by **newlines**.

- `;` is **not** a valid statement separator inside `Ellipsis`.

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

## 3. Identifiers and reserved words

`env` is reserved for environment access.

❌

```sh2
func main() {
  let <reserved env> = "nope"
}
```

✅

```sh2
func main() {
  let env_name = "ok"
}
```

---

## 4. Core statements

### 4.1 `let`

Bind a new local variable:

```sh2
let name = "world"
```

### 4.2 `set`

Assign to an existing lvalue (including `env.NAME` if supported by the build):

```sh2
set x = "new"
```

### 4.3 `run(...)`

Run an external command. Arguments are expressions:

```sh2
run("printf", "hello %s\n", name)
```

Pipelines are supported between **run calls**:

```sh2
run("printf", "hello\n") | run("tee", "/tmp/out.txt")
```

### 4.4 `print(...)` and `print_err(...)`

- `print(expr)` prints to stdout.
- `print_err(expr)` prints to stderr.
- These are **statements**, not expressions, so they **cannot** be used as pipeline stages.

### 4.5 `sh("...")` (literal only)

`sh(...)` runs a shell snippet, but currently only accepts a **string literal**:

✅

```sh2
sh("echo hello")
```

❌

```sh2
let cmd = "echo hello"
sh(cmd)
sh("echo " & "hello")
```

Recommended alternatives:
- Prefer explicit `run(...)` calls.
- Keep `sh("...")` as a literal and pass dynamic values via environment or shell variables (depending on your target policy).

---

## 5. Control flow

### 5.1 `if` / `else`

```sh2
if status() == 0 {
  print("ok")
} else {
  print_err("failed")
}
```

### 5.2 `case` (arrow is `=>`)

`case` arms use `=>` (not `hyphen+greater-than`):

```sh2
case cmd {
  "env" => { print("env") }
  _ => { print_err("unknown"); exit(2) }
}
```

---

## 6. Expressions

### 6.1 Literals

- strings: `"text"`
- numbers: `0`, `1`, ...
- booleans: `true`, `false`

### 6.2 Variables

Use an identifier as an expression:

```sh2
let path = base & "/env.meta"
```

### 6.3 Operators

- Concatenation: `a & b`
- Comparisons: `== != < <= > >=`
- Boolean: `&& ||` (used in `if` conditions)

---

## 7. Environment access

Read environment variables with `env.NAME`:

```sh2
let base = env.HOME & "/sh2c/docker-rootless"
```

Example safe base dir pattern:

```sh2
let root = env.HOME & "/sh2c/docker-rootless"
```

---

## 8. Status tracking

### 8.1 `status()`

`status()` returns the last tracked exit status (0 = success). It updates after command-like actions such as:

- `run(...)`
- `sh("...")`
- builtins that lower to `test` (e.g. `exists/is_dir/is_file`)

---

## 9. Function return model (bash target)

At the language level, `return <expr>` is a **string-valued return**.

In the Bash target, a function “returns a string” by printing it to **stdout** and returning success, because Bash `return` only supports numeric statuses.

Practical pattern:

```sh2
func get_tag() {
  return "demo:latest"
}

func main() {
  let tag = capture(run("bash", "-lc", "true"))  # example capture pattern
}
```

(Use `capture(...)` for string returns; the exact capture syntax may depend on your current feature set.)
