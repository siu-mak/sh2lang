---
title: "sh2 in CI/CD: non-interactive safety, logs, and predictable exits"
description: "How to run sh2 scripts in automation: confirm defaults, environment overrides, sudo non-interactive mode, and reliable error handling."
---
<a href="https://github.com/siu-mak/sh2lang">
  <img src="../images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# sh2 in CI/CD and Automation

This tutorial teaches you how to write sh2 scripts that run reliably in CI/CD pipelines—no hanging, no silent failures, and clear logs.

**What you'll learn:**
- Handle missing TTY (no interactive prompts)
- Use `confirm(..., default=...)` with `SH2_YES` / `SH2_NO`
- Use `sudo(..., n=true)` to avoid password hangs
- Control failure behavior with `allow_fail=true` + `status()`
- Emit logs suitable for CI artifacts
- Use `sh2c` and `sh2do` in CI steps

**Prerequisites:** Complete [Tutorial 01: Getting Started](01-getting-started.md) and [Tutorial 03: Error Handling](03-error-handling-and-status.md).

---

## 1. The CI Problem: Scripts Hang and Nobody Knows Why

CI/CD runners have no TTY. This breaks scripts in subtle ways:

### Problem 1: Prompts wait forever

```bash
# In CI, this hangs until the job times out
read -p "Continue? " answer
```

### Problem 2: `sudo` asks for a password

```bash
# In CI, this hangs or fails silently
sudo systemctl restart nginx
```

### Problem 3: Silent failures get ignored

```bash
# set -e doesn't catch everything
result=$(curl "$API" | jq '.data')  # jq failed? curl failed? Who knows.
```

sh2 provides explicit controls for all of these.

---

## 2. Confirm in Automation (v0.1.1)

The `confirm(...)` helper has a `default=` parameter for non-interactive mode:

```sh2
func main() {
    if confirm("Deploy to production?", default=false) {
        print("Deploying...")
        run("deploy.sh")
    } else {
        print("Skipped: no confirmation")
    }
}
```

### Behavior table

| Environment | TTY? | Result |
|-------------|------|--------|
| Local terminal | Yes | Prompts user, waits for y/n |
| CI (no TTY) | No | Returns `false` (the default) |
| `SH2_YES=1` | Either | Returns `true` immediately |
| `SH2_NO=1` | Either | Returns `false` immediately |

### CI-friendly pattern

In your CI script:

```yaml
# .github/workflows/deploy.yml
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - name: Deploy
        run: SH2_YES=1 ./deploy.sh
```

In your sh2 script:

```sh2
func main() {
    # In CI: SH2_YES=1 means this returns true
    # Locally: prompts user
    if confirm("Apply changes?", default=false) {
        run("apply-changes.sh")
    } else {
        print("Aborted.")
        return 0
    }
}
```

### Strategy: safe defaults

- Use `default=false` for destructive operations (deploy, restart, delete)
- Use `default=true` for informational confirmations (if you have any)
- In CI, set `SH2_YES=1` explicitly when you want to proceed

---

## 3. Sudo in Automation

The `sudo(...)` builtin has an `n=true` option for non-interactive mode.

### Without `n=true` (hangs in CI)

```sh2
# ❌ Will hang waiting for password in CI
sudo("systemctl", "restart", "nginx")
```

### With `n=true` (fails cleanly in CI)

```sh2
func main() {
    sudo("systemctl", "restart", "nginx", n=true, allow_fail=true)
    
    if status() != 0 {
        print_err("Error: sudo failed (no password-less sudo access?)")
        return 1
    }
    
    print("Service restarted successfully")
}
```

**Key insight:** `n=true` makes sudo fail immediately if it would need a password. This is what you want: **fail instead of hang**.

### CI configuration

Your CI runner needs passwordless sudo for the commands you use. In GitHub Actions:

```yaml
- name: Restart service
  run: |
    sudo -n systemctl restart nginx || echo "sudo not available"
```

Or configure sudoers for your CI user.

---

## 4. Predictable Failure Behavior

### Fail-fast by default

sh2 exits immediately on command failure:

```sh2
func main() {
    run("step-1")         # If this fails, script exits
    run("step-2")         # Never runs if step-1 failed
    print("Done")
}
```

This is ideal for CI: if something fails, stop immediately and show the exit code.

### Controlled failure: try, report, then exit

For more control, use `allow_fail=true` + `status()`:

```sh2
func main() {
    run("lint-code", allow_fail=true)
    
    if status() != 0 {
        print_err("Lint failed with exit code " & status())
        return 1
    }
    
    run("run-tests", allow_fail=true)
    
    if status() != 0 {
        print_err("Tests failed with exit code " & status())
        return 1
    }
    
    print("All checks passed")
}
```

### Collect failures pattern

Run multiple checks and report all failures at the end:

```sh2
func main() {
    let failures = 0
    
    print("=== Running lint ===")
    run("lint-code", allow_fail=true)
    if status() != 0 {
        print_err("FAIL: lint")
        set failures = failures + 1
    } else {
        print("PASS: lint")
    }
    
    print("=== Running tests ===")
    run("run-tests", allow_fail=true)
    if status() != 0 {
        print_err("FAIL: tests")
        set failures = failures + 1
    } else {
        print("PASS: tests")
    }
    
    print("=== Checking formatting ===")
    run("check-format", allow_fail=true)
    if status() != 0 {
        print_err("FAIL: format")
        set failures = failures + 1
    } else {
        print("PASS: format")
    }
    
    print("")
    if failures > 0 {
        print_err("Completed with " & failures & " failure(s)")
        return 1
    }
    
    print("All checks passed")
}
```

**Why this pattern?** In CI, you often want to see *all* failures, not just the first one. This runs everything and summarizes at the end.

---

## 5. Logging for CI Artifacts

CI systems let you upload artifacts (log files) after a run. sh2's `with redirect` makes this easy.

### Log to file AND console

```sh2
func ensure_log_dir() {
    run("mkdir", "-p", "logs", allow_fail=true)
}

func main() {
    ensure_log_dir()
    let log = "logs/ci-run.log"
    
    with redirect { 
        stdout: [file(log, append=true), inherit_stdout()],
        stderr: [file(log, append=true), inherit_stderr()]
    } {
        print("=== CI Run Started ===")
        print("Timestamp: " & capture(run("date", "+%Y-%m-%d %H:%M:%S")))
        
        run("step-1", allow_fail=true)
        if status() != 0 {
            print_err("step-1 failed")
        }
        
        run("step-2", allow_fail=true)
        if status() != 0 {
            print_err("step-2 failed")
        }
        
        print("=== CI Run Finished ===")
    }
    
    print("Log saved to: " & log)
}
```

**What this does:**
- `file(log, append=true)` — writes to log file
- `inherit_stdout()` / `inherit_stderr()` — also prints to console (visible in CI output)
- Both destinations receive the same content

### GitHub Actions artifact upload

```yaml
- name: Run checks
  run: ./ci-checks.sh

- name: Upload logs
  if: always()
  uses: actions/upload-artifact@v4
  with:
    name: ci-logs
    path: logs/
```

---

## 6. Two CI Recipes

### Recipe A: Compile only (lint-style check)

Use `sh2c` to verify sh2 syntax without running:

```yaml
# .github/workflows/lint.yml
jobs:
  lint-sh2:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Build sh2c
        run: cargo build --release -p sh2c
      
      - name: Check sh2 syntax
        run: |
          for f in tools/*.sh2; do
            echo "Checking $f..."
            ./target/release/sh2c --check "$f"
          done
```

**Expected outcomes:**
- Exit 0 if all files have valid syntax
- Exit non-zero if any file has syntax errors
- Errors print to stderr with file and line info

### Recipe B: Run script with environment overrides

Use `sh2do` to compile and run in one step:

```yaml
# .github/workflows/deploy.yml
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Build sh2 tools
        run: cargo build --release --workspace
      
      - name: Deploy application
        run: |
          SH2_YES=1 ./target/release/sh2do tools/deploy.sh2 -- production
```

Or compile first, then run:

```yaml
      - name: Compile deploy script
        run: ./target/release/sh2c tools/deploy.sh2 -o deploy.sh
      
      - name: Run deploy
        run: SH2_YES=1 ./deploy.sh production
```

**Expected outcomes:**
- `SH2_YES=1` makes `confirm(...)` return true
- Exit code reflects script success/failure
- Logs visible in CI output

---

## 7. Common CI Mistakes

### Mistake 1: Forgetting `default=` on `confirm`

```sh2
# ❌ Fails in CI: no default, no TTY
if confirm("Proceed?") { ... }

# ✅ Works in CI: has default
if confirm("Proceed?", default=false) { ... }
```

### Mistake 2: Missing `n=true` for sudo

```sh2
# ❌ Hangs in CI waiting for password
sudo("apt-get", "update")

# ✅ Fails cleanly if no passwordless sudo
sudo("apt-get", "update", n=true, allow_fail=true)
```

### Mistake 3: Losing the real error status

```sh2
# ❌ Status lost: print() runs between command and check
run("critical-step", allow_fail=true)
print("Step finished")
if status() != 0 { ... }  # status() is still correct here, but...

# ❌ Status lost: another command runs
run("critical-step", allow_fail=true)
run("log-step")  # This clobbers status()
if status() != 0 { ... }  # Now checking log-step's status!

# ✅ Check immediately or save
run("critical-step", allow_fail=true)
let exit_code = status()  # Save it
run("log-step")
if exit_code != 0 { ... }
```

### Mistake 4: Not checking status after allow_fail

```sh2
# ❌ Silently continues after failure
run("might-fail", allow_fail=true)
run("next-step")

# ✅ Explicit decision about what to do
run("might-fail", allow_fail=true)
if status() != 0 {
    print_err("might-fail failed, but continuing...")
}
run("next-step")
```

---

## 8. Checklist: CI-Ready sh2 Script

Before running in CI, verify:

- [ ] **All `confirm()` calls have `default=`** — prevents hanging
- [ ] **All `sudo()` calls have `n=true`** — prevents password prompts
- [ ] **Destructive operations use `default=false`** — safe by default
- [ ] **Critical commands check `status()`** — explicit error handling
- [ ] **Log directory is created** — `run("mkdir", "-p", "logs")`
- [ ] **Logs use fan-out** — `[file(...), inherit_stdout()]` for artifacts + console
- [ ] **Exit codes are meaningful** — `return 1` on failure, `return 0` on success
- [ ] **No raw `sh(...)` with interpolated input** — security risk
- [ ] **Script tested locally with `SH2_YES=0`** — simulates CI behavior
- [ ] **Script tested locally with `SH2_YES=1`** — simulates automated run

---

## 9. Quick Reference

### Environment variables

| Variable | Effect |
|----------|--------|
| `SH2_YES=1` | `confirm()` always returns `true` |
| `SH2_NO=1` | `confirm()` always returns `false` |

### Key options

| Option | Meaning |
|--------|---------|
| `confirm(..., default=false)` | Return `false` in non-interactive mode |
| `sudo(..., n=true)` | Fail instead of prompting for password |
| `run(..., allow_fail=true)` | Don't abort on failure |

### CI commands

```bash
# Check syntax only
sh2c --check script.sh2

# Compile to executable
sh2c script.sh2 -o script.sh

# Compile and run (with confirmation override)
SH2_YES=1 sh2do script.sh2 -- arg1 arg2
```

---

## Next Steps

You now know how to run sh2 safely in CI/CD pipelines.

### Related tutorials
- [Building a Real Tool](02-building-a-real-tool.md) — Complete tool example
- [Error Handling](03-error-handling-and-status.md) — Failure patterns

### Feature articles
- [confirm Helper](../articles/features/12-confirm-helper.md) — Full details
- [sudo Builtin](../articles/features/11-sudo-builtin.md) — All options
- [Logging and Redirects](../articles/features/16-logging-and-redirects.md) — Fan-out logging

### Release notes
- [v0.1.2 Release Notes](../releases/v0.1.2.md) — Job control, iterators, `which`
- [v0.1.1 Release Notes](../releases/v0.1.1.md) — `sudo`, `confirm`, semicolons

---

Happy automating! 🤖
