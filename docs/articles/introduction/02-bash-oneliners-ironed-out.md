---
title: "Bash one-liners ironed out"
description: "Eight real-world Bash one-liners, rewritten in sh2/sh2do, with honest comparisons."
---
<a href="https://github.com/siu-mak/sh2lang">
  <img src="../../images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# Bash one-liners ironed out

## What this article is

**One-liners are seductive.** They fit in a tweet, a README, or a quick Slack message. But the moment you need to understand, trust, or modify them, the magic evaporates:

* Quoting rules are invisible until they break.
* Word splitting happens to variables you didn't think about.
* `$` expansion fires when you didn't want it.
* Error handling is "hope nothing fails."

**sh2** is a small, structured shell language. You write structured snippets, compile them to Bash or POSIX sh, and run them. **sh2do** is the one-command wrapper: `sh2do 'snippet'` compiles and executes in one step.

The goal isn't to replace Bash—it's to give you a safer, more readable way to handle the hairy cases where Bash one-liners become write-only code.

This article walks through **8 real-world one-liner patterns**, shows the Bash version and the sh2do equivalent, and honestly assesses when sh2 helps and when Bash remains the better tool.

---

## 1. Confirmation before a dangerous action

### The Bash way

```bash
read -p "Delete /var/data? [y/N] " ans && [[ "$ans" =~ ^[Yy] ]] && rm -rf /var/data
```

**Common failure modes:**
- Forgetting to quote `$ans` (empty input breaks the test).
- Script runs in CI with no stdin—hangs forever or deletes anyway.
- No environment variable override for automation.

### The sh2do way

```bash
sh2do 'if confirm("Delete /var/data?", default=false) { run("rm", "-rf", "/var/data") }'
```

**Why it's better:**
- `confirm(...)` handles yes/no parsing, default values, and CI overrides.
- See [Confirm Helper](../features/12-confirm-helper.md) for details on `default=false` and `SH2_YES`.


---

## 2. Running a command as root with flags

### The Bash way

```bash
sudo -n -u deploy systemctl restart nginx
```

**Common failure modes:**
- The `-n` must come *before* the command, but flag order is easy to get wrong.
- Forgetting quotes around arguments that contain spaces.
- No way to safely add `--preserve-env=PATH` without risking syntax errors.

### The sh2do way

```bash
sh2do 'sudo("systemctl", "restart", "nginx", user="deploy", n=true)'
```

**Why it's better:**
- Named options (`user=`, `n=`) are validated at compile time.
- The compiler enforces a stable flag order with mandatory `--` separator.
- See [Sudo Builtin](../features/11-sudo-builtin.md) for the full list of options.


---

## 3. Error handling based on exit status

### The Bash way

```bash
output=$(grep "pattern" file.txt) || { echo "grep failed"; code=$?; exit "$code"; }
```

**Common failure modes:**
- `$?` gets clobbered: after `echo "grep failed"` succeeds, `$?` becomes 0, so `code` is 0.
- You must save `$?` *before* running any other command—easy to forget.
- Easy to forget the braces, turning the one-liner into two separate commands.
- Word splitting on `$output` if you later use it unquoted.

### The sh2do way

```bash
sh2do '
let out = capture(run("grep", "pattern", "file.txt"), allow_fail=true)
if status() != 0 {
    print_err("grep failed with " & status())
}
'
```

**Why it's better:**
- `allow_fail=true` prevents script abort, and `status()` is preserved.
- See [Error Handling](../features/15-error-handling.md) for more patterns.


---

## 4. The quoting / word-splitting footgun

### The Bash way

```bash
file="my document.txt"
cat $file  # WRONG: splits into "my" and "document.txt"
```

**Common failure modes:**
- Forgetting quotes around `$file` causes word splitting.
- Glob expansions fire if the variable contains `*` or `?`.
- Even experienced devs get this wrong in complex scripts.

### The sh2do way

```bash
sh2do '
let file = "my document.txt"
run("cat", file)
'
```

**Why it's better:**
- **No implicit expansion.** `run(...)` passes arguments as-is.
- See [No Implicit Expansion](../features/13-no-implicit-expansion.md) for the rules.


---

## 5. The `$` expansion / format-string scenario

### The Bash way

```bash
out=$(dpkg-query -W -f '${Package}\n' bash)
```

**Common failure modes:**
- If you accidentally use double quotes: `"${Package}"` becomes an empty variable expansion.
- Mixing single and double quotes to get the right behavior is error-prone.
- The distinction between `${}` as a Bash variable vs a dpkg-query format specifier is invisible.

### The sh2do way

```bash
sh2do '
let out = capture(run("dpkg-query", "-W", "-f", "${Package}\n", "bash"))
print(out)
'
```

**Why it's better:**
- sh2 string literals are **strict literals**. `"${Package}"` is passed exactly as written.
- See [No Implicit Expansion](../features/13-no-implicit-expansion.md) and the [Dollar Expansion Bug](../case-studies/22-dollar-expansion-bug.md) story.


---

## 6. Pipelines (where Bash remains better)

### The Bash way

```bash
grep ERROR /var/log/app.log | awk '{print $3}' | sort | uniq -c | sort -rn | head -5
```

### The sh2do way

For simple two-stage pipelines, sh2 works well:

```bash
sh2do 'run("echo", "hello world") | run("sed", "s/hello/hi/")'
```

For long text-processing chains, use `sh(...)`:

```bash
# sh(...) because: complex pipeline
sh2do 'sh("grep ERROR /var/log/app.log | awk '\''{print $3}'\'' | sort | uniq -c | sort -rn | head -5")'
```

**Honest assessment:** sh2 pipelines are structured (`run(...) | run(...)`), but for complex `grep|awk|sort|uniq` chains:
- **Bash is genuinely more concise** for ad-hoc text munging.
- sh2's `sh(...)` is an escape hatch that passes raw shell code, losing the safety guarantees.
- Use sh2 when the pipeline benefits from structured arguments; use Bash when the pipeline is pure text transformation.

---

## 7. File iteration (line-by-line processing)

### The Bash way

```bash
while IFS= read -r line; do echo "Line: $line"; done < file.txt
```

**Common failure modes:**
- Forgetting `IFS=` causes leading/trailing whitespace to be stripped.
- Forgetting `-r` causes backslash escapes to be processed.
- Complex quoting if the file path has spaces.

### The sh2do way

sh2 provides `lines(...)` to split a string into a list by newlines:

```bash
sh2do '
let content = read_file("file.txt")
for line in lines(content) {
    print("Line: " & line)
}
'
```

**Why it's better:**
- No `IFS` / `-r` incantations to remember.
- `read_file(...)` is explicit about what you're doing.
- `lines(...)` handles trailing newlines correctly (doesn't produce a final empty element).

**Streaming iteration:** For very large files where loading everything into memory is inefficient, pipe the file in and use `stdin_lines()` to stream line-by-line:

```bash
sh2do '
for line in stdin_lines() {
    print("Line: " & line)
}
' < file.txt
```

---

## 8. The "many files" pattern: `find -exec` vs `xargs`

### The Bash way

```bash
find . -name "*.log" -exec gzip {} \;
# or
find . -name "*.log" -print0 | xargs -0 gzip
```

**Common failure modes:**
- `find -exec` with `\;` is slow (one gzip per file).
- `xargs` without `-0` breaks on filenames with spaces/newlines.
- The `{}` placeholder syntax is arcane and easy to get wrong.

### The sh2do way

```bash
sh2do '
let files = capture(run("find", ".", "-name", "*.log"))
for f in lines(files) {
    run("gzip", f)
}
'
```

**Why it's better:**
- Each argument is properly quoted—no word splitting on filenames with spaces.
- The loop is explicit and readable.
- You can add logic (e.g., `if is_file(f) { ... }`) trivially.

**Trade-off:** This is functionally equivalent to `-exec {} \;` (one process per file). For batch processing, you'd still want `xargs`. sh2 doesn't have a `xargs`-style builtin (yet).

---

## Where sh2do doesn't help (yet)

sh2 is still young. Here are things it **cannot** do:

| Gap | Reality |
|-----|---------|
| **Streaming line iteration** | `stdin_lines()` provides a streaming `while read` equivalent. |
| **Process substitution** | No `<(...)` or `>(...)` syntax. Use `sh(...)` as an escape hatch. |
| **Background jobs / &** | Structured job control exists via `spawn()` and `wait()`, but not terse `&`. |
| **Here-strings** | No `<<<` syntax. Use `sh(...)` or temp files. |
| **Arithmetic in conditions** | Comparisons work, but `$(( ))` arithmetic expansion isn't built-in. |
| **Complex xargs patterns** | No batching multiple arguments. You can loop, but lose parallelism. |
| **Interactive REPL** | sh2do is compile-then-run; there's no interactive shell mode. |

For these cases, **use Bash directly** or use `sh(...)` to embed raw shell code.

---

## Rules of thumb

### When to use Bash

- Quick, interactive throwaway commands.
- Complex text pipelines where `|` is the primary structure.
- When you need features sh2 doesn't have (process substitution, job control).
- Sub-20-character commands where quoting is trivial.

### When to use sh2do

- Any one-liner with **user-controlled input** (file paths, usernames, etc.).
- Commands where **quoting would be error-prone** (spaces, globs, `$`).
- Scripts that need to run in **both interactive and CI** contexts (use `confirm(default=...)`).
- When you want **explicit error handling** (`allow_fail=true` + `status()`).

### When to write a `.sh2` file instead of a one-liner

- More than 3-4 statements.
- Reusable logic (functions, imports).
- You want **version control** and **code review** on your scripts.
- You're building tooling for a team, not just yourself.

---

## Comparison table

| Category | Bash one-liner | Common failure | sh2do version | Verdict |
|----------|---------------|----------------|---------------|---------|
| **Confirmation** | `read -p "..." && [[ $ans =~ ... ]]` | Hangs in CI; no default | `confirm("...", default=false)` | ✅ sh2 is safer |
| **Sudo + flags** | `sudo -n -u user cmd` | Flag order errors; no validation | `sudo("cmd", user="...", n=true)` | ✅ sh2 is clearer |
| **Error status** | `cmd || echo $?` | `$?` gets clobbered | `allow_fail=true` + `status()` | ✅ sh2 preserves status |
| **Word splitting** | `cat $file` | Splits on spaces | `run("cat", file)` | ✅✅ sh2's biggest win |
| **$ expansion** | `'${Package}'` vs `"${Package}"` | Wrong quote type expands | `"${Package}"` is literal | ✅ sh2 is safer |
| **Long pipelines** | `grep \| awk \| sort` | N/A—Bash is good at this | `sh("grep \| awk \| sort")` | ⚖️ Bash is better |
| **File iteration** | `while IFS= read -r` | Forget IFS or -r | `for line in lines(read_file(...))` | ✅ sh2 is cleaner |
| **Many files** | `find -exec \;` or `xargs -0` | Spaces break; slow | Loop with `run("gzip", f)` | ⚖️ Comparable |

---

# Docs

The GitHub repo is here:  
**[https://github.com/siu-mak/sh2lang](https://github.com/siu-mak/sh2lang)**

## Further Documentation

- [`docs/language.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/language.md) — full language reference (syntax + semantics)
- [`docs/sh2do.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/sh2do.md) — sh2do CLI documentation
- `tests/` — fixtures and integration tests (acts as an executable spec)

## Versions

- [v0.1.1](https://github.com/siu-mak/sh2lang/blob/main/docs/releases/v0.1.1.md) — Adds `sudo(...)`, `confirm(...)`, semicolon separators.
- [v0.1.0](https://github.com/siu-mak/sh2lang/blob/main/docs/releases/v0.1.0.md) — First public release of sh2.
