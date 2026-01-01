# sh2c / sh2

**sh2** is a small, compiled “semantic shell language”. **sh2c** compiles `*.sh2` source into a shell script (Bash by default).

This documentation reflects the language + CLI behavior observed through the incremental **devctl Steps 1–8**.

---

## Quick start

Compile to Bash on stdout:

```bash
sh2c --target bash ./devctl.sh2
```

Write to a file (auto `chmod +x` by default):

```bash
sh2c --target bash ./devctl.sh2 -o ./devctl
./devctl env list
```

Check-only (no output):

```bash
sh2c --check ./devctl.sh2
```

---

## CLI

This is the current `--help` output:

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

### Output, shebangs, and executable bit

- `--target bash` emits Bash-oriented shell output. In devctl Steps 1–8, generated Bash output includes a shebang:
  - `#!/usr/bin/env bash`
- `--target posix` emits a POSIX-sh compatible script. (Shebang behavior may differ by target; if you need a specific interpreter line, prefer `--target bash` or add the shebang you want at the top of the emitted script.)

When you use `-o/--out`, the compiler writes to that file. By default, the output file is made executable (`--chmod-x` default). Use `--no-chmod-x` to disable that behavior.

---

## Language at a glance

> **Important:** sh2 files contain **function definitions only**. Top-level statements are not allowed.
> The compiler-generated shell invokes `main()`.

A minimal program:

```sh2
func main() {
  print("hello")
}
```

---

## Cookbook patterns (devctl-style)

### 1) `case` uses `=>` (not `hyphen+greater-than`)

```sh2
func main() {
  let cmd = arg(0)

  case cmd {
    "env" => {
      print("env subcommand")
    }
    _ => {
      print_err("unknown command")
      exit(2)
    }
  }
}
```

### 2) Newlines separate statements (no `;` in blocks)

✅ valid:

```sh2
func main() {
  print("one")
  print("two")
}
```

❌ invalid:

```sh2
func main() {
  print("one"); print("two")
}
```

### 3) `env` is reserved

❌ invalid:

```sh2
func main() {
  let <reserved env> = "oops"
}
```

✅ fix:

```sh2
func main() {
  let env_name = "ok"
}
```

### 4) `env.NAME` for environment variables

```sh2
func main() {
  let base = env.HOME & "/sh2c/docker-rootless"
  run("mkdir", "-p", base)
}
```

### 5) Root check pattern (portable)

Avoid `uid()` on POSIX. Use `sh("...")` + `status()`:

```sh2
func main() {
  sh("test \"$(id -u)\" -eq 0")
  if status() != 0 {
    print_err("must run as root")
    exit(1)
  }
}
```

### 6) “Write file” without a `write_file` builtin

`print(...)` is a **statement**, not a pipeline stage, so it can’t be used with `|`.

Instead, use external tools:

```sh2
func main() {
  let path = env.HOME & "/env.meta"

  run("printf", "IMAGE_TAG=%s\n", "demo:latest")
    | run("tee", path)
}
```

Note: `tee` writes to the file **and** echoes to stdout. That is normal `tee` behavior.

---

## `sh("...")` (literal-only)

Currently, `sh(...)` accepts **only a string literal** (not a concatenated expression).

❌ fails today:

```sh2
func main() {
  let cmd = "echo hi"
  sh(cmd)          # not allowed
  sh("echo " & "hi")  # not allowed
}
```

✅ recommended alternatives:

- Put the dynamic parts into normal shell commands via `run(...)`.
- If you truly need an inline shell snippet, keep it a literal and reference variables (via the shell), or refactor into explicit `run(...)` calls.

---

## Status and `return`

- `status()` returns the last command/builtin exit status tracked by the runtime (used by `if`, `case` fallthrough patterns, and explicit checks).
- `return <expr>` in sh2 is *string-valued* at the language level; in Bash codegen, this is implemented by printing the return value to **stdout** and returning success (because `return "string"` is not a Bash thing). Use `capture(...)` when you need the returned string.

See **language.md** for details.
