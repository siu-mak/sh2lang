---
title: "Error handling you can read: allow_fail, status(), and predictable control flow"
description: "Bash error handling is full of edge cases. sh2 makes failure explicit with allow_fail and status(), so you can review scripts without simulating the shell."
---
<a href="https://github.com/siu-mak/sh2lang">
  <img src="../../images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# Error handling you can read: allow_fail, status(), and predictable control flow

A deploy script had this pattern:

```bash
set -e
result=$(curl -s "$API_URL" | jq '.version')
echo "Deployed version: $result"
```

It worked perfectly—until the API returned an error. `curl` succeeded (HTTP 500 is still a successful curl), `jq` failed to parse the HTML error page, and the script exited silently. No error message. No indication of what went wrong. The `$?` from `jq` was lost, swallowed by the `$()` subshell.

This is the core problem with Bash error handling: **it's implicit, context-dependent, and easy to get wrong.**

sh2 takes a different approach: **failure is explicit.**

---

## Bash vs sh2: the mental model

### Bash: implicit, context-dependent

- `set -e` exits on failure... except in `if`, `while`, pipelines, subshells, and many other contexts
- `$?` holds the exit code... until you run any other command
- Pipelines fail on the last command... unless you use `set -o pipefail`
- Subshells swallow errors unless you propagate them manually

The rules are complex enough that even experienced developers get them wrong.

### sh2: explicit control flow

- Commands fail fast by default (script exits on non-zero)
- `allow_fail=true` opts out of fail-fast for one command
- `status()` returns the exit code—always available, never clobbered
- You branch explicitly: `if status() != 0 { ... }`

**No hidden rules. You say what you mean.**

---

## 10 real-world examples

### 1. set -e vs explicit checks

**Bash:**
```bash
set -e
curl "$url" > file.txt       # Fails silently if curl returns 0 but writes error HTML
grep pattern file.txt        # Might fail; script exits
echo "Found it"              # Only runs if grep succeeds
```

**sh2:**
```sh2
run("curl", url, allow_fail=true) | run("tee", "file.txt")
if status() != 0 {
    print_err("curl failed")
    exit(1)
}

run("grep", "pattern", "file.txt", allow_fail=true)
if status() == 0 {
    print("Found it")
}
```

- Control flow is visible in the code.
- No guessing about when `set -e` applies.

---

### 2. The $? clobbering problem

**Bash:**
```bash
some_command
echo "Command exited with $?"   # $? is now 0 (from echo)
if [ $? -ne 0 ]; then           # Always false!
    echo "Failed"
fi
```

**sh2:**
```sh2
run("some_command", allow_fail=true)
print("Command exited with " & status())
if status() != 0 {
    print("Failed")
}
```

- `status()` is preserved until the next status-updating operation.
- The `print()` statement doesn't clobber it.

---

### 3. Capture output while allowing failure

**Bash:**
```bash
output=$(grep pattern file.txt) || true
# But now $? is 0 (from 'true'), not grep's exit code
```

**sh2:**
```sh2
let output = capture(run("grep", "pattern", "file.txt"), allow_fail=true)
if status() != 0 {
    print_err("grep failed with " & status())
} else {
    print(output)
}
```

- `capture(..., allow_fail=true)` returns the output AND preserves `status()`.
- No need for `|| true` hacks.

---

### 4. Simple branching on status

**Bash:**
```bash
command
if [ $? -eq 0 ]; then
    echo "Success"
else
    echo "Failed with $?"   # Oops, $? is now from the [ test
fi
```

**sh2:**
```sh2
run("command", allow_fail=true)
if status() == 0 {
    print("Success")
} else {
    print("Failed with " & status())
}
```

- `status()` remains stable through the `if` check.

---

### 5. Multi-step workflow with cleanup

**Bash:**
```bash
do_step_a
do_step_b
status_b=$?
if [ $status_b -ne 0 ]; then
    cleanup
    exit $status_b
fi
do_step_c
```

**sh2:**
```sh2
run("do_step_a")
run("do_step_b", allow_fail=true)
if status() != 0 {
    let failed_status = status()
    run("cleanup")
    exit(failed_status)
}
run("do_step_c")
```

- Explicit capture of the failing status.
- Cleanup runs, then exit with the original failure code.

---

### 6. Retry on failure

**Bash:**
```bash
for i in 1 2 3; do
    command && break
    sleep 1
done
```

**sh2:**
```sh2
let attempts = 0
let max_attempts = 3

while attempts < max_attempts {
    run("command", allow_fail=true)
    if status() == 0 {
        break
    }
    set attempts = attempts + 1
    run("sleep", "1")
}

if status() != 0 {
    print_err("Failed after " & max_attempts & " attempts")
    exit(1)
}
```

- Explicit retry logic.
- Clear termination condition.

---

### 7. Pipeline failure with sh() wrapper

**Bash:**
```bash
set -o pipefail
result=$(cmd1 | cmd2 | cmd3)
echo "Status: $?"
```

**sh2:**
```sh2
let result = capture(sh("cmd1 | cmd2 | cmd3"), allow_fail=true)  # sh(...) because: multi-command pipeline
if status() != 0 {
    print_err("Pipeline failed with " & status())
} else {
    print(result)
}
```

- `sh("...")` runs the pipeline in a subshell.
- `status()` reflects the pipeline's exit code.
- You trade some safety for shell pipeline syntax.

---

### 8. sudo with allow_fail (v0.1.1)

**Bash:**
```bash
if sudo -n systemctl restart nginx 2>/dev/null; then
    echo "Restarted"
else
    echo "Failed (probably no sudo access)"
fi
```

**sh2:**
```sh2
sudo("systemctl", "restart", "nginx", n=true, allow_fail=true)
if status() == 0 {
    print("Restarted")
} else {
    print("Failed (probably no sudo access)")
}
```

- `n=true` prevents password prompts.
- `allow_fail=true` prevents abort on failure.
- Named arguments make the intent clear.

---

### 9. Error messaging patterns

**sh2:**
```sh2
run("critical-operation", allow_fail=true)
if status() != 0 {
    print_err("ERROR: critical-operation failed with exit code " & status())
    exit(status())
}
```

- Clear error message to stderr.
- Exit with the actual failure code.

---

### 10. Partial failure is OK (collect and summarize)

**sh2:**
```sh2
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
} else {
    print("All tasks succeeded")
}
```

- All tasks run even if some fail.
- Summary at the end.

---

## Patterns you can copy/paste

### Fail fast with message

```sh2
run("critical-command")
# Script exits immediately if command fails
```

### Allow failure and branch

```sh2
run("maybe-fails", allow_fail=true)
if status() != 0 {
    print_err("Command failed")
}
```

### Capture output + check status

```sh2
let out = capture(run("command"), allow_fail=true)
if status() == 0 {
    print(out)
} else {
    print_err("Failed to capture output")
}
```

### Cleanup on error

```sh2
run("risky-operation", allow_fail=true)
if status() != 0 {
    let code = status()
    run("cleanup")
    exit(code)
}
```

### Retry N times

```sh2
let i = 0
while i < 3 {
    run("flaky", allow_fail=true)
    if status() == 0 { break }
    set i = i + 1
    run("sleep", "1")
}
```

### Summarize failures

```sh2
let failed = 0
for task in tasks {
    run(task, allow_fail=true)
    if status() != 0 { set failed = failed + 1 }
}
if failed > 0 {
    print_err(failed & " task(s) failed")
    exit(1)
}
```

### try/catch for complex error handling

```sh2
try {
    run("step1")
    run("step2")
    run("step3")
} catch {
    print_err("Failed at step with code " & status())
    run("cleanup")
    exit(status())
}
```

---

## Comparison table

| Problem | Bash common solution | Why it's tricky | sh2 approach | Why it's easier to review |
|---------|---------------------|-----------------|--------------|---------------------------|
| Exit on any failure | `set -e` | Doesn't work in `if`, `while`, `$()`, pipelines | Default behavior | Consistent everywhere |
| Check exit code | `$?` | Gets clobbered by next command | `status()` | Stable until next status-updating call |
| Capture + check | `out=$(cmd) || true` | Loses actual exit code | `capture(..., allow_fail=true)` | Preserves both output and status |
| Retry on failure | `for i in 1 2 3; do cmd && break; done` | No clear "all retries failed" handling | Explicit while loop with counter | Clear termination condition |
| Pipeline failure | `set -o pipefail` | Only reports last non-zero exit | `sh("...") + status()` | Explicit handling |
| Cleanup on error | Manual `trap` or `if` chains | Easy to forget, complex scoping | Explicit branch + cleanup call | Visible in code flow |
| Partial failures | Ignore with `|| true` everywhere | Swallows real errors | `allow_fail + counter` | Summarized at end |
| Sudo failure | `sudo ... 2>/dev/null || echo fail` | Stderr redirect hides real errors | `sudo(..., allow_fail=true)` | Named options, clean branching |

---

## The philosophy

Bash error handling grew organically. It's powerful, but the rules are scattered across man pages, StackOverflow answers, and hard-won experience.

sh2 makes error handling explicit:
- **Default: fail fast.** If a command fails, the script stops.
- **Opt-in: allow_fail.** When you want to handle failure yourself.
- **Observable: status().** The exit code is always there when you need it.

The result: you can review a script and know what happens when things go wrong—without simulating the shell in your head.

---

# Docs

The GitHub repo is here:  
**[https://github.com/siu-mak/sh2lang](https://github.com/siu-mak/sh2lang)**

## Further Documentation

- [`docs/language.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/language.md) — full language reference (syntax + semantics)
- [`docs/sh2do.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/sh2do.md) — sh2do CLI documentation
- `tests/` — fixtures and integration tests (acts as an executable spec)
