---
title: "Refactoring a Bash script into sh2: a step-by-step walkthrough"
description: "A practical migration tutorial: extract intent from a Bash script, reduce quoting hazards, and end with a reviewable sh2 tool."
---
<a href="https://github.com/siu-mak/sh2lang">
  <img src="../images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# Refactoring a Bash Script into sh2

This tutorial walks through migrating a real Bash script to sh2. You'll see the process step-by-step—not just the final result.

**What you'll learn:**
- Identify the intent behind messy Bash scripts
- Spot the hidden risks (quoting, splitting, status handling)
- Translate patterns into sh2 equivalents
- Know what sh2 solves and what still needs shell escape hatches

**Prerequisites:** Complete tutorials 01–06 first.

---

## 1. The Original Bash Script

Here's a typical "ops glue" script that restarts a service across multiple hosts:

```bash
#!/usr/bin/env bash
set -euo pipefail

# restart-across-hosts.sh
# Restarts a service on multiple hosts via SSH.

SERVICE="${1:-}"
HOSTS="${2:-hosts.txt}"
LOG="logs/restart-$(date +%Y%m%d-%H%M%S).log"

if [[ -z "$SERVICE" ]]; then
    echo "Usage: $0 <service> [hosts-file]" >&2
    exit 1
fi

if [[ ! -f "$HOSTS" ]]; then
    echo "Error: hosts file '$HOSTS' not found" >&2
    exit 1
fi

mkdir -p logs

echo "=== Restart job started at $(date) ===" | tee -a "$LOG"
echo "Service: $SERVICE" | tee -a "$LOG"
echo "Hosts file: $HOSTS" | tee -a "$LOG"
echo "" | tee -a "$LOG"

# Confirmation
read -r -p "Restart $SERVICE on all hosts in $HOSTS? [y/N] " answer
case "$answer" in
    [yY][eE][sS]|[yY]) ;;
    *) echo "Aborted."; exit 0 ;;
esac

failed=0
succeeded=0
total=0

while IFS= read -r host || [[ -n "$host" ]]; do
    # Skip empty lines and comments
    [[ -z "$host" || "$host" =~ ^# ]] && continue
    
    ((total++))
    echo "--- Restarting on $host ---" | tee -a "$LOG"
    
    if ssh -o ConnectTimeout=10 "$host" "sudo -n systemctl restart $SERVICE" 2>&1 | tee -a "$LOG"; then
        echo "OK: $host" | tee -a "$LOG"
        ((succeeded++))
    else
        echo "FAILED: $host (exit $?)" | tee -a "$LOG"
        ((failed++))
    fi
done < "$HOSTS"

echo "" | tee -a "$LOG"
echo "=== Summary ===" | tee -a "$LOG"
echo "Total: $total" | tee -a "$LOG"
echo "Succeeded: $succeeded" | tee -a "$LOG"
echo "Failed: $failed" | tee -a "$LOG"
echo "Log: $LOG"

if [[ $failed -gt 0 ]]; then
    exit 1
fi
```

---

## 2. What It's Trying to Do

In plain English:

1. **Accept a service name and optional hosts file**
2. **Validate inputs** (service required, hosts file must exist)
3. **Create a timestamped log file**
4. **Ask for confirmation** before proceeding
5. **Loop through hosts**, SSHing to each and restarting the service with sudo
6. **Track success/failure counts**
7. **Log everything** to file AND console
8. **Exit 1 if any host failed**

This is a common pattern: restart/deploy across a fleet with logging and confirmation.

---

## 3. The Hidden Risk List

| Risk | Where | What could go wrong |
|------|-------|---------------------|
| **Word splitting** | `$SERVICE`, `$host` | If either contains spaces, arguments break |
| **Command injection** | `systemctl restart $SERVICE` | If `$SERVICE` is `nginx; rm -rf /`, bad things happen |
| **$? clobbering** | `echo "FAILED: $host (exit $?)"` | The `tee` in the pipeline might clobber `$?` |
| **TTY required** | `read -r -p` | Hangs forever in CI/automation |
| **sudo prompts** | `sudo -n` | If sudo cache expired, might hang despite `-n` |
| **Quoting audit** | `"$host"`, `"$SERVICE"` | Are all expansions quoted? Hard to verify at a glance |
| **set -e exceptions** | Inside `while` and `if` | Failures in certain positions don't trigger exit |
| **IFS/read subtleties** | `while IFS= read -r` | Easy to get wrong; not obvious what it does |

A reviewer has to hold a lot of Bash trivia in their head to verify this script is safe.

---

## 4. Step 1: Move Confirmation into `confirm(...)`

**Bash (before):**
```bash
read -r -p "Restart $SERVICE on all hosts in $HOSTS? [y/N] " answer
case "$answer" in
    [yY][eE][sS]|[yY]) ;;
    *) echo "Aborted."; exit 0 ;;
esac
```

**sh2 (after):**
```sh2
if !confirm($"Restart {service} on all hosts?", default=false) {
    print("Aborted.")
    return 0
}
```

**What got better:**
- No case statement or regex matching
- `default=false` handles CI automatically (won't hang)
- `SH2_YES=1` works for automation
- Intent is obvious: "confirm, or abort"

---

## 5. Step 2: Make sudo Readable with `sudo(...)` Options

**Bash (before):**
```bash
ssh -o ConnectTimeout=10 "$host" "sudo -n systemctl restart $SERVICE"
```

**sh2 (after):**
```sh2
run("ssh", "-o", "ConnectTimeout=10", host,
    "sudo -n systemctl restart " & service,
    allow_fail=true)
```

**Note:** The command sent to SSH is still a string that runs on the remote host. sh2 can't protect the *remote* shell—only the local execution. For the local `run()` call, arguments are safely separated.

For local sudo (if you were running locally):

```sh2
sudo("systemctl", "restart", service, n=true, allow_fail=true)
```

**What got better:**
- Local argument safety
- `n=true` is explicit, not hidden in a string
- `allow_fail=true` is visible

**What's still tricky:**
- Remote commands via SSH are still strings; sh2 can't validate them

---

## 6. Step 3: Replace tee Plumbing with `with redirect`

**Bash (before):**
```bash
echo "Service: $SERVICE" | tee -a "$LOG"
```

Every line has `| tee -a "$LOG"`. It's noisy and easy to forget.

**sh2 (after):**
```sh2
with redirect { 
    stdout: [file(log, append=true), inherit_stdout()],
    stderr: [file(log, append=true), inherit_stderr()]
} {
    print("Service: " & service)
    # Everything in this block logs AND prints
}
```

**What got better:**
- Configure logging once, applies to entire block
- No `| tee -a` on every line
- Can't accidentally forget to pipe to log
- Fan-out is declarative: "file + console"

---

## 7. Step 4: Replace `$?` Patterns with `status()`

**Bash (before):**
```bash
if ssh ... "$host" "sudo ..."; then
    echo "OK: $host"
    ((succeeded++))
else
    echo "FAILED: $host (exit $?)"  # $? might be clobbered!
    ((failed++))
fi
```

**sh2 (after):**
```sh2
run("ssh", "-o", "ConnectTimeout=10", host, remote_cmd, allow_fail=true)

if status() == 0 {
    print("OK: " & host)
    set succeeded = succeeded + 1
} else {
    print_err("FAILED: " & host & " (exit " & status() & ")")
    set failed = failed + 1
}
```

**What got better:**
- `status()` isn't clobbered by `print()` or other statements
- Explicit `allow_fail=true` makes intent clear
- No confusion about what `$?` refers to

---

## 8. Step 5: Make Inputs Safe

**Bash (before):**
```bash
SERVICE="${1:-}"
HOSTS="${2:-hosts.txt}"
```

Then used directly in strings and commands.

**sh2 (after):**
```sh2
func main() {
    if argc() < 1 {
        usage()
        return 1
    }
    
    let service = arg(1)
    let hosts_file = "hosts.txt"
    if argc() >= 2 {
        set hosts_file = arg(2)
    }
    
    # Validate service name
    if contains(service, " ") {
        print_err("Error: service name cannot contain spaces")
        return 1
    }
    
    if !is_file(hosts_file) {
        print_err("Error: hosts file not found: " & hosts_file)
        return 1
    }
    
    # ...
}
```

**What got better:**
- Explicit validation before use
- No word splitting risk—`service` is always one value
- `is_file()` is clearer than `[[ -f "$HOSTS" ]]`

---

## 9. Final sh2 Version

```sh2
# restart-across-hosts.sh2
# Restarts a service on multiple hosts via SSH.

func usage() {
    print("Usage: restart-across-hosts.sh <service> [hosts-file]")
    print("")
    print("Restarts <service> on each host listed in hosts-file (default: hosts.txt)")
    print("")
    print("Environment:")
    print("  SH2_YES=1    Skip confirmation")
}

func validate_service(name) {
    if name == "" {
        print_err("Error: service name required")
        return false
    }
    if contains(name, " ") {
        print_err("Error: service name cannot contain spaces")
        return false
    }
    if contains(name, ";") {
        print_err("Error: service name cannot contain ';'")
        return false
    }
    return true
}

func main() {
    if argc() < 1 {
        usage()
        return 1
    }
    
    if arg(1) == "--help" {
        usage()
        return 0
    }
    
    let service = arg(1)
    let hosts_file = "hosts.txt"
    if argc() >= 2 {
        set hosts_file = arg(2)
    }
    
    # Validate inputs
    if !validate_service(service) {
        return 1
    }
    
    if !is_file(hosts_file) {
        print_err("Error: hosts file not found: " & hosts_file)
        return 1
    }
    
    # Create log directory and file
    run("mkdir", "-p", "logs", allow_fail=true)
    let timestamp = trim(capture(run("date", "+%Y%m%d-%H%M%S")))
    let log = "logs/restart-" & timestamp & ".log"
    
    # Confirmation
    if !confirm($"Restart {service} on all hosts in {hosts_file}?", default=false) {
        print("Aborted.")
        return 0
    }
    
    # Counters
    let failed = 0
    let succeeded = 0
    let total = 0
    
    # Read hosts
    let hosts_content = read_file(hosts_file)
    
    # Work inside redirect block for logging
    with redirect { 
        stdout: [file(log, append=true), inherit_stdout()],
        stderr: [file(log, append=true), inherit_stderr()]
    } {
        print("=== Restart job started ===")
        print("Timestamp: " & timestamp)
        print("Service: " & service)
        print("Hosts file: " & hosts_file)
        print("")
        
        for host in lines(hosts_content) {
            # Skip empty lines and comments
            if host == "" {
                continue
            }
            if starts_with(host, "#") {
                continue
            }
            
            set total = total + 1
            print("--- Restarting on " & host & " ---")
            
            # Build remote command (still a string for SSH)
            let remote_cmd = "sudo -n systemctl restart " & service
            
            run("ssh", "-o", "ConnectTimeout=10", host, remote_cmd, allow_fail=true)
            
            if status() == 0 {
                print("OK: " & host)
                set succeeded = succeeded + 1
            } else {
                print_err("FAILED: " & host & " (exit " & status() & ")")
                set failed = failed + 1
            }
        }
        
        print("")
        print("=== Summary ===")
        print("Total: " & total)
        print("Succeeded: " & succeeded)
        print("Failed: " & failed)
    }
    
    print("Log: " & log)
    
    if failed > 0 {
        return 1
    }
    
    return 0
}
```

---

## 10. Test Drive Commands

Compile the script:

```bash
sh2c restart-across-hosts.sh2 -o restart-across-hosts.sh
```

### Test 1: Help

```bash
./restart-across-hosts.sh --help
```

### Test 2: Missing argument

```bash
./restart-across-hosts.sh
# Expected: usage message, exit 1
```

### Test 3: Dry run (decline confirmation)

```bash
./restart-across-hosts.sh nginx hosts.txt
# At prompt, type 'n'
# Expected: "Aborted.", exit 0
```

### Test 4: CI mode (auto-yes)

```bash
SH2_YES=1 ./restart-across-hosts.sh nginx hosts.txt
# Expected: runs without prompting
```

### Test 5: Invalid service name

```bash
./restart-across-hosts.sh "nginx; whoami"
# Expected: "Error: service name cannot contain ';'", exit 1
```

---

## 11. What Changed: Before/After Comparison

| Aspect | Bash | sh2 |
|--------|------|-----|
| **Confirmation** | `read -p` + case statement | `confirm(..., default=false)` |
| **CI behavior** | Hangs waiting for input | Returns false (safe default) |
| **Logging** | `\| tee -a "$LOG"` on every line | `with redirect { ... }` once |
| **Status checking** | `$?` (easily clobbered) | `status()` (stable) |
| **Input validation** | `[[ -z "$VAR" ]]` | `if var == ""` |
| **File checks** | `[[ -f "$FILE" ]]` | `is_file(file)` |
| **Loop syntax** | `while IFS= read -r` | `for host in lines(content)` |
| **Quoting** | Must quote `"$var"` everywhere | Not needed; always safe |
| **Reviewability** | Need Bash expertise | More readable to non-experts |

---

## 12. Honest: What Still Isn't Solved

### Remote commands via SSH

sh2 can't protect what happens inside the SSH session. The remote command is still a string:

```sh2
let remote_cmd = "sudo -n systemctl restart " & service
run("ssh", host, remote_cmd)
```

If `service` contains shell metacharacters, the remote shell might interpret them. The validation function helps, but it's defense-in-depth.

### Process substitution

Bash patterns like `diff <(cmd1) <(cmd2)` have no sh2 equivalent. You'd need temp files.

### Job control

Background processes (`&`), `wait`, `fg`/`bg` aren't part of sh2.

### Complex awk/sed

For heavy text processing, you'll still call `awk` or `sed`. sh2 just makes the *calling* safer.

### Associative arrays

The Bash script used simple counters. If it had associative arrays, sh2's maps (Bash target only) would work, but the syntax differs.

---

## 13. Summary: When to Refactor to sh2

| Refactor to sh2 if... | Keep in Bash if... |
|-----------------------|-------------------|
| Script will be reviewed by others | It's a quick personal tool |
| Runs in CI/CD (needs predictable exits) | Uses heavy process substitution |
| Has `sudo`, confirmations, or logging | Is primarily awk/sed text transforms |
| Uses external commands with user input | Uses job control (`&`, `wait`) |
| Quoting bugs have bitten you before | You're confident in the Bash version |

---

## Next Steps

You've now seen a full refactoring walkthrough.

### Related tutorials
- [Building a Real Tool](02-building-a-real-tool.md) — Similar patterns, different script
- [CI and Automation](06-ci-and-automation.md) — More on non-interactive behavior

### Feature articles
- [confirm Helper](../articles/features/12-confirm-helper.md) — Full details
- [sudo Builtin](../articles/features/11-sudo-builtin.md) — All options
- [Logging and Redirects](../articles/features/16-logging-and-redirects.md) — Fan-out logging
- [Error Handling](../articles/features/15-error-handling.md) — allow_fail and status()
- [No Implicit Expansion](../articles/features/13-no-implicit-expansion.md) — Why strings are safe

---

Happy refactoring! 🔧
