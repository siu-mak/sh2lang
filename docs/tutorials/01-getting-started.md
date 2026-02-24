---
title: "Getting Started with sh2"
description: "A hands-on tutorial teaching you to write, compile, and run your first sh2 scripts."
---
<a href="https://github.com/siu-mak/sh2lang">
  <img src="../images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# Getting Started with sh2

Welcome! This tutorial will teach you how to use **sh2c** (the compiler) and **sh2do** (the snippet runner) to write safer, more readable shell scripts.

**What you'll learn:**
- What sh2 is and why it exists
- How to install/build from source
- Writing and compiling your first script
- Running quick snippets with `sh2do`
- The safety model: no implicit expansion
- Running commands, capturing output, and handling errors
- Pipelines, working directories, and file I/O
- New features in v0.1.1 (`confirm`, `sudo`, semicolons)
- When to use sh2 vs Bash

**Prerequisites:** Basic familiarity with Bash (running commands, piping, variables).

---

## 1. What is sh2?

**sh2** is a structured shell language that compiles to Bash or POSIX shell scripts.

Think of it as "safer shell glue":
- You write `.sh2` source files with explicit syntax
- The compiler (`sh2c`) outputs a regular `.sh` script
- The output runs anywhere Bash or POSIX sh runs

**Two tools:**
- **`sh2c`** — The compiler. Takes a `.sh2` file and outputs a `.sh` script.
- **`sh2do`** — A snippet runner. Compiles and runs sh2 code in one step.

---

## 2. Installation (Build from source)

The primary way to install sh2lang is to build from source. This works on Linux, macOS, and Windows (via WSL), and ensures you have the latest version.

### Prerequisites

You need a Rust toolchain. If you don't have one:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build and Install

```bash
git clone https://github.com/siu-mak/sh2lang.git
cd sh2lang
cargo build --workspace --release
```

Verify the build:

```bash
./target/release/sh2c --help
./target/release/sh2do --help
```

Optionally install to PATH:

```bash
cargo install --path sh2c --locked
cargo install --path sh2do --locked
```

> **Ubuntu 22.04 (jammy) users:** You can also install via APT. See [Installation](../../README.md#installation-apt--ubuntu-2204--jammy) in the README.

----

## 3. Your First sh2 Script

### Create `hello.sh2`

```sh2
func main() {
    print("Hello from sh2!")
}
```

### Compile it

```bash
sh2c hello.sh2 -o hello.sh
```

This creates `hello.sh` (and makes it executable).

### Run it

```bash
./hello.sh
```

Output:

```text
Hello from sh2!
```

**Try this:** Change `"Hello from sh2!"` to your own message, recompile, and run again.

---

## 4. Your First sh2do Snippet

`sh2do` is great for quick experiments. It compiles and runs in one step.

### Inline snippet

```bash
sh2do 'run("echo", "hello from sh2do")'
```

Output:

```text
hello from sh2do
```

### Multiple statements (use semicolons)

```bash
sh2do 'run("echo", "one"); run("echo", "two"); run("echo", "three")'
```

### From stdin

```bash
echo 'run("echo", "piped!")' | sh2do -
```

### Pass arguments

```bash
sh2do 'run("echo", "Hello, " & arg(1))' -- Alice
```

Output:

```text
Hello, Alice
```

**Try this:** Run `sh2do 'run("echo", arg(1) & " " & arg(2))' -- Hello World`

---

## 5. The Big Safety Rule: No Implicit Expansion

This is sh2's core safety feature. Strings are **strict literals**. No automatic:
- Word splitting on spaces
- Glob expansion (`*`, `?`, `[...]`)
- Tilde expansion (`~`)
- Variable expansion (`$FOO`, `${FOO}`)

### Globbing doesn't happen

**Try this:**

```bash
sh2do 'run("echo", "*")'
```

Output:

```text
*
```

In Bash, `echo *` would list all files. In sh2, `"*"` is a literal asterisk.

### Spaces don't split arguments

```bash
sh2do 'run("echo", "hello world")'
```

Output:

```text
hello world
```

The string `"hello world"` stays as one argument, not two.

### Variables don't expand

```bash
sh2do 'let x = "$HOME"; print(x)'
```

Output:

```text
$HOME
```

The literal `$HOME` is printed, not your home directory path.

**To get environment values, use `env.`:**

```bash
sh2do 'print(env.HOME)'
```

### Learn more

See [No Implicit Expansion](../articles/features/13-no-implicit-expansion.md) for the full explanation of this design.

---

## 6. Running Commands

### Basic execution

```sh2
func main() {
    run("echo", "hello")
    run("ls", "-la", "/tmp")
}
```

Arguments are **always** passed safely. No quoting gymnastics needed.

### Capturing output

Use `capture(...)` to get stdout as a string:

```sh2
func main() {
    let who = capture(run("whoami"))
    
    print("You are: " & who)
}
```

### Checking exit status

By default, if a command fails, the script stops (fail-fast). To allow a command to fail and check the result:

```sh2
func main() {
    run("grep", "pattern", "missing.txt", allow_fail=true)
    
    print("Exit code: " & status())
    
    if status() != 0 {
        print("File not found or no match")
    }
}
```

### Capture with allow_fail

```sh2
func main() {
    let output = capture(run("cat", "/etc/shadow"), allow_fail=true)
    
    if status() != 0 {
        print_err("Could not read file")
    }
}
```

### Learn more

See [Error Handling](../articles/features/15-error-handling.md) for patterns like retry loops, cleanup, and collecting partial failures.

---

## 7. Pipelines

sh2 supports structured pipelines with `|`:

```sh2
func main() {
    run("printf", "a\nb\nc\n") | run("wc", "-l")
}
```

### Capturing pipeline output

```sh2
func main() {
    let count = capture(run("ls", "-1") | run("wc", "-l"))
    
    print("Files: " & trim(count))
}
```

### Multi-stage pipelines

You can chain multiple stages:

```sh2
func main() {
    let result = capture(
        run("find", ".", "-name", "*.log", "-print")
        | run("xargs", "grep", "ERROR")
        | run("wc", "-l"),
        allow_fail=true
    )
}
```

### When to use `sh(...)`

For Bash-only features like **process substitution** or **job control**, sh2 provides an escape hatch:

```sh2
func main() {
    # sh(...) because: process substitution <(...)
    sh("diff <(sort file1.txt) <(sort file2.txt)")
}
```

Inside `sh("...")`, you're back in shell-land. Globs expand. Variables expand. Use sparingly, and always add a comment explaining why.

---

## 8. Working Directory and Files

### Scoped `cwd`

Change the working directory for a block only:

```sh2
func main() {
    with cwd("/tmp") {
        run("pwd")      # prints /tmp
        run("touch", "test.txt")
    }
    run("pwd")          # back to original directory
}
```

**Important:** `cwd(...)` requires a **string literal** path. Computed paths (variables) are not allowed. This is a deliberate safety restriction.

> **Dynamic cwd is not supported.** If you absolutely must use a dynamic path, you can use `sh($"cd {dir} && ls")`, but this is **injection-prone** if `dir` comes from untrusted input. Prefer restructuring your script to use literal paths where possible.

### Reading files

```sh2
func main() {
    let content = read_file("config.txt")
    print(content)
}
```

### Iterating lines

```sh2
func main() {
    let text = read_file("names.txt")
    
    for name in lines(text) {
        print("Hello, " & name)
    }
}
```

### Writing files

```sh2
func main() {
    write_file("output.txt", "hello\n")
    
    append_file("log.txt", "entry\n")
}
```

> **Note:** These functions read the entire file into memory (or return a string). For streaming large files line-by-line, pipe the file into a sh2 script and use the `stdin_lines()` iterator.

---

## 9. Recent Features

### `confirm(...)` — Interactive prompts

Ask for yes/no confirmation:

```sh2
func main() {
    if confirm("Delete all .bak files?", default=false) {
        # Use find to handle glob patterns (sh2 doesn't expand globs)
        run("find", ".", "-name", "*.bak", "-delete")
    }
}
```

### `confirm` with default for non-interactive (CI) environments

```sh2
func main() {
    if confirm("Proceed with deployment?", default=false) {
        run("deploy.sh")
    } else {
        print("Aborted.")
    }
}
```

**Environment overrides:**
- `SH2_YES=1` — Always return true
- `SH2_NO=1` — Always return false

### `sudo(...)` — Privileged execution

Structured wrapper with named options:

```sh2
func main() {
    sudo("apt-get", "update")
    sudo("systemctl", "restart", "nginx", user="root")
    sudo("ls", "/root", n=true)
}
```

**Supported options:** `user`, `n`, `k`, `prompt`, `E`, `env_keep`, `allow_fail`

Named options are self-documenting. No more decoding `-u root -n -E`.

### Semicolons

Multiple statements on one line:

```sh2
func main() {
    print("one"); print("two"); print("three")
}
```

### Learn more

See the [v0.1.2 Release Notes](../releases/v0.1.2.md) for full details.

---

## 10. When to Use sh2 vs Bash

| Use sh2 when... | Use Bash when... |
|-----------------|------------------|
| Script needs code review | Quick interactive exploration |
| Script runs in CI/CD | Dense text pipelines (`grep \| awk \| sort`) |
| Script uses `sudo`, `rm`, `systemctl` | Process substitution (`<(...)`) |
| Script will be shared/maintained | Interactive job control (`fg`, `bg`) |
| You want fail-fast error handling | One-off throwaway commands |

**The escape hatch:** When you genuinely need shell syntax, use `sh("...")`. But add a comment explaining why.

---

## 11. Mini Project: Backup Cleanup Tool

Let's build a real tool that:
1. Takes a directory argument
2. Finds old `.bak` files (older than 30 days)
3. Prints a count
4. Asks for confirmation
5. Deletes safely with logging

### Create `cleanup-backups.sh2`

```sh2
# tools/cleanup-backups.sh2
# Deletes backup files older than 30 days.

func usage() {
    print("Usage: cleanup-backups.sh <directory>")
    print("")
    print("Options:")
    print("  --help    Show this message")
    print("")
    print("Environment:")
    print("  SH2_YES=1    Skip confirmation")
}

func main() {
    # Argument parsing
    if argc() < 1 {
        usage()
        print_err("Error: missing directory argument")
        return 1
    }
    
    let dir = arg(1)
    
    if arg(1) == "--help" {
        usage()
        return 0
    }
    
    # Validate directory
    if !is_dir(dir) {
        print_err($"Error: '{dir}' is not a directory")
        return 1
    }
    
    # Find files to delete (run() handles arguments safely)
    let files = capture(
        run("find", dir, "-name", "*.bak", "-mtime", "+30", "-print"),
        allow_fail=true
    )
    if status() != 0 {
        print_err("Error: find command failed")
        return 1
    }
    
    # Count files
    let count = 0
    for f in lines(files) {
        if f != "" {
            set count = count + 1
        }
    }
    
    if count == 0 {
        print("No backup files older than 30 days found.")
        return 0
    }
    
    print($"Found {count} backup file(s).")
    
    # Confirm before deletion
    if !confirm($"Delete {count} file(s)?", default=false) {
        print("Aborted.")
        return 0
    }
    
    # Perform deletion with logging
    with redirect { stdout: [file("cleanup.log", append=true), inherit_stdout()] } {
        for f in lines(files) {
            if f != "" {
                run("rm", "--", f, allow_fail=true)
                if status() == 0 {
                    print($"Deleted: {f}")
                } else {
                    print_err($"Warning: could not delete {f}")
                }
            }
        }
    }
    
    print("Done. See cleanup.log for details.")
}
```

### Compile and run

```bash
sh2c cleanup-backups.sh2 -o cleanup-backups.sh
./cleanup-backups.sh /var/backups
```

### What this demonstrates

- **Argument parsing** with `argc()` and `arg(n)`
- **Input validation** with `is_dir()`
- **Error handling** with `allow_fail=true` and `status()`
- **Confirmation** with `confirm(..., default=false)`
- **Safe deletion** with `run("rm", "--", path, ...)`
- **Logging** with `with redirect { ... }`
- **Interpolation** with `$"...{var}..."`

---

## Next Steps

You now know the basics! Here's where to go next:

### Reference docs
- [Language Reference](../language.md) — Full syntax and semantics
- [sh2do Documentation](../sh2do.md) — Snippet runner details

### Key feature articles
- [No Implicit Expansion](../articles/features/13-no-implicit-expansion.md) — Why strings are strict literals
- [Error Handling](../articles/features/15-error-handling.md) — Patterns for `allow_fail`, retry, cleanup
- [sudo Builtin](../articles/features/11-sudo-builtin.md) — Named options for privileged execution
- [confirm Helper](../articles/features/12-confirm-helper.md) — Interactive prompts and CI behavior

### Case studies
- [The Hidden Tax of Reviewing Bash](../articles/case-studies/21-hidden-tax-reviewing-bash.md)
- [The Dollar Expansion Bug](../articles/case-studies/22-dollar-expansion-bug.md)

### Release notes
- [v0.1.2 Release Notes](../releases/v0.1.2.md) — Job control, iterators, `which`
- [v0.1.1 Release Notes](../releases/v0.1.1.md) — `sudo`, `confirm`, semicolons
- [v0.1.0 Release Notes](../releases/v0.1.0.md) — Initial release

---

Happy scripting! 🎉
