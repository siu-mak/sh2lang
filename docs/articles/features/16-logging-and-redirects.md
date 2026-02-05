---
title: "Logging without regret: redirects, tee patterns, and scoped output"
description: "Bash logging often turns into file descriptor gymnastics. sh2 keeps logging scoped and readable with redirect blocks."
---
<a href="https://github.com/siu-mak/sh2lang">
  <img src="../../images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# Logging without regret: redirects, tee patterns, and scoped output

An install script logged everything to a file:

```bash
exec > >(tee -a /var/log/install.log) 2>&1
```

It worked—until someone ran it twice. The second run's logs were interleaved with the first's buffered output. Worse, a later `read` command hung because stdin was somehow affected by the earlier `exec` redirect. No one could explain why.

This is the file descriptor problem: **Bash redirection is global, implicit, and hard to reason about.**

sh2's answer: **scoped redirect blocks.** Logging applies only where you say it applies.

---

## The 3 most common Bash logging patterns (and why they're tricky)

### 1. `cmd | tee -a log`

```bash
important_command | tee -a install.log
```

**Footgun:** The pipeline's exit code is `tee`'s exit code, not `important_command`'s. Even with `set -o pipefail`, the behavior is subtle and easy to forget.

```bash
false | tee -a log
echo $?  # 1 with pipefail, 0 without
```

### 2. `exec > >(tee -a log) 2>&1`

```bash
exec > >(tee -a install.log) 2>&1
# All output now goes to tee AND console
apt-get update
apt-get install ...
```

**Footgun:** This is global. It affects every command after it—including interactive prompts, `read` statements, and subshells. It's also Bash-specific (process substitution) and behaves differently across Bash versions.

### 3. `cmd >>log 2>&1`

```bash
long_command >>install.log 2>&1
```

**Footgun:** No console output. If the command hangs, you won't know until you check the log. And reviewers have to simulate `2>&1` ordering in their heads (stderr-to-stdout must come *after* the stdout redirect).

---

## sh2's mental model: scoped logging

In sh2, redirects are declared in a **scoped block**:

```sh2
with redirect { stdout: file("install.log") } {
    run("apt-get", "update")
    run("apt-get", "install", "-y", "nginx")
}
# Log file closed. Output goes back to normal.
```

### Key properties:

1. **Scoped**: Redirects apply only inside the block.
2. **Explicit**: You declare what goes where.
3. **Multi-sink support**: Write to file AND console (tee equivalent).
4. **Append mode**: `file("log", append=true)` is explicit.

---

## 10 real-world examples

### 1. Log stdout to file, still show console output (tee equivalent)

**Bash:**
```bash
cmd | tee -a install.log
```

**sh2:**
```sh2
with redirect { stdout: [file("install.log"), inherit_stdout()] } {
    run("apt-get", "update")
}
```

- Output goes to both file AND console.
- Exit code is preserved (no pipeline masking).
- `inherit_stdout()` keeps the terminal visible.

---

### 2. Log stderr separately from stdout

**Bash:**
```bash
cmd >>stdout.log 2>>stderr.log
```

**sh2:**
```sh2
with redirect { stdout: file("stdout.log"), stderr: file("stderr.log") } {
    run("build.sh")
}
```

- Clear: stdout goes one place, stderr another.
- No mental simulation of `2>&1` vs `2>>`.

---

### 3. Append logs across runs

**Bash:**
```bash
cmd >>install.log 2>&1
```

**sh2:**
```sh2
with redirect { stdout: file("install.log", append=true), stderr: to_stdout() } {
    run("install-step")
}
```

- `append=true` is explicit—no guessing about `>` vs `>>`.
- `stderr: to_stdout()` merges stderr into stdout (which then goes to the file).

---

### 4. Scoped logging: one block logs, next block doesn't

**Bash:**
```bash
exec > >(tee -a log) 2>&1
cmd1  # logged
# How do I stop logging?
```

**sh2:**
```sh2
with redirect { stdout: [file("log"), inherit_stdout()] } {
    run("cmd1")  // logged
}
run("cmd2")  // NOT logged
```

- When the block ends, redirection ends.
- No global state to undo.

---

### 5. Capture output AND log it

**Bash:**
```bash
output=$(cmd | tee -a log)
# Exit code is tee's, not cmd's
```

**sh2:**
```sh2
let output = ""
with redirect { stdout: [file("log"), inherit_stdout()] } {
    set output = capture(run("cmd"))
}
print("Captured: " & output)
```

- Output is logged AND captured into a variable.
- Exit code is preserved via `status()` if you use `allow_fail=true`.

> **Note:** This captures stdout. If you also need to log stderr, add a stderr redirect.

---

### 6. Multi-step install script logging

**Bash:**
```bash
exec > >(tee -a /var/log/install.log) 2>&1
apt-get update
apt-get install -y nginx
systemctl start nginx
```

**sh2:**
```sh2
with redirect { stdout: [file("/var/log/install.log"), inherit_stdout()], stderr: to_stdout() } {
    run("apt-get", "update")
    run("apt-get", "install", "-y", "nginx")
    run("systemctl", "start", "nginx")
}
```

- All commands logged with visible output.
- When the block ends, logging stops.
- Reviewers know exactly what's logged.

---

### 7. Fail-fast with logs

**sh2:**
```sh2
with redirect { stdout: [file("deploy.log", append=true), inherit_stdout()] } {
    run("step1")
    run("step2", allow_fail=true)
    if status() != 0 {
        print_err("step2 failed with " & status())
        exit(status())
    }
    run("step3")
}
```

- Logs capture everything, including the failure.
- `allow_fail=true` + `status()` for explicit error handling.

---

### 8. Why process substitution is hard to audit

**Bash:**
```bash
exec > >(tee -a log) 2>&1
# What happens to stdin?
# What if tee is slow and buffers?
# What if we fork a background job?
```

These questions require simulating Bash internals. Reviewers can't easily answer them.

**sh2:**
```sh2
with redirect { stdout: [file("log"), inherit_stdout()] } {
    ...
}
```

- Scoped. The redirect applies to commands inside the block.
- No hidden buffering surprises.
- No stdin side effects.

---

### 9. Audit intent: reader sees what happens

**Bash:**
```bash
cmd 2>&1 | tee -a log | grep pattern >> matches.txt
```

A reviewer must mentally trace: stderr merges to stdout, pipes to tee (file + next stage), then grep filters to another file. Exit code? Probably wrong.

**sh2:**
```sh2
with redirect { stdout: [file("log"), inherit_stdout()], stderr: to_stdout() } {
    let out = capture(run("cmd"))
}
// Then process 'out' separately
```

- Intent is explicit: log everything, capture stdout.
- Processing happens in separate, readable steps.

---

### 10. Escape hatch: sh("...") for Bash-only tricks

Sometimes you genuinely need process substitution or complex FD plumbing:

**sh2:**
```sh2
sh("curl -s https://example.com 2>&1 | tee -a curl.log | jq '.'")
```

- `sh("...")` gives you raw Bash.
- You lose sh2's guarantees inside the string.
- Use sparingly, and document why.

---

## Copy/paste recipes

### Log everything with visible output

```sh2
with redirect { stdout: [file("output.log"), inherit_stdout()], stderr: to_stdout() } {
    run("your-command")
}
```

### Append logs

```sh2
with redirect { stdout: file("app.log", append=true) } {
    run("daily-task")
}
```

### Separate error log

```sh2
with redirect { stdout: file("stdout.log"), stderr: file("stderr.log") } {
    run("build")
}
```

### Log with timestamps (using sh escape hatch)

```sh2
// sh2 doesn't have built-in timestamps, but you can wrap:
sh("your-command 2>&1 | while read line; do echo \"$(date): $line\"; done | tee -a timed.log")
```

### CI-friendly logging

```sh2
// Log to file, show on console for CI visibility
with redirect { stdout: [file("ci-build.log"), inherit_stdout()], stderr: to_stdout() } {
    run("make", "build")
    run("make", "test")
}
// CI sees output in real-time, and logs are saved as artifacts
```

### Short-form with log() helper

```sh2
with log("activity.log", append=true) {
    run("echo", "hello")
}
```

> **Note:** `with log()` is a convenience wrapper (Bash target only). It's equivalent to `with redirect { stdout: file(...), stderr: to_stdout() }`.

### Multiple output files (no console)

```sh2
with redirect { stdout: [file("primary.log"), file("backup.log")] } {
    run("critical-command")
}
// Both files get the same output; console is silent
```

---

## Comparison table

| Goal | Bash solution | Why it's tricky | sh2 approach | Why it's easier to review |
|------|--------------|-----------------|--------------|---------------------------|
| Log + show output | `cmd \| tee log` | Exit code from tee, not cmd | `[file(...), inherit_stdout()]` | Exit code preserved |
| Append logs | `cmd >>log 2>&1` | `2>&1` ordering matters | `file(..., append=true)` | Explicit append mode |
| Separate stderr | `cmd >out 2>err` | Easy to forget the `2>` | `stdout: file(...), stderr: file(...)` | Named destinations |
| Stop logging | (undo exec?) | Global state hard to reset | Block ends = redirect ends | Scoped by design |
| Log multi-step | `exec > >(tee...)` | Global, affects everything | `with redirect { ... } { ... }` | Clear scope |
| Capture + log | `out=$(cmd \| tee log)` | Exit code lost | `capture()` inside redirect block | Status preserved |
| Review FD plumbing | `2>&1 \| tee -a` chains | Mental simulation required | Read the `redirect { }` spec | Declarative |
| Complex pipelines | Process substitution | Bash-specific, version-dependent | `sh("...")` escape hatch | Explicit trade-off |

---

## The philosophy

Bash file descriptor redirection is powerful. You can do almost anything with `>&`, `<()`, and friends.

But that power comes at a cost: **reviewability.** When you see `exec > >(tee -a log) 2>&1`, you need to know Bash internals to understand what will happen—and what might go wrong.

sh2 trades some flexibility for clarity:
- **Scoped**: Redirects apply only inside a block.
- **Declarative**: You say what goes where.
- **Explicit multi-sink**: Fan-out is a list, not FD arithmetic.

The result: you can read the code and know what gets logged, without simulating the shell.

---

# Docs

The GitHub repo is here:  
**[https://github.com/siu-mak/sh2lang](https://github.com/siu-mak/sh2lang)**

## Further Documentation

- [`docs/language.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/language.md) — full language reference (syntax + semantics)
- [`docs/sh2do.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/sh2do.md) — sh2do CLI documentation
- `tests/` — fixtures and integration tests (acts as an executable spec)
