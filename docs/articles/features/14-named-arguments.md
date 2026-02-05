---
title: "Named arguments: making shell scripts self-documenting"
description: "Stop guessing what the third parameter means. Named arguments make sh2 scripts easier to read, safer to review, and easier to refactor."
---
<a href="https://github.com/siu-mak/sh2lang">
  <img src="../../images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# Named arguments: making shell scripts self-documenting

A deploy script had this function call:

```bash
backup_and_copy "$src" "$dst" true false 30
```

What does `true false 30` mean? Is `true` verbose mode? Is `30` a timeout or a retry count? Did someone swap `src` and `dst`?

Three months later, someone added a new parameter for "dry run mode" in position 4. Half the call sites broke silently—`30` was now being interpreted as a boolean.

This is the "flag soup" problem. Positional arguments work until you have more than two of them. Then they become a maintenance nightmare.

sh2 solves this with **named arguments** for builtins.

---

## Parameters vs arguments: a quick primer

**Parameters** are in the function definition:

```sh2
func greet(name, title) {
    print("Hello, " & title & " " & name)
}
```

**Arguments** are at the call site:

```sh2
# Positional arguments (matched by position)
greet("Alice", "Dr.")

# Named arguments (matched by name) — for builtins only
sudo("ls", user="root", n=true)
```

In sh2:
- **User-defined functions** accept positional arguments only.
- **Builtins** (`run`, `sudo`, `capture`, `confirm`, `sh`) support named arguments for options.

---

## Why named arguments matter

### 1. Self-documenting code

```sh2
# What does "true" mean?
confirm("Proceed?", true)

# Clear intent
confirm("Proceed?", default=true)
```

### 2. Order doesn't matter

```sh2
# All equivalent
sudo("ls", user="root", n=true)
sudo("ls", n=true, user="root")
sudo(n=true, "ls", user="root")
```

### 3. Adding options doesn't break calls

When a builtin gains a new option, existing calls keep working. You opt into new behavior by adding the named argument.

### 4. Compile-time validation

Typos and duplicates are caught before your script runs:

```
error: unknown sudo() option 'usr'; supported: user, n, k, prompt, E, env_keep, allow_fail
```

---

## Before/after examples

### 1. Confirmation with default

**Positional (ambiguous):**
```sh2
# What does "false" mean?
if confirm("Delete all?", false) { ... }
```

**Named (clear):**
```sh2
if confirm("Delete all?", default=false) { ... }
```

- `default=false` is unmistakable: in non-interactive mode, decline.

---

### 2. sudo with options

**Bash flag soup:**
```bash
sudo -n -u root -E apt-get update
```

**sh2 named:**
```sh2
sudo("apt-get", "update", n=true, user="root", E=true)
```

- Intent is obvious: non-interactive, as root, preserve environment.
- No need to remember `-n` vs `-u` vs `-E` order.

---

### 3. Function with multiple optional parameters

**Positional (confusing):**
```sh2
# retry(command, max_retries, timeout_sec, verbose)
retry("curl http://example.com", 5, 30, true)
# Is 5 the timeout? Is 30 the retries?
```

**If sh2 supported named args for user functions (it doesn't yet), you'd write:**
```
retry("curl ...", max_retries=5, timeout=30, verbose=true)
```

For now, the workaround is to use clear variable names:

```sh2
let max_retries = 5
let timeout = 30
let verbose = true
retry("curl http://example.com", max_retries, timeout, verbose)
```

---

### 4. The "swap bug"

**Dangerous:**
```sh2
# copy(src, dst) — but which is which?
copy(dst, src)  # Oops, silently backwards
```

**Safer pattern:**
```sh2
let src = "/data/important"
let dst = "/backup/important"
copy(src, dst)  # Variable names make intent clear
```

Named arguments would prevent this entirely. For user functions, explicit variable names are the current workaround.

---

### 5. Adding an option without breaking calls

**Scenario:** `capture(...)` gains a new `trim` option.

**Old call (still works):**
```sh2
let out = capture(run("whoami"))
```

**New call (opts in):**
```sh2
let out = capture(run("whoami"), trim=true)  # hypothetical
```

Existing code doesn't break because the new option is opt-in via its name.

---

### 6. Explicit allow_fail

**Implicit (confusing):**
```sh2
run("might-fail", true)  # What does "true" do?
```

**Named (clear):**
```sh2
run("might-fail", allow_fail=true)
```

- Reviewers immediately see: "this command is allowed to fail."

---

### 7. List options like env_keep

**Bash:**
```bash
sudo --preserve-env=PATH,HOME,HTTP_PROXY env
```

**sh2:**
```sh2
sudo("env", env_keep=["PATH", "HOME", "HTTP_PROXY"])
```

- The list syntax `[...]` makes it clear these are preserved variables.
- No comma-separated string parsing at runtime.

---

### 8. Mixed positional and named ordering

sh2 allows mixing positional command arguments with named options in any order (as of v0.1.1):

**All valid:**
```sh2
sudo("systemctl", "restart", "nginx", user="root", n=true)
sudo(user="root", "systemctl", "restart", "nginx", n=true)
sudo(n=true, "systemctl", user="root", "restart", "nginx")
```

- Positional arguments (the command words) are collected in order.
- Named arguments (the options) are collected by name.
- The compiler sorts it out.

---

## Rules and gotchas

### Which builtins support named arguments?

| Builtin | Supported named arguments |
|---------|---------------------------|
| `run(...)` | `allow_fail` |
| `sudo(...)` | `user`, `n`, `k`, `prompt`, `E`, `env_keep`, `allow_fail` |
| `capture(...)` | `allow_fail` |
| `confirm(...)` | `default` |
| `sh(...)` | `allow_fail` |

### User-defined functions: positional only

```sh2
func greet(name, title) { ... }

greet("Alice", "Dr.")       # ✅ Works
greet(name="Alice", ...)    # ❌ Compile error
```

Named arguments at call sites are reserved for builtins.

### Unknown names are errors

```sh2
sudo("ls", usr="root")
# Error: unknown sudo() option 'usr'; supported: user, n, k, prompt, E, env_keep, allow_fail
```

### Duplicates are errors

```sh2
confirm("go?", default=true, default=false)
# Error: default specified more than once
```

### Literal-only constraints

Some options require literal values (not variables):

| Builtin | Option | Required type |
|---------|--------|---------------|
| `sudo` | `user` | string literal |
| `sudo` | `prompt` | string literal |
| `sudo` | `n`, `k`, `E` | boolean literal |
| `sudo` | `env_keep` | list of string literals |
| `confirm` | `default` | boolean literal |

```sh2
let u = "root"
sudo("ls", user=u)
# Error: user must be a string literal
```

This ensures the generated shell command is predictable at compile time.

### Context restrictions

`allow_fail` has context-specific rules:

```sh2
# ✅ Statement form
sudo("ls", allow_fail=true)

# ❌ Expression form
let out = capture(sudo("ls", allow_fail=true))
# Error: allow_fail is only valid on statement-form sudo(...);
#        use capture(sudo(...), allow_fail=true) to allow failure during capture
```

---

## Compiler diagnostics

The compiler catches common mistakes:

### 1. Unknown option

```sh2
sudo("ls", xyz=true)
```

```
error: unknown sudo() option 'xyz'; supported: user, n, k, prompt, E, env_keep, allow_fail
```

### 2. Duplicate option

```sh2
sudo("ls", n=true, n=false)
```

```
error: n specified more than once
```

### 3. Wrong type

```sh2
sudo("ls", user=123)
```

```
error: user must be a string literal
```

### 4. allow_fail in wrong context

```sh2
let x = capture(sudo("ls", allow_fail=true))
```

```
error: allow_fail is only valid on statement-form sudo(...); use capture(sudo(...), allow_fail=true) to allow failure during capture
```

---

## When to use named arguments

| Situation | Recommendation |
|-----------|----------------|
| **Boolean options** | Always use named: `n=true`, not `true` |
| **Optional parameters** | Always use named: `default=false` |
| **Multiple options** | Named prevents ordering confusion |
| **Self-documenting code** | Named arguments are documentation |
| **Positional command args** | Keep positional: `"apt-get", "update"` |

**Rule of thumb:** If a reader would need to check the docs to understand an argument, use a named argument.

---

## Comparison table

| Pattern | Bash / positional style | Named-arg sh2 style | Why it's easier to review |
|---------|-------------------------|---------------------|---------------------------|
| Non-interactive mode | `sudo -n ls` | `sudo("ls", n=true)` | `n=true` is self-explanatory |
| Run as user | `sudo -u admin ls` | `sudo("ls", user="admin")` | `user="admin"` reads like English |
| Default prompt answer | (custom read logic) | `confirm("go?", default=false)` | Intent is in the code |
| Allow command failure | `cmd || true` | `run("cmd", allow_fail=true)` | Explicit and searchable |
| Preserve environment | `sudo -E cmd` | `sudo("cmd", E=true)` | No need to remember `-E` |
| Preserve specific vars | `sudo --preserve-env=X,Y` | `sudo("cmd", env_keep=["X","Y"])` | List syntax is clearer |
| Multiple options | `sudo -n -u root -E` | `sudo(..., n=true, user="root", E=true)` | Order doesn't matter |
| Capture with failure | `out=$(cmd) || true` | `capture(run(...), allow_fail=true)` | Explicit control flow |

---

## The philosophy

Shell scripts are often write-once, debug-forever. The original author knows what `-n -u root -E` means—but the next person doesn't.

Named arguments shift that knowledge from the author's head into the code itself. When you write `n=true, user="root", E=true`, the meaning is there for everyone to see.

That's the whole point: **code that explains itself.**

---

# Docs

The GitHub repo is here:  
**[https://github.com/siu-mak/sh2lang](https://github.com/siu-mak/sh2lang)**

## Further Documentation

- [`docs/language.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/language.md) — full language reference (syntax + semantics)
- [`docs/sh2do.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/sh2do.md) — sh2do CLI documentation
- `tests/` — fixtures and integration tests (acts as an executable spec)
