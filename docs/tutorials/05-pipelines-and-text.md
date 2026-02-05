---
title: "Pipelines and text processing: sh2 structure without losing Bash power"
description: "Structured pipelines with run(...) | run(...), capturing output, and knowing when to keep a pipeline in Bash."
---
<a href="https://github.com/siu-mak/sh2lang">
  <img src="../images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# Pipelines and Text Processing

This tutorial teaches you how to build pipelines in sh2—and when to let Bash do the heavy lifting instead.

**What you'll learn:**
- Write readable sh2 pipelines with `run(...) | run(...)`
- Capture pipeline output safely
- Understand how `status()` works with pipelines
- Know when sh2 helps vs when Bash is simpler
- Use `sh(...)` as an escape hatch (rarely)

**Prerequisites:** Complete [Tutorial 01: Getting Started](01-getting-started.md) and [Tutorial 03: Error Handling](03-error-handling-and-status.md).

---

## 1. Why Pipelines Feel Easy in Bash—And Why They Become Hard

Bash pipelines are concise and powerful:

```bash
ps aux | grep nginx | grep -v grep | awk '{print $2}' | xargs kill -9
```

This "works" for quick commands. But as pipelines grow, problems emerge:

### Problem 1: Quoting inside awk/sed

```bash
# Which quotes go where? Easy to break.
grep "error" log.txt | awk -F: '{print $1 " had " $2 " errors"}'
```

### Problem 2: Status handling

```bash
set -o pipefail
result=$(curl -s "$url" | jq '.data')
# Did curl fail? Did jq fail? $? only tells you "something failed"
```

### Problem 3: Readability at scale

```bash
find . -name "*.log" -mtime +7 -print0 | \
    xargs -0 grep -l "ERROR" | \
    sort -u | \
    head -20 | \
    while read f; do echo "Problem: $f"; done
```

This is hard to review. What happens if `find` fails? What if a filename has special characters? The answers require deep Bash knowledge.

---

## 2. sh2 Structured Pipelines

sh2 pipelines use `|` between `run(...)` calls. Each argument is safely quoted.

### Simple example

```sh2
func main() {
    run("printf", "a\nb\nc\n") | run("wc", "-l")
}
```

Output: `3`

### Slightly longer: count unique shells

```sh2
func main() {
    # Get unique shells from /etc/passwd
    let shells = capture(
        run("cut", "-d:", "-f7", "/etc/passwd")
        | run("sort")
        | run("uniq", "-c")
        | run("sort", "-rn")
    )
    
    print("Shell usage:")
    print(shells)
}
```

**What's clearer:**
- Each stage is a distinct `run(...)` call
- Arguments don't need quoting gymnastics
- The pipeline structure is visually obvious

---

## 3. Capturing Pipeline Output

Use `capture(...)` to store pipeline output in a variable:

```sh2
func main() {
    let line_count = capture(
        run("find", ".", "-name", "*.txt")
        | run("wc", "-l")
    )
    
    print("Found " & trim(line_count) & " text files")
}
```

### Use `trim(...)` to remove trailing whitespace

Many commands (like `wc`) output extra whitespace. Use `trim()`:

```sh2
let count = trim(capture(run("ls", "-1") | run("wc", "-l")))
```

### Handling failures with `allow_fail=true`

For pipelines that might fail:

```sh2
func main() {
    let result = capture(
        run("grep", "ERROR", "app.log")
        | run("wc", "-l"),
        allow_fail=true
    )
    
    if status() != 0 {
        print("No errors found (or file missing)")
    } else {
        print("Error count: " & trim(result))
    }
}
```

---

## 4. Exit Status Rules

### How `status()` works after a pipeline

After a pipeline, `status()` reflects the **last command's exit code** (like Bash default behavior).

```sh2
func main() {
    # Pipeline: true | false
    run("true") | run("false")
    print("Status: " & status())  # Prints: 1 (from false)
}
```

This is important: if `grep` finds nothing (exit 1) but `wc` succeeds (exit 0), the overall status is 0.

### Pattern: if pipeline fails, print error and exit

```sh2
func main() {
    let output = capture(
        run("curl", "-s", "https://api.example.com/data")
        | run("jq", ".items"),
        allow_fail=true
    )
    
    if status() != 0 {
        print_err("Pipeline failed (curl or jq)")
        exit(1)
    }
    
    print(output)
}
```

### When you need per-stage error checking

Break the pipeline into steps:

```sh2
func main() {
    let raw = capture(run("curl", "-sf", "https://api.example.com/data"), allow_fail=true)
    if status() != 0 {
        print_err("curl failed")
        exit(1)
    }
    
    # Write to temp file for jq
    write_file("/tmp/api_data.json", raw)
    
    let parsed = capture(run("jq", ".items", "/tmp/api_data.json"), allow_fail=true)
    if status() != 0 {
        print_err("jq failed: invalid JSON?")
        exit(1)
    }
    
    print(parsed)
}
```

---

## 5. Real Examples

### Example 1: Parse `w` output to list unique usernames

**Bash:**
```bash
w -h | awk '{print $1}' | sort -u
```

**sh2:**
```sh2
func main() {
    let users = capture(
        run("w", "-h")
        | run("awk", "{print $1}")
        | run("sort", "-u")
    )
    
    print("Logged-in users:")
    for user in lines(users) {
        if user != "" {
            print("  " & user)
        }
    }
}
```

**What got clearer:**
- Intent is explicit: "get users, then iterate"
- Adding per-user logic is easy

**What got worse:**
- More lines than the Bash one-liner

---

### Example 2: Extract and count shells from `/etc/passwd`

**Bash:**
```bash
cut -d: -f7 /etc/passwd | sort | uniq -c | sort -rn | head -5
```

**sh2:**
```sh2
func main() {
    print("Top 5 shells by user count:")
    
    run("cut", "-d:", "-f7", "/etc/passwd")
    | run("sort")
    | run("uniq", "-c")
    | run("sort", "-rn")
    | run("head", "-n", "5")
}
```

**What got clearer:**
- Arguments are unambiguous (`"-d:"` vs `-d:` quoting issues)
- Easy to modify one stage

**What got worse:**
- Slightly more verbose

---

### Example 3: JSON API with error handling (curl | jq)

**Bash:**
```bash
result=$(curl -sf "$API_URL" | jq -r '.name') || { echo "Failed"; exit 1; }
echo "Name: $result"
```

**sh2:**
```sh2
func main() {
    let url = "https://api.github.com/repos/siu-mak/sh2lang"
    
    let data = capture(
        run("curl", "-sf", url)
        | run("jq", "-r", ".name"),
        allow_fail=true
    )
    
    if status() != 0 {
        print_err("Failed to fetch or parse API response")
        exit(1)
    }
    
    print("Repo name: " & trim(data))
}
```

**What got clearer:**
- Error handling is explicit
- URL is a variable, not inline with quoting risk

**What got worse:**
- More lines for the same result

---

### Example 4: Grep logs and summarize error counts

**Bash:**
```bash
grep -c "ERROR" *.log 2>/dev/null | awk -F: '{sum+=$2} END {print sum}'
```

**sh2:**
```sh2
func main() {
    # Find all log files and count ERROR lines
    let logs = capture(run("find", ".", "-name", "*.log", "-type", "f"), allow_fail=true)
    
    let total = 0
    for log in lines(logs) {
        if log != "" {
            let count = trim(capture(
                run("grep", "-c", "ERROR", log),
                allow_fail=true
            ))
            
            if status() == 0 {
                if count != "" {
                    if count != "0" {
                        print(log & ": " & count & " errors")
                        # Note: string-to-int addition requires a workaround
                    }
                }
            }
        }
    }
    
    print("See above for per-file counts")
}
```

**What got clearer:**
- Works correctly even if no `.log` files exist
- Handles files with spaces in names

**What got worse:**
- Significantly more verbose
- This is a case where Bash's terseness wins

---

### Example 5: CSV formatting with awk

For complex text transformations, `awk` shines. Here's a case where using `sh(...)` is reasonable:

**Bash:**
```bash
awk -F, '{printf "%-20s %s\n", $1, $2}' data.csv
```

**sh2:**
```sh2
func main() {
    # Use run() for simple awk patterns
    run("awk", "-F,", "{printf \"%-20s %s\\n\", $1, $2}", "data.csv")
}
```

**Note:** Even awk expressions work as `run()` arguments when they're simple. For complex multi-line awk scripts, consider keeping them in a separate `.awk` file and calling `run("awk", "-f", "script.awk", "data.csv")`.

---

## 6. Where Bash Still Wins

Be honest: sh2 is not always the best choice.

### Category 1: Dense awk/sed one-liners

```bash
awk '{gsub(/foo/,"bar"); print}' file.txt
sed -n '/START/,/END/p' log.txt
```

These are compact, well-tested patterns. Wrapping them in sh2 adds verbosity without adding safety.

### Category 2: Process substitution

```bash
diff <(sort file1.txt) <(sort file2.txt)
```

sh2 has no equivalent. You'd need a workaround with temp files.

### Category 3: Job control

```bash
long_task &
pid=$!
# ... do other work ...
wait $pid
```

Background jobs and `wait` aren't part of sh2's structured model.

### The honest truth

sh2 is great for **structured glue**—scripts that run commands, check status, branch, and log. It's not trying to replace Bash's text-processing DSL.

---

## 7. Rule of Thumb + Decision Table

| Your pipeline... | Prefer |
|------------------|--------|
| Is a quick one-liner you'll run once | Bash |
| Uses complex awk/sed patterns | Bash (or external script) |
| Needs error handling per stage | sh2 |
| Will be reviewed by others | sh2 |
| Mixes commands with `sudo`, `confirm`, logging | sh2 |
| Uses process substitution or job control | Bash |
| Has 2–4 stages with standard tools | Either works |

### Quick decision flow

1. **Will this be reviewed?** → Lean toward sh2
2. **Is it mostly text transformation?** → Lean toward Bash
3. **Do I need per-command error handling?** → sh2
4. **Is it a one-off command?** → Bash
5. **Does it use `sudo`, `confirm`, or file logging?** → sh2

---

## 8. The Escape Hatch: `sh(...)`

When you genuinely need shell syntax, use `sh(...)`. But be explicit about why:

```sh2
func main() {
    # sh(...) because: process substitution <(...) has no sh2 equivalent
    run("diff", "<(sort file1.txt)", "<(sort file2.txt)", allow_fail=true)
    # ❌ This won't work: <(...) is literal text, not substitution
    
    # Correct approach: use temp files or accept sh(...)
}
```

If you must use complex pipelines with shell features:

```sh2
func main() {
    # sh(...) because: multi-stage pipeline with subshell grouping
    # Note: No user input is interpolated here; pipeline is static
    let result = capture(
        run("sh", "-c", "cat *.log 2>/dev/null | grep ERROR | wc -l"),
        allow_fail=true
    )
    
    if status() == 0 {
        print("Total errors: " & trim(result))
    }
}
```

**Safety rule:** Never interpolate user input into `sh(...)` commands. Always validate first.

---

## Next Steps

You now understand when sh2 pipelines help and when Bash is the right tool.

### Related tutorials
- [Error Handling](03-error-handling-and-status.md) — Handle pipeline failures gracefully

### Feature articles
- [Where Bash Still Wins](../articles/introduction/04-where-bash-still-wins.md) — Honest comparison
- [No Implicit Expansion](../articles/features/13-no-implicit-expansion.md) — Why sh2 strings are safe

### Reference
- [Language Reference](../language.md) — Full pipeline syntax

---

Happy piping! 🔧
