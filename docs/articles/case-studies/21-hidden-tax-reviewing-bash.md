---
title: "The hidden tax of reviewing Bash scripts"
description: 'A story-driven case study: why "normal" Bash scripts are costly to audit, and how sh2 makes intent visible.'
---
<a href="https://github.com/siu-mak/sh2lang">
  <img src="../../images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# The hidden tax of reviewing Bash scripts

Last week, a colleague asked me to review an install script before it went into production. "It's pretty simple," she said. "Just installs some packages, sets up nginx, and logs everything."

The script was 80 lines. I should have been able to skim it in five minutes.

I spent forty-five.

Not because the script was badly written—it wasn't. It was exactly the kind of "normal" Bash you'd find in any ops team. But every few lines, I had to stop and simulate shell semantics in my head: Does this quote correctly? Does that `tee` hide a failure? Will this `cd` leak into later commands? Can `sudo` prompt in CI?

That's the hidden tax of reviewing Bash scripts: the language does things you have to actively think about, on every line, forever.

---

## The script

Here's a simplified excerpt of what I was reviewing:

```bash
#!/bin/bash
set -euo pipefail

LOG="/var/log/install-$(date +%Y%m%d).log"
exec > >(tee -a "$LOG") 2>&1

install_nginx() {
    echo "Installing nginx..."
    cd /tmp
    
    # Clean up old downloads
    rm -rf nginx-*.tar.gz
    
    sudo apt-get update
    sudo apt-get install -y nginx
    
    # Copy config files
    for cfg in /opt/configs/*.conf; do
        sudo cp "$cfg" /etc/nginx/conf.d/
    done
    
    sudo systemctl restart nginx
    
    echo "Nginx installed successfully."
}

install_app() {
    echo "Installing app..."
    
    while read -r pkg; do
        sudo apt-get install -y "$pkg"
    done < /opt/packages.txt
    
    sudo docker-compose -f /opt/app/docker-compose.yml up -d
}

install_nginx
install_app
echo "Done."
```

Looks reasonable, right? Let's go through what I actually had to verify.

---

## The reviewer's mental checklist

### 1. Does `exec > >(tee ...)` break something later?

Process substitution is global. It affects every command after it—including `read`, interactive prompts, and `sudo`. If `sudo` needs a password, the TTY might be confused.

**Verdict:** I had to check whether this script ever runs interactively.

### 2. Does `rm -rf nginx-*.tar.gz` expand correctly?

If the glob matches nothing, behavior depends on shell settings. With `set -e`, this might fail. With `shopt -s nullglob`, it might silently do nothing. Without either, it tries to delete a literal file named `nginx-*.tar.gz`.

**Verdict:** I had to check nullglob settings (not set), then check if the glob matching nothing is a failure condition.

### 3. Does `rm -rf` work safely with filenames starting with `-`?

No `--` before the glob. If a file is named `-rf`, chaos ensues.

**Verdict:** Unlikely, but I had to think about it.

### 4. Does `cd /tmp` leak?

Yes. After `install_nginx` returns, cwd is still `/tmp`. The `install_app` function runs in `/tmp`, not the original directory.

**Verdict:** Side effect. Probably fine, but I had to trace it.

### 5. Does the `for cfg in /opt/configs/*.conf` loop handle spaces?

It does—`"$cfg"` is quoted. But I had to check. If someone later removes the quotes, it breaks silently on files with spaces.

**Verdict:** Safe, but fragile.

### 6. Does `sudo apt-get` prompt for a password in CI?

If credentials aren't cached, `sudo` will prompt. In CI, that hangs forever.

**Verdict:** Need to check CI environment, or add `-n` flag.

### 7. Does `tee` hide the exit code?

With `pipefail`, the pipeline returns the first non-zero exit. But `exec > >(tee ...)` isn't a pipeline—it's process substitution. The exit code of `tee` is lost.

**Verdict:** If `tee` fails (disk full?), we won't know.

### 8. Does `while read ... < file` work correctly?

It does here. But `read -r` is needed to avoid backslash interpretation, and the redirect must come after the `done`, not after `read`. I had to check the syntax.

**Verdict:** Correct, but not obvious.

### 9. Does `${...}` ever expand unexpectedly?

The `date` command uses `+%Y%m%d`. If there were a `${...}` in a string, it might conflict with shell variable expansion. I had to scan for that.

**Verdict:** No conflicts here.

---

## Nine questions for 80 lines of code

That's the hidden tax. Every question required me to simulate Bash in my head. Not because the script was bad, but because Bash requires it.

---

## The same script in sh2

Here's how I'd rewrite this in sh2:

```sh2
func install_nginx() {
    print("Installing nginx...")
    
    with cwd("/tmp") {
        # sh(...) because: glob expansion not yet supported in cleanup
        sh("rm -rf nginx-*.tar.gz")
    }
    # cwd returns to original after block
    
    sudo("apt-get", "update", n=true)
    sudo("apt-get", "install", "-y", "nginx", n=true)
    
    # Copy config files
    # sh(...) because: glob expansion for file list
    let files = capture(sh("find /opt/configs -maxdepth 1 -name '*.conf'"), allow_fail=true)
    if status() == 0 {
        for cfg_line in lines(files) {
            sudo("cp", cfg_line, "/etc/nginx/conf.d/", n=true)
        }
    }
    
    sudo("systemctl", "restart", "nginx", n=true)
    
    print("Nginx installed successfully.")
}

func install_app() {
    print("Installing app...")
    
    let packages = capture(run("cat", "/opt/packages.txt"))
    for pkg in lines(packages) {
        sudo("apt-get", "install", "-y", pkg, n=true)
    }
    
    sudo("docker-compose", "-f", "/opt/app/docker-compose.yml", "up", "-d", n=true)
}

func main() {
    let log_file = "/var/log/install.log"
    
    with redirect { stdout: [file(log_file, append=true), inherit_stdout()], stderr: to_stdout() } {
        if confirm("Proceed with installation?", default=false) {
            install_nginx()
            install_app()
            print("Done.")
        } else {
            print("Installation cancelled.")
        }
    }
}
```

---

## What changed?

### 1. Scoped cwd

```sh2
with cwd("/tmp") {
    sh("rm -rf nginx-*.tar.gz")
}
// cwd is back to original
```

- The directory change applies only inside the block.
- No leakage into later functions.

### 2. Scoped logging

```sh2
with redirect { stdout: [file(log_file, append=true), inherit_stdout()], stderr: to_stdout() } {
    ...
}
```

- Logging applies only inside the block.
- Output goes to file AND console (tee equivalent).
- No global `exec` side effects.

### 3. Named sudo options

```sh2
sudo("apt-get", "update", n=true)
```

- `n=true` means non-interactive (no password prompt).
- Reviewers see the intent without decoding `-n`.
- The `--` separator is inserted automatically.

### 4. Confirmation guard

```sh2
if confirm("Proceed with installation?", default=false) {
    ...
}
```

- Dangerous operation requires explicit confirmation.
- `default=false` means CI/automation skips it safely.
- Override with `SH2_YES=1` for automated runs.

### 5. Explicit failure handling

```sh2
let files = capture(sh("ls /opt/configs/*.conf 2>/dev/null"), allow_fail=true)
if status() == 0 {
    ...
}
```

- `allow_fail=true` prevents script abort.
- `status()` is checked explicitly.
- No silent failures.

---

## Side-by-side: reviewability comparison

| Aspect | Bash | sh2 |
|--------|------|-----|
| **cwd leakage** | `cd` affects all later commands | `with cwd` is scoped |
| **logging scope** | `exec > >(tee)` is global | `with redirect` is scoped |
| **sudo flags** | `-n -u root` requires decoding | `n=true, user="root"` is readable |
| **failure behavior** | `set -e` has exceptions | Fail-fast by default, `allow_fail` explicit |
| **expansion** | `$FOO`, `${...}`, `*`, `~` expand implicitly | Strings are literal; expansion is explicit |
| **argument safety** | Must quote `"$var"` correctly | Variables are values, not text |
| **confirmation** | Custom `read -p` with regex | `confirm(default=false)` |
| **exit codes** | `$?` clobbered easily | `status()` preserved |

---

## Honest limitations: where we still drop to Bash

sh2 doesn't cover everything. Here's where you still need `sh("...")`:

### Complex pipelines

```sh2
# sh(...) because: complex pipeline with multiple redirects
let count = capture(sh("find . -name '*.log' | wc -l"))
```

sh2 supports pipelines with `|`, but complex chains with multiple redirects are clearer in Bash.

### Process substitution

```sh2
# sh(...) because: process substitution <(...)
sh("diff <(sort file1) <(sort file2)")
```

No sh2 equivalent. Use the escape hatch.

### Background jobs

```sh2
# sh(...) because: job control (&)
sh("long_task &")
```

sh2 doesn't have job control. Use `sh("...")`.

### NUL-safe filename iteration

```sh2
# sh(...) because: NUL-safe iteration not yet supported
sh("find . -print0 | xargs -0 rm")
```

For filenames with newlines, you need `\0` handling. sh2's `lines(...)` splits on newlines.

**The trade-off:** Inside `sh("...")`, you get full Bash power but lose sh2's safety guarantees. Use sparingly, and document why.

---

## When to consider rewriting in sh2

| Situation | Recommendation |
|-----------|----------------|
| ✅ Install/deploy scripts | High value: sudo, confirmation, logging |
| ✅ CI/CD automation | Fail-fast behavior, no TTY surprises |
| ✅ Scripts reviewed by multiple people | Readability matters |
| ✅ Scripts that touch production | Safety matters |
| ⚠️ Quick one-off scripts | Bash is fine |
| ⚠️ awk/sed-heavy text processing | Bash pipelines are more natural |
| ❌ Interactive scripts with job control | sh2 doesn't support `&`, `fg`, `bg` |

---

## The takeaway

The hidden tax of reviewing Bash isn't about bad code. It's about a language where intent is implicit and behavior depends on context.

Every `cd`, every `$var`, every missing `--`, every `set -e` exception—they all add up. You end up simulating the shell in your head, over and over.

sh2 isn't a Bash replacement. It's a way to write the scripts that matter—the ones that touch production, that run with elevated privileges, that other people have to review—in a form where the intent is visible on the page.

That's not a small thing. That's forty-five minutes I'd like back.

---

# Docs

The GitHub repo is here:  
**[https://github.com/siu-mak/sh2lang](https://github.com/siu-mak/sh2lang)**

## Further Documentation

- [`docs/language.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/language.md) — full language reference (syntax + semantics)
- [`docs/sh2do.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/sh2do.md) — sh2do CLI documentation
- `tests/` — fixtures and integration tests (acts as an executable spec)
