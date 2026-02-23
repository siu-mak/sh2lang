---
title: "Where Bash still wins (and how sh2 fits in anyway)"
description: "A practical, non-defensive guide: which problems Bash solves best, and how sh2/sh2do complements it."
---
<a href="https://github.com/siu-mak/sh2lang">
  <img src="../../images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# Where Bash still wins (and how sh2 fits in anyway)

**Bash is a power tool.** It has 30 years of history, a massive ecosystem, and a syntax optimized for one specific thing: shoving text from one process to another with minimum keystrokes.

If you are doing ad-hoc text munging, interactive exploration, or using tools like `awk`, `sed`, and `jq` as intended, Bash is often the best tool for the job.

**sh2 is not trying to replace that.** It's trying to make the *glue* safer. It's for when you move from "exploring" to "engineering"—when you need to trust the script, review it, or run it in CI without crossing your fingers.

Here are the three areas where Bash reigns supreme—and how you can use sh2 to wrap that power without losing your safety guarantees.

---

## 1. Dense text pipelines

Bash's pipe syntax (`|`) is legendary. When you are stringing together five different text-processing tools, nothing beats it.

### Bash Example 1: Log aggregation

```bash
grep ERROR /var/log/syslog | awk '{print $5}' | sort | uniq -c | sort -nr | head
```

**Why Bash wins:**
- Identical syntax for interactive and scripted use.
- Concise: no commas, no quotes, just data flow.
- Everyone knows `awk '{print $5}'`.

**Why it's hard to review:**
- If you add `sudo` somewhere, where does it go?
- If `grep` fails (no errors found), the pipeline continues and prints nothing (maybe misleading).
- Quoting variables inside `awk` is a nightmare.

### How sh2 fits in: The "Structural Wrapper"

You can keep the pipeline in Bash (using `sh(...)`), but wrap it with sh2 handling.

**sh2do equivalent:**

```sh2
sh2do '
# sh(...) because: complex pipeline (grep|awk|sort)
let summary = capture(sh("grep ERROR /var/log/syslog | awk '\''{print $5}'\'' | sort | uniq -c | sort -nr | head"), allow_fail=true)

if status() != 0 {
    print_err("Pipeline failed")
    exit(1)
}

if trim(summary) == "" {
    print("No errors found.")
} else {
    print("Top errors:")
    print(summary)
}
'
```

**What you gain:**
- **Explicit variable handling:** `summary` is a safe string variable.
- **Logic:** You can check if it's empty before printing.
- **Containment:** The raw shell is isolated inside `sh(...)`.

---

### Bash Example 2: JSON query

```bash
curl -s https://api.github.com/repos/siu-mak/sh2lang | jq '.stargazers_count'
```

**Why Bash wins:**
- `jq` is a domain-specific language that lives happily inside Bash strings.
- `curl | jq` is the standard API client of the terminal.

**How sh2 fits in:**

If you need to *use* that data safely:

**sh2do equivalent:**

```sh2
sh2do '
let url = "https://api.github.com/repos/siu-mak/sh2lang"

# sh(...) because: curl | jq pipeline
let stars = trim(capture(sh($"curl -s {url} | jq .stargazers_count"), allow_fail=true))

if status() != 0 {
    print_err("Failed to fetch stars")
} else {
    print($"Stars: {stars}")
}
'
```

**What you gain:**
- **Interpolation:** `$"Stars: {stars}"` is readable and safe.
- **Safety:** Splitting it into steps lets you check `status()` or validate `json` content.

---

## 2. Shell-native tricks

Bash has features that are deeply integrated into the OS process model. sh2 intentionally avoids some of these to stay portable and structured, meaning Bash is simply more capable here.

### Bash Example 3: Process substitution

```bash
diff <(sort current.txt) <(sort expected.txt)
```

**Why Bash wins:**
- Scopes temporary file descriptors implicitly (files vanish after command).
- No cleanup logic needed.

**How sh2 fits in:**
It doesn't. **Stay in Bash** for this. If you absolutely must do it in sh2, use `sh(...)`:

```sh2
# sh(...) because: process substitution <(...)
sh2do 'sh("diff <(sort current.txt) <(sort expected.txt)")'
```



## 3. Ad-hoc interactive glue

Sometimes you just need to bang out a command.

### Bash Example 5: One-off SSH pipelines

```bash
ssh user@host "cat /var/log/syslog | grep ERROR"
```

**Why Bash wins:**
- You type it, it runs.
- `ssh` expects a string argument, which shell provides easily.

**How sh2 fits in:**

When that "one-off" becomes a recurrent task, quoting arguments prevents disasters.

**sh2do equivalent:**

```sh2
sh2do '
let host = "user@example.com"
let pattern = "ERROR"
# run() quotes arguments safely, even if pattern has spaces/special chars
run("ssh", host, "grep", pattern, "/var/log/syslog")
'
```

**What you gain:**
- **Argument safety:** `run(...)` passes `pattern` as a distinct argument to `ssh`. This means `ssh` receives it cleanly, avoiding local shell splitting. (Note: `ssh` still concatenates arguments on the remote side, but passing structured args locally is safer than building one giant shell string yourself.)

### Bash Example 6: Bulk remote operations (Xargs)

```bash
cat hosts.txt | xargs -n1 -I{} ssh {} "systemctl restart nginx"
```

**Why Bash wins:**
- `xargs` is the ultimate parallelizer (with `-P`).
- Ideal for fire-and-forget commands.

**How sh2 fits in:**

When you need **safety checks** before triggering a fleet-wide action.

**sh2do equivalent:**

```sh2
sh2do '
let hosts = lines(read_file("hosts.txt"))
if confirm($"Restart nginx on {len(hosts)} hosts?", default=false) {
    for host in hosts {
        print($"Restarting {host}...")
        # run() quotes the SSH command safely
        run("ssh", host, "systemctl", "restart", "nginx", allow_fail=true)
    }
}
'
```

**What you gain:**
- **Confirmation:** `confirm()` prevents accidental fleet rollouts.
- **Observability:** You can print progress explicitly.
- **Non-blocking default:** `default=false` blocks this from running in CI without override.

---

## The Rubric: When to use what

- **Stay in Bash when:**
  - You are working interactively in a terminal.
  - You are writing a simple filter (grep/awk/sed).
  - You need features like `&`, `wait`, `<(...)`, or `<<<`.
  - The script is under 10 lines and handles no user input.

- **Use sh2do when:**
  - You are handling **filenames**, **paths**, or **user input** (quoting safety).
  - You need to run in **CI** safely (`confirm` defaults, `allow_fail`).
  - You want explicit error handling (`status()` checks).
  - You are using `sudo` flags (validated options).

- **Write a .sh2 tool when:**
  - The logic is complex (functions, imports).
  - You need to distribute the tool to others.
  - You want a clear `usage()` help message.
  - Side effects (deletions, restarts) are involved.

---

## Quick Comparison

| Task Type | Bash is best because... | sh2 helps by... | Recommended |
|-----------|-------------------------|-----------------|-------------|
| **Text Munging** | `|` pipelines are concise and powerful | `sh(...)` wrapper adds error checking | **Bash** |
| **Simple Loops** | `for i in {1..5}` is typed in seconds | Structured loops are more readable long-term | **Bash** (interactive) / **sh2** (script) |
| **JSON/API** | `curl \| jq` is standard | `capture()` + `status()` validates response | **Mix** (wrap jq in sh2) |
| **Parallelism** | `xargs -P` / `&` job control | (Not supported natively) | **Bash** |
| **Remote Commands** | `ssh` interacts well with shell | `run("ssh", ...)` quotes args safely | **sh2** (if args are dynamic) |
| **Dangerous Ops** | (It isn't; `rm -rf` is risky) | `confirm(...)` + `run` safety | **sh2** (Always) |

---

**sh2 is the safety guard, not the engine.** Use Bash engine for what it's good at (processing text), and use sh2 to drive it safely (handling control flow, arguments, and errors).
