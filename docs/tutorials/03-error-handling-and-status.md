---
title: "Error handling in sh2: fail fast, allow failure, and keep the real status"
description: "A hands-on guide to allow_fail, status(), and writing scripts that behave predictably when commands fail."
---
<a href="https://github.com/siu-mak/sh2lang">
  <img src="../images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# Error Handling in sh2

This tutorial teaches you how sh2 handles command failures—and how to write scripts that behave predictably when things go wrong.

**What you'll learn:**
- How sh2's fail-fast default works
- When and how to use `allow_fail=true`
- Using `status()` to check exit codes (without the Bash `$?` footgun)
- Patterns for collecting failures and reporting clearly
- When Bash pipelines are fine vs when sh2 structure helps

**Prerequisites:** Complete [Tutorial 01: Getting Started](01-getting-started.md) first.

---

## 1. The Mental Model

**In sh2, command failure is an error unless you explicitly allow it.**

This is the opposite of Bash, where you must opt *in* to fail-fast behavior with `set -e` (which has many exceptions).

| Bash approach | sh2 approach |
|---------------|--------------|
| Commands succeed or fail silently | Commands fail fast by default |
| `set -e` opts into "fail-fast" (with exceptions) | Default behavior, no exceptions |
| `cmd || true` suppresses failure | `allow_fail=true` allows failure explicitly |
| `$?` holds exit code (until next command) | `status()` holds exit code (stable) |

### Why this matters

Consider this Bash pattern:

```bash
set -e
cmd || { echo "failed"; echo $?; }   # Oops: $? is now 0 (from echo)
```

The `$?` is clobbered immediately by the `echo` inside the block. You never see the real exit code.

In sh2, `status()` remains stable until the next status-updating command:

```sh2
run("cmd", allow_fail=true)
if status() != 0 {
    print("failed with " & status())  # status() still has cmd's exit code
}
```

---

## 2. Fail-Fast by Default

Create this script:

```sh2
func main() {
    print("Step 1")
    run("false")        # This fails (exit code 1)
    print("Step 2")     # Never runs
}
```

Compile and run:

```bash
sh2c fail_demo.sh2 -o fail_demo.sh
./fail_demo.sh
echo "Script exit code: $?"
```

Output:

```text
Step 1
Script exit code: 1
```

**Key point:** The script stopped at `run("false")`. Step 2 never printed. The script exited with code 1.

This is sh2's default: **if a command fails, the script stops immediately**.

---

## 3. Allow Failure Intentionally

Sometimes you want a command to fail without stopping the script. Use `allow_fail=true`.

### Statement form

```sh2
func main() {
    run("grep", "pattern", "missing.txt", allow_fail=true)
    
    if status() != 0 {
        print("File not found or no match")
    }
    
    print("Script continues")
}
```

The `allow_fail=true` option tells sh2: "Let this command fail without aborting. I'll check the result myself."

### Capture form

When capturing output from a potentially failing command:

```sh2
func main() {
    let output = capture(run("cat", "/etc/shadow"), allow_fail=true)
    
    if status() != 0 {
        print_err("Could not read file (status " & status() & ")")
    } else {
        print(output)
    }
}
```

You can also write it with `allow_fail` on the inner `run`:

```sh2
let output = capture(run("cat", "/etc/shadow", allow_fail=true))
```

Both forms are equivalent—the `allow_fail` is "hoisted" to the capture.

---

## 4. Why status() Is Easier Than Bash $?

In Bash, `$?` is fragile. It changes after every command, including commands inside your error-handling code.

### The classic Bash footgun

```bash
cmd
if [ $? -ne 0 ]; then
    echo "Command failed"        # $? is now 0 (from echo)
    echo "Exit code was: $?"     # Always prints 0!
fi
```

Even worse:

```bash
cmd
status=$?
echo "Got status"                # If you forget this line...
if [ $status -ne 0 ]; then      # ...you might use $? here by mistake
```

### sh2's solution

In sh2, `status()` holds the exit code until the *next* status-updating operation (like another `run` call):

```sh2
run("cmd", allow_fail=true)
print("Command exited with " & status())   # print() doesn't clobber status
if status() != 0 {
    print("Failed with " & status())       # Still the original status
}
```

**What updates status()?**
- `run(...)` 
- `capture(...)` 
- `try_run(...)`
- Filesystem predicates (`exists()`, `is_file()`, etc.)

**What does NOT update status()?**
- `print()`, `print_err()`
- `let`, `set`
- `if`, `while`, `for`
- Arithmetic and string operations

---

## 5. Patterns You'll Use in Real Scripts

### Pattern A: Try optional thing, continue

For commands that might fail but aren't critical:

```sh2
func main() {
    # Try to load optional config
    run("source", "~/.myconfig", allow_fail=true)
    
    # Continue regardless
    print("Continuing with defaults if config failed")
}
```

### Pattern B: Try N things, report failures at end

When you need to run multiple tasks and report all failures:

```sh2
func main() {
    let failures = 0
    
    run("task1", allow_fail=true)
    if status() != 0 {
        print_err("task1 failed")
        set failures = failures + 1
    }
    
    run("task2", allow_fail=true)
    if status() != 0 {
        print_err("task2 failed")
        set failures = failures + 1
    }
    
    run("task3", allow_fail=true)
    if status() != 0 {
        print_err("task3 failed")
        set failures = failures + 1
    }
    
    if failures > 0 {
        print_err("Completed with " & failures & " failure(s)")
        exit(1)
    }
    
    print("All tasks succeeded")
}
```

### Pattern C: Retry with backoff

For flaky commands that might succeed on retry:

```sh2
func main() {
    let attempts = 0
    let max_attempts = 3
    
    while attempts < max_attempts {
        run("flaky-command", allow_fail=true)
        if status() == 0 {
            break
        }
        
        set attempts = attempts + 1
        print("Attempt " & attempts & " failed, retrying...")
        run("sleep", "1")
    }
    
    if status() != 0 {
        print_err("Failed after " & max_attempts & " attempts")
        exit(1)
    }
    
    print("Success!")
}
```

### Pattern D: Best-effort cleanup that never hides failures

When cleanup must run but shouldn't hide the original error:

```sh2
func main() {
    run("risky-operation", allow_fail=true)
    
    if status() != 0 {
        let original_code = status()
        print_err("Operation failed with " & original_code)
        
        # Cleanup runs, but we preserve the original exit code
        run("cleanup", allow_fail=true)
        
        exit(original_code)
    }
    
    print("Operation succeeded")
}
```

---

## 6. Mini Project: Check a List of Commands

Build a small tool that:
1. Runs a list of commands
2. Prints OK/FAIL for each
3. Exits 0 if all succeed, 1 if any fail

### Create `check-commands.sh2`

```sh2
# check-commands.sh2
# Runs each command and reports status.

func check(cmd) {
    run(cmd, allow_fail=true)
    if status() == 0 {
        print("OK:   " & cmd)
        return 0
    } else {
        print("FAIL: " & cmd & " (exit " & status() & ")")
        return 1
    }
}

func main() {
    let failures = 0
    
    # Commands to check (add your own)
    let r = check("true")
    if r != 0 { set failures = failures + 1 }
    
    set r = check("false")
    if r != 0 { set failures = failures + 1 }
    
    set r = check("ls")
    if r != 0 { set failures = failures + 1 }
    
    set r = check("nonexistent-command-xyz")
    if r != 0 { set failures = failures + 1 }
    
    print("")
    if failures == 0 {
        print("All commands OK")
        exit(0)
    } else {
        print(failures & " command(s) failed")
        exit(1)
    }
}
```

### Compile and run

```bash
sh2c check-commands.sh2 -o check-commands.sh
./check-commands.sh
echo "Exit code: $?"
```

Expected output:

```text
OK:   true
FAIL: false (exit 1)
OK:   ls
FAIL: nonexistent-command-xyz (exit 127)

2 command(s) failed
Exit code: 1
```

---

## 7. When Bash Pipelines Are Fine vs When sh2 Helps

### Bash pipelines are fine when:

- You're doing simple text processing: `grep | sort | uniq`
- All commands are trusted and won't fail unexpectedly
- You don't need to inspect intermediate exit codes

### sh2 structure helps when:

- You need to handle failure at specific stages
- You need to capture exit codes for logging/reporting
- The script runs in CI/CD where failure visibility matters
- The script will be reviewed by others

**Example:** Simple grep pipeline (Bash is fine):

```bash
cat log.txt | grep ERROR | wc -l
```

**Example:** Same thing with error handling (sh2 helps):

```sh2
let count = capture(
    run("cat", "log.txt") | run("grep", "ERROR") | run("wc", "-l"),
    allow_fail=true
)

if status() != 0 {
    print_err("Pipeline failed")
    exit(1)
}

print("Error count: " & trim(count))
```

---

## 8. Common Mistakes and How to Avoid Them

### Mistake 1: Forgetting `allow_fail=true`

```sh2
# ❌ Script exits if grep finds nothing
run("grep", "pattern", "file.txt")

# ✅ Handle the "not found" case
run("grep", "pattern", "file.txt", allow_fail=true)
if status() != 0 {
    print("Pattern not found")
}
```

### Mistake 2: Assuming `capture` implies success

```sh2
# ❌ output might be empty string from failed command
let output = capture(run("cat", "missing.txt"), allow_fail=true)
print(output)

# ✅ Check status after capture
let output = capture(run("cat", "missing.txt"), allow_fail=true)
if status() != 0 {
    print_err("Could not read file")
} else {
    print(output)
}
```

### Mistake 3: Ignoring `status()` after a failing run

```sh2
# ❌ Runs command but ignores failure
run("flaky", allow_fail=true)
run("next-step")  # This might be wrong if flaky was supposed to succeed

# ✅ Check and decide
run("flaky", allow_fail=true)
if status() != 0 {
    print_err("flaky failed, aborting")
    exit(1)
}
run("next-step")
```

### Mistake 4: Too much logic in pipelines

```sh2
# ❌ Hard to debug: which stage failed?
let x = capture(run("a") | run("b") | run("c") | run("d"))

# ✅ Break it up when you need visibility
run("a", allow_fail=true)
if status() != 0 { exit(1) }

let b_out = capture(run("b"), allow_fail=true)
if status() != 0 { exit(1) }

# etc.
```

---

## Next Steps

You now understand sh2's error-handling model. Here's where to go next:

### Reference docs
- [Error Handling Feature Article](../articles/features/15-error-handling.md) — 10 real-world examples
- [Language Reference](../language.md) — Full syntax and semantics
- [sh2do Documentation](../sh2do.md) — Snippet runner details

### Related tutorials
- [Tutorial 02: Building a Real Tool](02-building-a-real-tool.md) — Uses error handling in a complete tool

---

Happy error-free scripting! 🛡️
