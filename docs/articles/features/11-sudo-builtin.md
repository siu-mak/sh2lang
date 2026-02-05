---
title: "Safer sudo: readable options instead of memorized flags"
description: "How sh2's sudo(...) builtin makes privileged commands easier to review, safer in CI, and harder to misuse."
---
<a href="https://github.com/siu-mak/sh2lang">
  <img src="../../images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# Safer sudo: readable options instead of memorized flags

An ops team once shipped a deploy script with this line:

```bash
sudo -u deploy $CMD
```

It worked great—until someone set `CMD` to `-n cat /etc/shadow`. The script ran `sudo -u deploy -n cat /etc/shadow`, interpreting the dash as a flag. In CI, another script hung indefinitely because `sudo` prompted for a password that no one would ever type.

These aren't exotic bugs. They're what happens when you mix unvalidated strings with privileged commands.

sh2's `sudo(...)` builtin exists to close this gap.

---

## What `sudo(...)` is

`sudo(...)` is a structured wrapper that compiles to a `sudo ... -- cmd args...` invocation.

- **Named options** replace cryptic flags: `user="root"` instead of `-u root`.
- **Compile-time validation** catches typos and type errors before you run the script.
- **Automatic `--` separator** prevents command arguments from being interpreted as sudo flags.
- **Stable flag ordering**: the generated command is predictable and reviewable.

---

## Supported options

| Option | Type | Maps to | Notes |
|--------|------|---------|-------|
| `user` | string literal | `-u` | Run as specified user |
| `n` | boolean literal | `-n` | Non-interactive (no password prompt; fails if password required) |
| `k` | boolean literal | `-k` | Invalidate cached credentials |
| `prompt` | string literal | `-p` | Custom password prompt |
| `E` | boolean literal | `-E` | Preserve entire environment |
| `env_keep` | list of string literals | `--preserve-env=...` | Preserve specific variables |
| `allow_fail` | boolean literal | *(control flow)* | Statement-form only; don't abort on failure |

**All option values must be literals.** You cannot pass a variable as `user=my_var`—the compiler will reject it. This ensures the generated command is predictable.

---

## Examples: Before and After

### 1. Basic sudo

**Bash:**
```bash
sudo apt-get update
```

**sh2:**
```sh2
sudo("apt-get", "update")
```

- The `--` separator is inserted automatically: `sudo -- apt-get update`.
- No accidental flag injection from arguments.

---

### 2. Run as specific user (`-u`)

**Bash:**
```bash
sudo -u deploy whoami
```

**sh2:**
```sh2
sudo("whoami", user="deploy")
```

- Intent is obvious: the reader sees `user="deploy"` instead of decoding `-u`.
- Generated: `sudo -u deploy -- whoami`.

---

### 3. Non-interactive mode for CI (`-n`)

**Bash:**
```bash
sudo -n systemctl restart nginx
```

**sh2:**
```sh2
sudo("systemctl", "restart", "nginx", n=true)
```

- `n=true` signals "fail immediately if a password is needed" (essential for CI).
- Generated: `sudo -n -- systemctl restart nginx`.

---

### 4. Invalidate cached credentials (`-k`)

**Bash:**
```bash
sudo -k apt install htop
```

**sh2:**
```sh2
sudo("apt", "install", "htop", k=true)
```

- Force a fresh authentication by setting `k=true`.
- Generated: `sudo -k -- apt install htop`.

---

### 5. Preserve entire environment (`-E`)

**Bash:**
```bash
sudo -E make install
```

**sh2:**
```sh2
sudo("make", "install", E=true)
```

- Keeps your environment variables when running as root.
- Generated: `sudo -E -- make install`.

---

### 6. Preserve specific variables (`--preserve-env=...`)

**Bash:**
```bash
sudo --preserve-env=HTTP_PROXY,HTTPS_PROXY curl https://example.com
```

**sh2:**
```sh2
sudo("curl", "https://example.com", env_keep=["HTTP_PROXY", "HTTPS_PROXY"])
```

- Only named variables are preserved; clearer than `-E` (which preserves everything).
- Generated: `sudo --preserve-env=HTTP_PROXY,HTTPS_PROXY -- curl https://example.com`.

---

### 7. Commands with arguments that start with `-`

**Bash (risky):**
```bash
sudo rm $FILE    # If FILE is "-rf", this is sudo rm -rf
```

**Bash (safer):**
```bash
sudo -- rm "$FILE"
```

**sh2 (always safe):**
```sh2
sudo("rm", file)
```

- sh2 always inserts `--` before the command, so `file` containing `-rf` becomes a literal argument, not a flag.

---

### 8. Combining with confirmation
Combine with `confirm(...)` for interactive safety. (See [Confirm Helper](12-confirm-helper.md))

```sh2
if confirm("Delete cache?", default=false) {
    sudo("rm", "-rf", "/var/cache/*", n=true)
}
```

---

### 9. Handling failure explicitly
Use `allow_fail=true` to handle errors without aborting. (See [Error Handling](15-error-handling.md))

```sh2
sudo("systemctl", "is-active", "nginx", allow_fail=true)
if status() != 0 {
    # Handle inactive service
}
```

> **Note:** `allow_fail` is only valid in statement form. If you need to capture output while allowing failure, use `capture(sudo(...), allow_fail=true)` instead.

---

### 10. Mixed positional and named arguments

sh2 allows mixing positional command arguments with named options in any order:

**sh2:**
```sh2
sudo(user="root", "apt-get", "update", n=true)
sudo("apt-get", n=true, "upgrade", user="root")
sudo(n=true, "ls", user="admin")
```

All three compile successfully. The generated flag order is always stable: `-u ... -n -k -p ... -E --preserve-env=... -- cmd args...`.

---

## Mistakes sh2c catches

The compiler validates `sudo(...)` calls at compile time:

### 1. Unknown option

```sh2
sudo("ls", xyz=true)
```

**Error:**
```
unknown sudo() option 'xyz'; supported: user, n, k, prompt, E, env_keep, allow_fail
```

### 2. Duplicate option

```sh2
sudo("ls", n=true, n=false)
```

**Error:**
```
n specified more than once
```

### 3. Wrong type for `user`

```sh2
let u = "root"
sudo("ls", user=u)
```

**Error:**
```
user must be a string literal
```

### 4. Wrong type for `env_keep`

```sh2
sudo("env", env_keep="PATH")
```

**Error:**
```
env_keep must be a list of string literals
```

### 5. `allow_fail` in expression form

```sh2
let out = capture(sudo("ls", allow_fail=true))
```

**Error:**
```
allow_fail is only valid on statement-form sudo(...); use capture(sudo(...), allow_fail=true) to allow failure during capture
```

---

## When to use what

| Situation | Recommendation |
|-----------|----------------|
| **Most privileged commands** | Use `sudo(...)`. You get validation, `--`, and readable options. |
| **Need variables in options** | Not supported. If you truly need dynamic users, use `run("sudo", "-u", user, "--", "cmd")` and accept the review burden. |
| **Complex pipelines** | `sudo(...)` works in pipelines: `run("cat", "file") \| sudo("tee", "/etc/file", n=true)`. |
| **Raw shell features needed** | Use `sh("sudo -u root cmd")` as a last resort. You lose validation. |

### Why not `run("sudo", ...)`?

You can write `run("sudo", "-n", "--", "apt", "update")`, but:
- No compile-time validation of flags.
- Easy to forget `--`.
- Less readable than `sudo("apt", "update", n=true)`.

### Why not `sh("sudo ...")`?

`sh("sudo apt update")` works, but:
- No argument safety (word splitting, shell injection).
- No compile-time checks.
- Use only when you need full shell syntax (e.g., `sudo sh -c "..."` for complex commands).

---

## Copy/paste cheatsheet

### Restart a service (non-interactive)

```sh2
sudo("systemctl", "restart", "nginx", n=true)
```

### Install a package as root

```sh2
sudo("apt-get", "install", "-y", "htop", user="root", n=true)
```

### Edit a protected file (with confirmation)

```sh2
if confirm("Edit /etc/hosts?", default=false) {
    sudo("nano", "/etc/hosts")
}
```

### Check service status without aborting

```sh2
sudo("systemctl", "is-active", "nginx", allow_fail=true)
if status() != 0 {
    print_err("nginx is not active")
}
```

### Run a command with HTTP proxy preserved

```sh2
sudo("curl", "https://internal.corp/file", env_keep=["HTTP_PROXY", "HTTPS_PROXY"], n=true)
```

### Force credential re-prompt

```sh2
sudo("apt", "upgrade", k=true)
```

### Deploy as a specific user

```sh2
sudo("deploy.sh", user="deploy", n=true)
```

### Pipeline with sudo

```sh2
run("cat", "local.conf") | sudo("tee", "/etc/app/config.conf", n=true)
```

---

# Docs

The GitHub repo is here:  
**[https://github.com/siu-mak/sh2lang](https://github.com/siu-mak/sh2lang)**

## Further Documentation

- [`docs/language.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/language.md) — full language reference (syntax + semantics)
- [`docs/sh2do.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/sh2do.md) — sh2do CLI documentation
- `tests/` — fixtures and integration tests (acts as an executable spec)
