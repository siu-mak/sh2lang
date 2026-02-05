---
title: "Confirmations done right: confirm(...), defaults, and CI overrides"
description: "A practical guide to safe prompts in sh2: predictable defaults, automation-friendly overrides, and readable control flow."
---
<a href="https://github.com/siu-mak/sh2lang">
  <img src="../../images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# Confirmations done right: confirm(...), defaults, and CI overrides

A deploy script had this line:

```bash
read -p "Continue? " ans
if [[ $ans =~ ^[Yy] ]]; then
    ...
fi
```

It worked perfectly—until it ran in CI. The script hung forever, waiting for input that would never come. The pipeline timed out after 30 minutes. The next attempt used `yes |` as a prefix, which worked until someone accidentally ran it locally and deleted production data without being asked.

Prompt handling in Bash is surprisingly hard to get right.

---

## What `confirm(...)` is

`confirm(prompt)` asks the user a yes/no question and returns a boolean.

```sh2
if confirm("Delete all logs?") {
    run("rm", "-rf", "logs/")
}
```

That's it. But the small details matter:

- **Accepted inputs**: `y`, `yes`, `Y`, `YES` → `true`; `n`, `no`, `N`, `NO` → `false` (case-insensitive).
- **Optional default**: `confirm("Proceed?", default=false)`.
- **Non-interactive behavior**: If stdin is not a TTY and a default is provided, it uses the default. If no default, the script fails with an error.
- **Environment overrides**: `SH2_YES=1` → always `true`; `SH2_NO=1` → always `false`.

---

## Behavior table

| stdin is TTY? | `default` provided? | `SH2_YES` | `SH2_NO` | Result |
|--------------|---------------------|-----------|----------|--------|
| Yes | — | — | — | Prompt user, return based on input |
| Yes | — | `1` | — | Return `true` (no prompt) |
| Yes | — | — | `1` | Return `false` (no prompt) |
| No | `true` | — | — | Return `true` |
| No | `false` | — | — | Return `false` |
| No | *(none)* | — | — | **Error**: fails with message |
| No | *(any)* | `1` | — | Return `true` (override) |
| No | *(any)* | — | `1` | Return `false` (override) |

**Priority**: `SH2_YES` / `SH2_NO` override everything else.

---

## Examples: Before and After

### 1. Basic "Proceed?" prompt

**Bash:**
```bash
read -p "Proceed? [y/N] " ans
if [[ "$ans" =~ ^[Yy] ]]; then
    echo "Proceeding..."
fi
```

**sh2:**
```sh2
if confirm("Proceed?") {
    print("Proceeding...")
}
```

- No regex to get wrong.
- No quoting around `$ans`.
- Prompt text is the only argument.

---

### 2. Destructive delete with `default=false`

**Bash:**
```bash
read -p "Delete /var/cache/*? [y/N] " ans
if [[ "$ans" =~ ^[Yy](es)?$ ]]; then
    rm -rf /var/cache/*
fi
```

**sh2:**
```sh2
if confirm("Delete /var/cache/*?", default=false) {
    run("rm", "-rf", "/var/cache/*")
}
```

- `default=false` ensures CI/automation safely skips deletion.
- No regex complexity.
- Clear intent in the code.

---

### 3. Safe "yes by default" for non-destructive actions

**Bash:**
```bash
read -p "Show verbose output? [Y/n] " ans
if [[ ! "$ans" =~ ^[Nn] ]]; then
    VERBOSE=1
fi
```

**sh2:**
```sh2
let verbose = confirm("Show verbose output?", default=true)
if verbose {
    set env.VERBOSE = "1"
}
```

- `default=true` means pressing Enter (or running in CI) proceeds.
- Appropriate for safe, reversible actions.

---

### 4. Confirm before `sudo` (restart service)

**sh2:**
```sh2
if confirm("Restart nginx?", default=false) {
    sudo("systemctl", "restart", "nginx", n=true)
}
```

- Combines safety of `confirm` with safety of `sudo(...)`.
- `n=true` prevents sudo from prompting for a password in CI.

---

### 5. Confirm before `sudo` (install packages)

**sh2:**
```sh2
if confirm("Install htop and curl?", default=false) {
    sudo("apt-get", "install", "-y", "htop", "curl", n=true)
}
```

- User must explicitly agree to package installation.
- `-y` handles apt's own prompt; `confirm` handles yours.

---

### 6. Preview → Confirm → Apply

**sh2:**
```sh2
print("The following files will be deleted:")
run("find", ".", "-name", "*.bak", "-print")

if confirm("Proceed with deletion?", default=false) {
    run("find", ".", "-name", "*.bak", "-delete")
}
```

- Show what will happen first.
- Ask for confirmation with safe default.
- Apply only if user agrees.

---

### 7. Non-interactive CI with `SH2_YES=1`

**sh2:**
```sh2
if confirm("Apply database migrations?", default=false) {
    run("./migrate.sh")
}
```

**Running in CI:**
```bash
SH2_YES=1 ./deploy.sh
```

- `SH2_YES=1` overrides the prompt, returning `true`.
- No need to pipe `yes` or modify the script.
- Explicit: reviewers see `SH2_YES=1` in the CI config.

---

### 8. Fail-fast when no default in non-interactive mode

**sh2:**
```sh2
if confirm("This action cannot be undone. Continue?") {
    run("dangerous-operation")
}
```

**Running in CI (no `SH2_YES`, no default):**
```
Error: confirm() requires a default in non-interactive mode
```

- The script fails loudly instead of hanging.
- Forces automation to make an explicit choice (`SH2_YES=1` or add `default=...`).

---

### 9. Double confirmation for very dangerous operations

**sh2:**
```sh2
if confirm("Delete production database?", default=false) {
    if confirm("Are you REALLY sure? This cannot be undone.", default=false) {
        run("./drop-database.sh")
    }
}
```

- Two prompts for irreversible actions.
- Both default to `false`—safe in automation.

---

### 10. Dynamic prompt with concatenation

**sh2:**
```sh2
let target = "staging"
if confirm("Deploy to " & target & "?", default=false) {
    run("./deploy.sh", target)
}
```

- Use `&` to build the prompt dynamically.
- Prompt reads: "Deploy to staging?"

---

### 11. Confirm + `allow_fail` + `status()` (retry pattern)

**sh2:**
```sh2
run("./flaky-test.sh", allow_fail=true)
if status() != 0 {
    if confirm("Test failed. Retry?", default=false) {
        run("./flaky-test.sh")
    }
}
```

- First run doesn't abort on failure.
- User decides whether to retry.

---

## Mistakes sh2 prevents

| Bash mistake | What goes wrong | How `confirm` avoids it |
|--------------|-----------------|-------------------------|
| `[[ $ans =~ ^[Yy] ]]` | Matches "Yikes", not just "yes" | Exact match: only `y`, `yes`, `Y`, `YES` accepted |
| `if [ $ans = "y" ]` | Unquoted variable breaks with spaces/empty | No variable to quote; `confirm` returns boolean |
| `read -p` not portable | `-p` flag differs between shells | `confirm` compiles to portable code |
| Script hangs in CI | `read` waits forever with no TTY | Fails fast or uses default in non-interactive |
| Empty input treated as yes | `[[ "" =~ ^[Yy]? ]]` is true | Empty input is not accepted; re-prompts or uses default |

---

## Rule of thumb

### When to use `confirm(...)`

- **Destructive operations**: Delete, overwrite, drop.
- **Privileged operations**: Before `sudo(...)`.
- **Irreversible changes**: Database migrations, production deploys.
- **Human checkpoints**: "Did you review the diff?"

### When NOT to prompt

- **Idempotent reads**: Listing files, checking status.
- **Fully automated pipelines**: Use `SH2_YES=1` or provide `default=`.
- **Repeated runs**: If the script runs every minute, don't prompt.

### When to use a flag instead

For scripts where "no prompt" should be explicit:

```sh2
# Instead of: if confirm("Force?", default=false)
# Consider: require a --force flag

let force = (arg(1) == "--force")
if !force {
    print_err("Use --force to skip confirmation")
    exit(1)
}
```

This pattern is better when:
- The action is always dangerous.
- You want the caller to be explicit in code/commands.
- You don't want any interactive prompts at all.

---

## Copy/paste recipes

### Delete a directory

```sh2
if confirm("Delete ./build/?", default=false) {
    run("rm", "-rf", "./build/")
}
```

### Restart a service

```sh2
if confirm("Restart nginx?", default=false) {
    sudo("systemctl", "restart", "nginx", n=true)
}
```

### Rotate logs

```sh2
if confirm("Rotate and compress logs?", default=false) {
    run("logrotate", "-f", "/etc/logrotate.conf")
}
```

### Apply changes after dry-run

```sh2
print("Dry run:")
run("terraform", "plan")

if confirm("Apply these changes?", default=false) {
    run("terraform", "apply", "-auto-approve")
}
```

### Double confirmation for destructive operations

```sh2
if confirm("Drop all tables?", default=false) {
    if confirm("This is irreversible. Confirm again?", default=false) {
        run("./drop-tables.sh")
    }
}
```

### CI override

```bash
# In your CI pipeline:
SH2_YES=1 ./deploy.sh
```

### Force "no" in test runs

```bash
# Ensure prompts are declined:
SH2_NO=1 ./risky-script.sh
```

### Conditional prompt based on environment

```sh2
# Only prompt in production
if env.ENV == "production" {
    if !confirm("Deploy to PRODUCTION?", default=false) {
        print("Aborted.")
        exit(0)
    }
}
run("./deploy.sh")
```

---

# Docs

The GitHub repo is here:  
**[https://github.com/siu-mak/sh2lang](https://github.com/siu-mak/sh2lang)**

## Further Documentation

- [`docs/language.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/language.md) — full language reference (syntax + semantics)
- [`docs/sh2do.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/sh2do.md) — sh2do CLI documentation
- `tests/` — fixtures and integration tests (acts as an executable spec)
