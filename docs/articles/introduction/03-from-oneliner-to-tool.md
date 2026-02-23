---
title: "From one-liner to tool: when a snippet deserves structure"
description: "A practical guide to knowing when Bash is enough, when sh2do helps, and when to commit a real .sh2 script."
---
<a href="https://github.com/siu-mak/sh2lang">
  <img src="../../images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# From one-liner to tool: when a snippet deserves structure

## The story

You run a one-liner.

```bash
find . -name "*.bak" -mtime +30 -delete
```

It works. You paste it into Slack. Someone asks you to add a confirmation. You slap a `read -p` in front. Now it's two lines.

A month later, you're asked to make it run in CI. But CI has no stdin. The script hangs. You add a flag. Then someone runs it with `sudo` and it deletes the wrong thing.

Now you're scared to touch it. Everyone uses it, but nobody understands it.

**This article is about recognizing when a snippet has outgrown its format—and what to do next.**

---

## The decision ladder

Not everything needs to be a "production tool." Here's a practical ladder:

| Level | Format | When it fits | When it fails |
|-------|--------|--------------|---------------|
| **1** | Bash one-liner | Throwaway commands, interactive exploration | Any quoting complexity, needs to run in CI, has side-effects |
| **2** | Bash script (quick) | Slightly longer, still ad-hoc | Grows past 20 lines, has sudo/rm/systemctl, needs review |
| **3** | `sh2do` snippet | Structured one-liner, CI-safe, explicit error handling | More than 3-4 statements, needs functions, needs version control |
| **4** | `.sh2` script (sh2c) | Reviewable tool, importable functions, proper argument parsing | You just needed a quick grep |

**The goal:** pick the lowest level that doesn't bite you.

---

## Case Study: The Backup Cleaner

Let's follow a single script as it evolves from a quick command to a production tool.

### Level 1: The Bash one-liner

You just need to delete some files.

```bash
read -p "Delete old backups? [y/N] " a && [[ $a =~ ^[Yy] ]] && find . -name "*.bak" -delete
```

**Problem:** This hangs in CI (no stdin). It has no default. Quoting is fragile if you change the find command.

### Level 3: The structured snippet (sh2do)

You want it to be safe for automation.

```bash
sh2do 'if confirm("Delete old backups?", default=false) { run("find", ".", "-name", "*.bak", "-delete") }'
```

**What you gained:**
- `default=false` makes non-interactive runs safe (returns false automatically).
- `SH2_YES=1` allows CI to override the prompt safely.
- Arguments are passed safely to `find` without word-splitting.

### Level 4: The committed tool (.sh2)

You want to share this with the team, add logging, and maybe handle errors gracefully.

```sh2
func main() {
    print("Scanning for old backups...")
    
    # Capture files first instead of just deleting
    let files = capture(run("find", ".", "-name", "*.bak", "-mtime", "+30", "-print"), allow_fail=true)
    
    if status() != 0 {
        print_err("Error scanning directory.")
        return 1
    }
    
    if files == "" {
        print("No backups found.")
        return 0
    }
    
    if confirm("Delete found backups?", default=false) {
        # Loop explicitly for transparency or logging
        for f in lines(files) {
            run("rm", f)
            print("Deleted: " & f)
        }
    } else {
        print("Aborted.")
    }
}
```

**What you gained:**
- **Logic:** You can check status codes and empty results before acting.
- **Observability:** You print what you delete.
- **Reviewability:** The team can read this without parsing `&&` chains.

---

## Other common patterns

This case study focused on confirmation and file iteration. For other patterns like **sudo**, **strict literals**, or **error handling**, see the catalog:

- [Running commands as root (sudo)](02-bash-oneliners-ironed-out.md#2-running-a-command-as-root-with-flags)
- [Capturing errors without aborting](02-bash-oneliners-ironed-out.md#3-error-handling-based-on-exit-status)
- [Strict literals (avoiding `${...}` bugs)](../features/13-no-implicit-expansion.md)
- [File iteration and cleanup](02-bash-oneliners-ironed-out.md#7-file-iteration-line-by-line-processing)

For the full list of one-liner patterns, see [Bash one-liners ironed out](02-bash-oneliners-ironed-out.md).

---

## When to commit: the checklist

Promote a snippet to a committed `.sh2` script when **any** of these apply:

- [ ] **Needs review** — someone else should see this before it runs in prod
- [ ] **Has side effects** — deletes files, restarts services, modifies config
- [ ] **Will be reused** — pasted into more than one Slack thread already
- [ ] **Will run in CI** — needs to work without a tty
- [ ] **Contains sudo / rm / systemctl** — privileged or destructive
- [ ] **Needs logging** — you want a record of what happened
- [ ] **Needs usage text** — people ask "how do I use this?"
- [ ] **Needs argument validation** — more than one positional argument

If you check 2 or more boxes, it's time to commit.

---

## Starter template: a real `.sh2` tool

Here's a minimal but complete template for a committed tool:

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
    
    # Find files to delete
    let files = capture(run("find", dir, "-name", "*.bak", "-mtime", "+30"), allow_fail=true)
    if status() != 0 {
        print_err("Error: find command failed")
        return 1
    }
    
    let count = 0
    for f in lines(files) {
        set count = count + 1
    }
    
    if count == 0 {
        print("No backup files older than 30 days found.")
        return 0
    }
    
    print($"Found {count} backups.")
    
    # Confirm before deletion
    if !confirm($"Delete {count} backups?", default=false) {
        print("Aborted.")
        return 0
    }
    
    # Perform deletion (with sudo if needed for protected files)
    for f in lines(files) {
        run("rm", "--", f, allow_fail=true)
        if status() != 0 {
            print_err($"Warning: could not delete {f}")
        }
    }
    
    print("Done.")
}
```

**What this template includes:**
- `usage()` function with clear help text
- Argument parsing with `argc()` and `arg(n)`
- `--help` flag handling
- Input validation with `is_dir()`
- `allow_fail=true` + `status()` for error handling
- `confirm(..., default=false)` for safe CI behavior
- `$"..."` interpolation for dynamic messages
- Clear exit codes (0 = success, 1 = error)

Compile and install:

```bash
sh2c -o tools/cleanup-backups.sh tools/cleanup-backups.sh2
./tools/cleanup-backups.sh /var/backups
```

---

## Summary: the right tool for the job

| Situation | Use |
|-----------|-----|
| Quick exploration, throwaway | Bash one-liner |
| Needs quoting safety, runs in CI | `sh2do 'snippet'` |
| Reusable, reviewed, committed | `.sh2` script + `sh2c` |
| Pure text pipelines (grep/awk/sort) | Keep it in Bash |

**The goal isn't to rewrite everything.** It's to recognize when a snippet has outgrown the format where you wrote it—and move it to a format where it can be understood, reviewed, and trusted.

---

# Docs

The GitHub repo is here:  
**[https://github.com/siu-mak/sh2lang](https://github.com/siu-mak/sh2lang)**

## Further Documentation

- [`docs/language.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/language.md) — full language reference (syntax + semantics)
- [`docs/sh2do.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/sh2do.md) — sh2do CLI documentation
- `tests/` — fixtures and integration tests (acts as an executable spec)

## Versions

- [v0.1.2](https://github.com/siu-mak/sh2lang/blob/main/docs/releases/v0.1.2.md) — Adds for-loop ranges, job control (`spawn`/`wait`), iterators, `which()`.
- [v0.1.1](https://github.com/siu-mak/sh2lang/blob/main/docs/releases/v0.1.1.md) — Adds `sudo(...)`, `confirm(...)`, semicolon separators.
- [v0.1.0](https://github.com/siu-mak/sh2lang/blob/main/docs/releases/v0.1.0.md) — First public release of sh2.
