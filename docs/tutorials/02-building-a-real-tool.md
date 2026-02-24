---
title: "Building a real sh2 tool: restart a service with safety rails"
description: "Turn a working snippet into a reviewable .sh2 tool: args, validation, logging, sudo, confirm, and predictable failure behavior."
---
<a href="https://github.com/siu-mak/sh2lang">
  <img src="../images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# Building a Real sh2 Tool

This tutorial walks you through building a practical ops tool from scratch. You'll learn why sh2 is safer than Bash for "glue scripts" that run with elevated privileges.

**What you'll build:** A service restart tool with argument parsing, validation, confirmation prompts, sudo, logging, and proper error handling.

**Prerequisites:** Complete [Tutorial 01: Getting Started](01-getting-started.md) first.

---

## 1. What We're Building

A tool called `restart-service.sh` with this interface:

```
Usage: restart-service.sh <service> [--dry-run] [--yes] [--help]

Arguments:
  <service>     Name of the systemd service to restart (required)

Options:
  --help        Show this message and exit
  --dry-run     Print what would run without executing
  --yes         Skip confirmation prompt
```

**Example runs:**

```bash
# Normal use (will prompt for confirmation)
./restart-service.sh nginx

# Skip confirmation
./restart-service.sh nginx --yes

# CI mode (non-interactive, uses default=false unless SH2_YES=1)
SH2_YES=1 ./restart-service.sh nginx

# See what would happen
./restart-service.sh nginx --dry-run

# Get help
./restart-service.sh --help
```

---

## 2. Start From a sh2do Snippet

Before writing a full script, prototype with `sh2do`:

```bash
sh2do '
let svc = "nginx"
run("systemctl", "status", svc, allow_fail=true)
print("Status code: " & status())
'
```

This quick test confirms:
- `run(...)` passes arguments safely (no injection risk)
- `allow_fail=true` prevents script abort on failure
- `status()` captures the exit code

**Why start here?** sh2do lets you experiment without file management. Once the logic works, promote to a `.sh2` file.

---

## 3. Promote to a .sh2 File

Create `tools/restart-service.sh2`:

```sh2
func main() {
    let svc = "nginx"
    run("systemctl", "status", svc, allow_fail=true)
    print("Status code: " & status())
}
```

Compile and run:

```bash
sh2c tools/restart-service.sh2 -o tools/restart-service.sh
./tools/restart-service.sh
```

**Why is this better?**
- Version control friendly
- Reviewable before deployment
- Reusable across environments

---

## 4. Add Argument Parsing

Replace the hardcoded service name with argument handling:

```sh2
func usage() {
    print("Usage: restart-service.sh <service> [--dry-run] [--yes] [--help]")
    print("")
    print("Arguments:")
    print("  <service>     Name of the systemd service to restart")
    print("")
    print("Options:")
    print("  --help        Show this message and exit")
    print("  --dry-run     Print what would run without executing")
    print("  --yes         Skip confirmation prompt")
}

func main() {
    # Parse flags
    let dry_run = false
    let skip_confirm = false
    let service = ""
    
    for arg in args() {
        if arg == "--help" {
            usage()
            return 0
        }
        if arg == "--dry-run" {
            set dry_run = true
        } else if arg == "--yes" {
            set skip_confirm = true
        } else {
            # First non-flag argument is the service name
            if service == "" {
                set service = arg
            }
        }
    }
    
    # Require service name
    if service == "" {
        usage()
        print_err("Error: <service> is required")
        return 1
    }
    
    print("Service: " & service)
    print("Dry run: " & dry_run)
    print("Skip confirm: " & skip_confirm)
}
```

**Test it:**

```bash
sh2c tools/restart-service.sh2 -o tools/restart-service.sh
./tools/restart-service.sh --help
./tools/restart-service.sh nginx --dry-run
./tools/restart-service.sh
```

---

## 5. Add Validation + Clear Errors

Reject invalid service names before doing anything else:

```sh2
func validate_service(name) {
    # Empty check
    if name == "" {
        print_err("Error: service name cannot be empty")
        return false
    }
    
    # Reject spaces (common mistake)
    if contains(name, " ") {
        print_err("Error: service name cannot contain spaces")
        return false
    }
    
    # Reject slashes (path injection attempt)
    if contains(name, "/") {
        print_err("Error: service name cannot contain '/'")
        return false
    }
    
    return true
}

func main() {
    # ... (argument parsing from above) ...
    
    # Validate
    if !validate_service(service) {
        return 1
    }
    
    # ... rest of the tool ...
}
```

**Why this matters:**

In Bash, if someone passes `"nginx; rm -rf /"` as the service name and you accidentally use it in an unquoted context, you get code execution. In sh2:

```sh2
run("systemctl", "restart", service)
```

This passes `service` as a **single argument** to `systemctl`. The semicolon and everything after it are just characters in the argument string—never interpreted as shell commands.

**sh2's strict literal model means passing weird input can cause a command to fail, but never causes injection.**

---

## 6. Add Logging with `with redirect`

Create a log of all operations:

```sh2
func ensure_log_dir() {
    run("mkdir", "-p", "logs", allow_fail=true)
}

func main() {
    # ... (argument parsing and validation) ...
    
    ensure_log_dir()
    let log = "logs/restart-service.log"
    
    with redirect { 
        stdout: [file(log, append=true), inherit_stdout()],
        stderr: [file(log, append=true), inherit_stderr()]
    } {
        print("---")
        print("Timestamp: " & capture(run("date", "+%Y-%m-%d %H:%M:%S")))
        print("Service: " & service)
        
        # ... rest of the tool runs inside this block ...
    }
}
```

**What this does:**
- `file(log, append=true)` — appends to the log file
- `inherit_stdout()` / `inherit_stderr()` — also prints to terminal
- Both outputs go to both destinations (fan-out)

**The block is scoped:** After the `with redirect { ... }` block ends, stdout/stderr return to normal.

---

## 7. Add sudo(...) + Status Checks

Now implement the actual restart logic with proper privilege handling:

```sh2
func restart_service(service, dry_run, skip_confirm) {
    # Step 1: Check current status
    print("Checking current status...")
    run("systemctl", "status", service, allow_fail=true)
    let initial_status = status()
    print("Current status code: " & initial_status)
    
    # Step 2: Dry run exit
    if dry_run {
        print("[DRY RUN] Would execute: sudo systemctl restart " & service)
        return 0
    }
    
    # Step 3: Confirmation
    if !skip_confirm {
        if !confirm($"Restart {service}?", default=false) {
            print("Aborted by user.")
            return 0
        }
    }
    
    # Step 4: Restart with sudo
    print("Restarting " & service & "...")
    sudo("systemctl", "restart", service, n=true, allow_fail=true)
    
    if status() != 0 {
        print_err("Error: restart failed with exit code " & status())
        return 1
    }
    
    # Step 5: Verify new status
    print("Verifying new status...")
    run("systemctl", "status", service, allow_fail=true)
    
    if status() == 0 {
        print("OK: " & service & " restarted successfully")
    } else {
        print_err("FAILED: " & service & " is not running after restart")
        return 1
    }
    
    return 0
}
```

**Key v0.1.1 features used:**

| Feature | Usage | Why |
|---------|-------|-----|
| `sudo(..., n=true)` | Non-interactive sudo | Fails cleanly in CI without hanging |
| `sudo(..., allow_fail=true)` | Don't abort on failure | Handle the error ourselves |
| `status()` | Check exit code | Know exactly what happened |
| `confirm(..., default=false)` | Safe CI default | Non-interactive = no restart |
| `$"..."` interpolation | `$"Restart {service}?"` | Clean, readable prompts |

---

## 8. Confirmation Behavior for CI

The `confirm(...)` helper has three modes:

| Scenario | Behavior |
|----------|----------|
| Interactive terminal | Prompts user, waits for y/n |
| `SH2_YES=1` or `--yes` | Returns true immediately |
| `SH2_NO=1` | Returns false immediately |
| Non-interactive + `default=false` | Returns false |

**This is critical for CI/CD:**

```bash
# In CI pipeline (non-interactive)
SH2_YES=1 ./restart-service.sh nginx

# Or using the --yes flag
./restart-service.sh nginx --yes
```

Without `SH2_YES=1` or `--yes`, the tool safely does nothing in non-interactive mode.

---

## 9. Complete Tool

Here's the full `tools/restart-service.sh2`:

```sh2
# tools/restart-service.sh2
# Safely restart a systemd service with logging and confirmation.

func usage() {
    print("Usage: restart-service.sh <service> [--dry-run] [--yes] [--help]")
    print("")
    print("Arguments:")
    print("  <service>     Name of the systemd service to restart")
    print("")
    print("Options:")
    print("  --help        Show this message and exit")
    print("  --dry-run     Print what would run without executing")
    print("  --yes         Skip confirmation prompt")
    print("")
    print("Environment:")
    print("  SH2_YES=1     Skip confirmation (same as --yes)")
    print("  SH2_NO=1      Force abort at confirmation")
}

func validate_service(name) {
    if name == "" {
        print_err("Error: service name cannot be empty")
        return false
    }
    if contains(name, " ") {
        print_err("Error: service name cannot contain spaces")
        return false
    }
    if contains(name, "/") {
        print_err("Error: service name cannot contain '/'")
        return false
    }
    return true
}

func ensure_log_dir() {
    run("mkdir", "-p", "logs", allow_fail=true)
}

func restart_service(service, dry_run, skip_confirm) {
    # Step 1: Check current status
    print("Checking current status...")
    run("systemctl", "status", service, allow_fail=true)
    let initial_status = status()
    print("Current status code: " & initial_status)
    
    # Step 2: Dry run exit
    if dry_run {
        print("[DRY RUN] Would execute: sudo systemctl restart " & service)
        return 0
    }
    
    # Step 3: Confirmation
    if !skip_confirm {
        if !confirm($"Restart {service}?", default=false) {
            print("Aborted by user.")
            return 0
        }
    }
    
    # Step 4: Restart with sudo (n=true for non-interactive)
    print("Restarting " & service & "...")
    sudo("systemctl", "restart", service, n=true, allow_fail=true)
    
    if status() != 0 {
        print_err("Error: restart failed with exit code " & status())
        return 1
    }
    
    # Step 5: Verify new status
    print("Verifying new status...")
    run("systemctl", "status", service, allow_fail=true)
    
    if status() == 0 {
        print("OK: " & service & " restarted successfully")
    } else {
        print_err("FAILED: " & service & " is not running after restart")
        return 1
    }
    
    return 0
}

func main() {
    # Parse arguments
    let dry_run = false
    let skip_confirm = false
    let service = ""
    
    for arg in args() {
        if arg == "--help" {
            usage()
            return 0
        }
        if arg == "--dry-run" {
            set dry_run = true
        } else if arg == "--yes" {
            set skip_confirm = true
        } else {
            if service == "" {
                set service = arg
            }
        }
    }
    
    # Require service
    if service == "" {
        usage()
        print_err("Error: <service> is required")
        return 1
    }
    
    # Validate
    if !validate_service(service) {
        return 1
    }
    
    # Set up logging
    ensure_log_dir()
    let log = "logs/restart-service.log"
    
    with redirect { 
        stdout: [file(log, append=true), inherit_stdout()],
        stderr: [file(log, append=true), inherit_stderr()]
    } {
        print("---")
        print("Timestamp: " & capture(run("date", "+%Y-%m-%d %H:%M:%S")))
        print("Service: " & service); print("Dry run: " & dry_run)
        
        let result = restart_service(service, dry_run, skip_confirm)
        return result
    }
}
```

---

## 10. Test Drive

Compile the tool:

```bash
sh2c tools/restart-service.sh2 -o tools/restart-service.sh
```

### Test 1: Help

```bash
./tools/restart-service.sh --help
```

**Expected:** Usage message, exit 0.

### Test 2: Dry run

```bash
./tools/restart-service.sh nginx --dry-run
```

**Expected:** Shows current status, prints "[DRY RUN] Would execute...", exit 0.

### Test 3: Missing argument

```bash
./tools/restart-service.sh
```

**Expected:** Usage message, "Error: <service> is required", exit 1.

### Test 4: Invalid service name

```bash
./tools/restart-service.sh "nginx; whoami"
```

**Expected:** "Error: service name cannot contain spaces", exit 1.

---

## 11. Common Mistakes in Bash and How sh2 Avoids Them

| Bash Mistake | What Goes Wrong | sh2 Solution |
|--------------|-----------------|--------------|
| `systemctl restart $service` | Word splitting if service has spaces | `run("systemctl", "restart", service)` — always one argument |
| `sudo systemctl restart $1` | Injection if `$1` contains shell metacharacters | `sudo("systemctl", "restart", arg(1))` — arguments are never parsed as shell |
| `echo "Restarting $service..." >> log` | Expansion if service contains `$` | `print("Restarting " & service)` — strict literals |
| `read -p "Continue?" && ...` | Hangs in CI | `confirm(..., default=false)` — safe non-interactive default |
| `set -e` + `cmd || true` | Confusing status capture | `allow_fail=true` + `status()` — explicit and clear |
| cd into directory, forget to cd back | Affects subsequent commands | `with cwd(...)` — scoped, auto-reverts |

---

## 12. Next Steps

You now know how to build production-quality sh2 tools. Here's where to go next:

### Reference docs
- [Language Reference](../language.md) — Full syntax and semantics
- [sh2do Documentation](../sh2do.md) — Snippet runner details
- [v0.1.2 Release Notes](../releases/v0.1.2.md) — Job control, iterators, `which`
- [v0.1.1 Release Notes](../releases/v0.1.1.md) — What's new

### Feature deep-dives
- [sudo Builtin](../articles/features/11-sudo-builtin.md) — All options explained
- [confirm Helper](../articles/features/12-confirm-helper.md) — Interactive prompts and CI behavior
- [Error Handling](../articles/features/15-error-handling.md) — allow_fail, status(), try/catch
- [Logging and Redirects](../articles/features/16-logging-and-redirects.md) — Fan-out, file logging
- [No Implicit Expansion](../articles/features/13-no-implicit-expansion.md) — Why strings are safe

---

Happy building! 🛠️
