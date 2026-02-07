---
title: "Files and directories in sh2: safe loops, find, cwd, and redirects"
description: "Practical patterns for working with paths and files in sh2 without quoting bugs: find, iteration, cwd blocks, and logging."
---
<a href="https://github.com/siu-mak/sh2lang">
  <img src="../images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# Files and Directories in sh2

This tutorial covers practical patterns for working with files and directories in sh2. You'll learn how to list, find, copy, and delete files safely—without the quoting bugs that plague Bash scripts.

**What you'll learn:**
- Why file operations are error-prone in Bash
- How sh2's "no implicit expansion" rule prevents common bugs
- Safe patterns for find, iteration, and deletion
- Using `with cwd(...)` for scoped directory changes
- Logging with `with redirect { ... }`

**Prerequisites:** Complete [Tutorial 01: Getting Started](01-getting-started.md) and [Tutorial 03: Error Handling](03-error-handling-and-status.md).

---

## 1. Why Files Are Hard in Bash

File operations in Bash are a minefield. The culprits:

- **Word splitting**: Spaces in filenames become multiple arguments
- **Globbing**: `*` and `?` expand unexpectedly
- **Unquoted variables**: `rm -rf $dir` can delete your entire system

### A tiny Bash script that breaks on spaces

```bash
# ❌ Dangerous Bash script
dir="/tmp/my project"
cd $dir              # Fails: tries to cd to "/tmp/my" then "project"
rm -rf $dir/*.bak    # Fails: word splits the path
```

If `$dir` isn't quoted, Bash splits it on spaces. If `$dir` is empty, `rm -rf /*.bak` runs on root.

Even experienced developers get bitten:

```bash
# ❌ Still dangerous
for f in $(find . -name "*.txt"); do
    rm "$f"          # Breaks on filenames with spaces
done
```

The `$(...)` command substitution splits output on whitespace. A file named `my file.txt` becomes two arguments: `my` and `file.txt`.

---

## 2. The sh2 Rule That Changes Everything

**sh2 performs no implicit expansion.** Strings are strict literals.

This single rule prevents most file-handling bugs:

```sh2
func main() {
    run("echo", "*")          # Prints: *  (literal asterisk)
    run("echo", "hello world") # One argument, not two
}
```

Test it:

```bash
sh2do 'run("echo", "*")'
```

Output:

```text
*
```

In Bash, `echo *` would list all files. In sh2, `"*"` is just an asterisk.

### Paths with spaces remain one argument

```sh2
func main() {
    let path = "/tmp/my project/data file.txt"
    run("cat", path)  # Always one argument, no quoting needed
}
```

You never need to think about quoting. Every argument is safely separated.

---

## 3. Listing Files Safely

### Simple listing with `run("ls", ...)`

```sh2
func main() {
    run("ls", "-la", "/tmp")
}
```

### Safe flat iteration with `glob()` (Bash only)

For simple file matching in the current directory (flat, non-recursive), use `glob()`:

```sh2
func main() {
    # glob() returns a sorted list of matches
    for f in glob("*.txt") {
        print("Found text file: " & f)
    }
    
    # Check if empty (no matches = empty list)
    let logs = glob("*.log")
    if count(logs) == 0 {
        print("No logs found")
    }
}
```

### Capturing file lists with `find`

For programmatic use, `find` with `capture` and `lines`:

```sh2
func main() {
    let files = capture(run("find", ".", "-type", "f", "-name", "*.txt"))
    
    for f in lines(files) {
        if f != "" {
            print("Found: " & f)
        }
    }
}
```

**Key points:**
- `capture(run("find", ...))` returns newline-separated output
- `lines(...)` splits into an iterable list
- The `if f != ""` check handles trailing newlines

### Honest limitation: newlines in filenames

If a filename contains a literal newline character (rare but possible), the `lines()` approach breaks. For maximum safety, use `find -print0` with a shell helper—but this is almost never needed in practice.

---

## 4. Three Common Tasks (Mini-Tools)

### Task A: Print the 10 largest files under a directory

```sh2
# largest-files.sh2
# Prints the 10 largest files in a directory.

func usage() {
    print("Usage: largest-files.sh <directory>")
}

func main() {
    if argc() < 1 {
        usage()
        return 1
    }
    
    let dir = arg(1)
    
    if !is_dir(dir) {
        print_err("Error: '" & dir & "' is not a directory")
        return 1
    }
    
    # find all files, get sizes, sort numerically, take top 10
    let result = capture(
        run("find", dir, "-type", "f", "-exec", "du", "-h", "{}", ";")
        | run("sort", "-rh")
        | run("head", "-n", "10"),
        allow_fail=true
    )
    
    if status() != 0 {
        print_err("Error finding files")
        return 1
    }
    
    print("Top 10 largest files in " & dir & ":")
    print(result)
}
```

Compile and run:

```bash
sh2c largest-files.sh2 -o largest-files.sh
./largest-files.sh /var/log
```

### Task B: Delete .bak files older than N days (with confirmation)

```sh2
# cleanup-bak.sh2
# Deletes .bak files older than N days with confirmation.

func usage() {
    print("Usage: cleanup-bak.sh <directory> [--dry-run]")
    print("")
    print("Options:")
    print("  --dry-run    Show what would be deleted without deleting")
}

func main() {
    if argc() < 1 {
        usage()
        return 1
    }
    
    let dir = arg(1)
    let dry_run = false
    
    if argc() >= 2 {
        if arg(2) == "--dry-run" {
            set dry_run = true
        }
    }
    
    if !is_dir(dir) {
        print_err("Error: '" & dir & "' is not a directory")
        return 1
    }
    
    # Find .bak files older than 7 days
    let files = capture(
        run("find", dir, "-name", "*.bak", "-mtime", "+7", "-type", "f"),
        allow_fail=true
    )
    
    if status() != 0 {
        print_err("Error searching for files")
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
        print("No .bak files older than 7 days found.")
        return 0
    }
    
    print($"Found {count} .bak file(s) older than 7 days:")
    for f in lines(files) {
        if f != "" {
            print("  " & f)
        }
    }
    
    if dry_run {
        print("[DRY RUN] Would delete " & count & " file(s)")
        return 0
    }
    
    # Confirm before deletion
    if !confirm($"Delete {count} file(s)?", default=false) {
        print("Aborted.")
        return 0
    }
    
    # Delete each file safely
    let failures = 0
    for f in lines(files) {
        if f != "" {
            run("rm", "--", f, allow_fail=true)
            if status() == 0 {
                print("Deleted: " & f)
            } else {
                print_err("Failed to delete: " & f)
                set failures = failures + 1
            }
        }
    }
    
    if failures > 0 {
        print_err("Completed with " & failures & " failure(s)")
        return 1
    }
    
    print("Done.")
}
```

**Key patterns:**
- `run("rm", "--", f)` — The `--` ensures filenames starting with `-` aren't treated as options
- `confirm(..., default=false)` — Non-interactive mode safely does nothing
- `--dry-run` — Always offer a preview mode for destructive operations

### Task C: Copy matching files to a destination

```sh2
# copy-logs.sh2
# Copies all .log files to a destination directory.

func usage() {
    print("Usage: copy-logs.sh <source-dir> <dest-dir>")
}

func main() {
    if argc() < 2 {
        usage()
        return 1
    }
    
    let src = arg(1)
    let dest = arg(2)
    
    if !is_dir(src) {
        print_err("Error: source '" & src & "' is not a directory")
        return 1
    }
    
    # Create destination if needed
    run("mkdir", "-p", "--", dest, allow_fail=true)
    if status() != 0 {
        print_err("Error: could not create destination directory")
        return 1
    }
    
    let files = capture(
        run("find", src, "-name", "*.log", "-type", "f"),
        allow_fail=true
    )
    
    if status() != 0 {
        print_err("Error searching for files")
        return 1
    }
    
    let copied = 0
    for f in lines(files) {
        if f != "" {
            # Use --target-directory for safe copy
            run("cp", "--", f, dest, allow_fail=true)
            if status() == 0 {
                print("Copied: " & f)
                set copied = copied + 1
            } else {
                print_err("Failed to copy: " & f)
            }
        }
    }
    
    print("Copied " & copied & " file(s) to " & dest)
}
```

**Key patterns:**
- `run("mkdir", "-p", "--", dest)` — Create directory tree safely
- `run("cp", "--", f, dest)` — The `--` protects against filenames starting with `-`

---

## 5. Scoped Working Directory

### Using `with cwd("/path") { ... }`

Change the working directory for a block only:

```sh2
func main() {
    print("Before: " & pwd())
    
    with cwd("/tmp") {
        print("Inside: " & pwd())
        run("ls", "-la")
    }
    
    print("After: " & pwd())  # Back to original
}
```

**The block is scoped:** After the `with cwd(...)` block ends, you're back to the original directory automatically.

### Why it requires a string literal

The path in `cwd(...)` must be a literal string—not a variable:

```sh2
# ✅ Works
with cwd("/tmp/build") {
    run("make")
}

# ❌ Compile error: cwd requires literal path
let dir = "/tmp"
with cwd(dir) { ... }
```

**Why?** This is a safety restriction. Dynamic `cd` paths are a common source of injection bugs. By requiring literals, sh2 forces you to be explicit about where your script runs.

### Workaround for dynamic paths

If you genuinely need a dynamic working directory, use this safe pattern:

```sh2
func main() {
    let dir = arg(1)
    
    # Validate first
    if !is_dir(dir) {
        print_err("Not a directory: " & dir)
        return 1
    }
    
    # Safe pattern: pass path as argument to sh -c
    # sh(...) because: dynamic cwd requires shell; path passed as $1 prevents injection
    run("sh", "-c", "cd -- \"$1\" && ls -la", "sh2", dir)
}
```

**Caveat:** This uses `run("sh", "-c", ...)` which bypasses some sh2 safety. Always:
1. Validate the path first with `is_dir()`
2. Pass the variable as a positional argument (`$1`)
3. Never interpolate directly into the command string

---

## 6. Logging with Redirects

### Basic file logging

```sh2
func main() {
    with redirect { stdout: file("output.log") } {
        print("This goes to the file")
        run("date")
    }
    
    print("This goes to terminal")
}
```

### Logging to file AND terminal (fan-out)

```sh2
func main() {
    with redirect { 
        stdout: [file("output.log"), inherit_stdout()],
        stderr: [file("output.log", append=true), inherit_stderr()]
    } {
        print("Visible on terminal AND logged to file")
        run("some-command")
    }
}
```

### Real example: install script with logging

```sh2
# install-prereqs.sh2
# Installs prerequisites with full logging.

func ensure_log_dir() {
    run("mkdir", "-p", "logs", allow_fail=true)
}

func main() {
    ensure_log_dir()
    let log = "logs/install.log"
    
    print("Installing prerequisites...")
    print($"Log file: {log}")
    
    with redirect { 
        stdout: [file(log, append=true), inherit_stdout()],
        stderr: [file(log, append=true), inherit_stderr()]
    } {
        print("---")
        print("Started: " & capture(run("date")))
        
        print("Updating package lists...")
        sudo("apt-get", "update", n=true, allow_fail=true)
        if status() != 0 {
            print_err("Warning: apt-get update failed")
        }
        
        print("Installing build-essential...")
        sudo("apt-get", "install", "-y", "build-essential", n=true, allow_fail=true)
        if status() != 0 {
            print_err("Failed to install build-essential")
            return 1
        }
        
        print("Finished: " & capture(run("date")))
    }
    
    print("Installation complete. Check " & log & " for details.")
}
```

---

## 7. Rules of Thumb

### When to use `find -delete` vs loop with `rm --`

| Use `find -delete` | Use loop with `rm --` |
|--------------------|----------------------|
| Simple bulk deletion | Need per-file logging |
| No confirmation needed | Need confirmation per file |
| Trust the `find` expression | Want to validate each path |

**With `find -delete`:**
```sh2
run("find", ".", "-name", "*.bak", "-mtime", "+30", "-delete")
```

**With loop:**
```sh2
for f in lines(files) {
    if f != "" {
        run("rm", "--", f, allow_fail=true)
        if status() == 0 { print("Deleted: " & f) }
    }
}
```

### When to prefer pipelines vs structured steps

| Use pipelines | Use structured steps |
|---------------|---------------------|
| All stages are trustworthy | Need error handling per stage |
| Output is small | Output is large/streaming |
| One-liner clarity | Multi-line clarity |

**Pipeline (compact):**
```sh2
let top = capture(run("find", ".") | run("wc", "-l"))
```

**Structured (debuggable):**
```sh2
let files = capture(run("find", "."), allow_fail=true)
if status() != 0 { return 1 }
let count = capture(run("wc", "-l"), allow_fail=true)
```

### When to promote sh2do to a .sh2 script

| Keep as sh2do | Promote to .sh2 file |
|---------------|---------------------|
| Quick one-off | Will run again |
| Testing an idea | Needs arguments |
| Interactive exploration | Needs review/versioning |
| <5 statements | Needs functions |

---

## 8. Common Mistakes

### Mistake 1: Forgetting `--` before paths

```sh2
# ❌ Breaks if path starts with -
run("rm", path)

# ✅ Safe for any path
run("rm", "--", path)
```

### Mistake 2: Not checking `is_dir()` before operations

```sh2
# ❌ Could operate on wrong path
let dir = arg(1)
run("rm", "-rf", "--", dir)

# ✅ Validate first
if !is_dir(dir) {
    print_err("Not a directory: " & dir)
    return 1
}
```

### Mistake 3: Skipping the empty-string check in loops

```sh2
# ❌ May process empty string as filename
for f in lines(files) {
    run("rm", "--", f)  # Could run "rm --" with no file
}

# ✅ Filter empty lines
for f in lines(files) {
    if f != "" {
        run("rm", "--", f)
    }
}
```

---

## Next Steps

You now know how to handle files and directories safely in sh2. Here's where to go next:

### Reference docs
- [Language Reference](../language.md) — Full syntax and semantics
- [No Implicit Expansion](../articles/features/13-no-implicit-expansion.md) — Why strings are strict literals
- [Logging and Redirects](../articles/features/16-logging-and-redirects.md) — Fan-out, file logging

### Related tutorial
- [Error Handling](03-error-handling-and-status.md) — Handle failures in file operations

---

Happy file wrangling! 📁
