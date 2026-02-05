---
title: "No implicit expansion: why sh2 treats strings literally"
description: "Bash expands words in surprising ways. sh2 keeps strings literal by default, so scripts are easier to review and safer to run."
---
<a href="https://github.com/siu-mak/sh2lang">
  <img src="../../images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# No implicit expansion: why sh2 treats strings literally

A developer wrote a template generation script:

```bash
dpkg-query -W -f '${Package}\n' bash
```

Worked fine. Then someone moved it into a bigger script where `set -u` was enabled. Suddenly it failed: Bash tried to expand `${Package}` as a variable, found it unset, and aborted. The fix? More quotes. Or escaping. Or both. Or maybe a here-doc.

This is the core problem: **Bash does things to your strings before passing them to commands.** Spaces split. Asterisks glob. Dollar signs expand. Tildes transform. You can quote around most of it, but the rules are inconsistent and the failure modes are silent.

sh2 takes a different approach: **strings are literal by default.** You opt into expansion explicitly.

---

## What counts as "implicit expansion"?

In Bash, text goes through multiple transformation phases before reaching a command:

| Phase | What happens | Example pitfall |
|-------|--------------|-----------------|
| **Word splitting** | Unquoted variables split on whitespace | `rm $file` deletes multiple files if `file="a b"` |
| **Globbing** | `*`, `?`, `[...]` expand to matching files | `echo *` lists the directory instead of printing `*` |
| **Parameter expansion** | `$FOO`, `${FOO}` replace with variable value | `echo $undefined` silently becomes empty |
| **Tilde expansion** | `~` becomes `$HOME` | Works in some contexts, not others |
| **Command substitution** | `$(...)` runs a command | Nested quoting becomes a nightmare |

All of these happen *automatically* unless you know the right quoting incantation.

---

## sh2's rule: strings are literal

In sh2:
- `"..."` is a strict literal. Dollar signs, asterisks, braces—all stay as-is.
- `run(...)` passes each argument as-is. No splitting, no globbing.
- If you want variables in strings, you explicitly use `&` concatenation or `$"..."` interpolation.

This means you can read sh2 code and know what will happen.

---

## 12 examples: Bash vs sh2

### 1. Spaces in variables

**Bash (dangerous):**
```bash
file="my document.txt"
rm $file
# Bash runs: rm my document.txt (two arguments!)
```

**sh2 (safe):**
```sh2
let file = "my document.txt"
run("rm", file)
# rm receives exactly one argument: "my document.txt"
```

**Why it matters:** No quoting required. Variables are values, not text to be re-parsed.

---

### 2. Literal asterisk

**Bash:**
```bash
echo "*"    # Prints *, but only because it's quoted
echo *      # Lists files in current directory
```

**sh2:**
```sh2
run("echo", "*")
# Prints: *
```

**Why it matters:** What you see is what you get.

---

### 3. Literal question mark

**Bash:**
```bash
echo "pattern?"   # Stays literal (quoted)
ls pattern?       # Globs to pattern1, pattern2, etc.
```

**sh2:**
```sh2
run("ls", "pattern?")
# Passes literal "pattern?" to ls (no glob)
```

**Why it matters:** No silent transformation into something else.

---

### 4. Tilde is just a character

**Bash:**
```bash
ls ~/Documents    # Tilde expands to $HOME
ls "~/Documents"  # Tilde stays literal (ls fails!)
```

**sh2:**
```sh2
run("ls", "~/Documents")
# Passes literal "~/Documents" (THIS WILL FAIL)

# Correct sh2 pattern:
run("ls", env.HOME & "/Documents")
```

**Why it matters:** No hidden expansion. The correct pattern (`env.HOME & "..."`) makes intent explicit.

---

### 5. Literal `$FOO`

**Bash:**
```bash
echo "$FOO"   # Prints value of FOO (or empty if unset)
echo '$FOO'   # Prints literal $FOO (single quotes)
```

**sh2:**
```sh2
print("$FOO")
# Prints: $FOO
```

**Why it matters:** You don't need to know single-vs-double quote rules.

---

### 6. Literal `${Package}` for dpkg-query

**Bash:**
```bash
dpkg-query -W -f '${Package}\n' bash   # Must use single quotes
dpkg-query -W -f "${Package}\n" bash   # BUG: expands as variable
```

**sh2:**
```sh2
run("dpkg-query", "-W", "-f", "${Package}\n", "bash")
# ${Package} passes literally to dpkg-query
```

**Why it matters:** Format strings with `${...}` just work. No escaping needed.

---

### 7. Preventing option injection with `--`

**Bash:**
```bash
rm -- "$file"   # Must remember to add --
rm "$file"      # If file is "-rf", disaster
```

**sh2:**
```sh2
run("rm", "--", file)
# You can add -- explicitly if desired, but...

run("rm", file)
# Even without --, sh2 quotes the argument correctly
# so "-rf" stays an argument, not a flag
```

**Why it matters:** Arguments are arguments. They don't get reinterpreted as flags by the shell.

---

### 8. Safe concatenation with `&`

**Bash:**
```bash
path="$HOME/repos/$project/main.go"   # Easy to mis-quote
```

**sh2:**
```sh2
let path = env.HOME & "/repos/" & project & "/main.go"
```

**Why it matters:** Explicit joining. No expansion surprises.

---

### 9. Explicit interpolation with `$"..."` *(v0.1.1+)*

**sh2:**
```sh2
let user = "alice"
print($"Welcome, {user}!")
# Output: Welcome, alice!
```

### With expressions:

```sh2
print($"Sum: {1 + 2}")
# Output: Sum: 3
```

**Why it matters:** You opt into interpolation. The `$` prefix signals intent.

> **Note:** `$"..."` interpolation was added in v0.1.1. Both `&` concatenation and `$"..."` are valid approaches.

---

### 10. Escape hatch: `sh("...")`

When you genuinely need shell parsing:

**sh2:**
```sh2
sh("ls *.log | wc -l")
# Globs expand. Pipes work. You're in shell-land.
```

**Why it matters:** The escape hatch is explicit. Reviewers see `sh(...)` and know: "shell rules apply here."

> **Warning:** Inside `sh(...)`, you lose sh2's safety guarantees. Avoid with user input.

---

### 11. `with cwd(...)` requires a literal path

**sh2:**
```sh2
# ✅ Works
with cwd("/tmp") {
    run("ls")
}

# ❌ Compile error
let dir = "/tmp"
with cwd(dir) { ... }
# Error: cwd(...) requires a string literal path
```

**Why it exists:** If `cwd(...)` accepted variables, the compiler couldn't verify path safety at compile time. Use `run("sh", "-c", ...)` with an explicit `cd` if you need dynamic paths.

---

### 12. Explicit control: `allow_fail` + `status()`

**Bash:**
```bash
rm "$file" || true        # Ignore error
status=$?                 # Oops, captured `true`'s status
```

**sh2:**
```sh2
run("rm", file, allow_fail=true)
if status() != 0 {
    print("rm failed with " & status())
}
```

**Why it matters:** No shell magic. You say "allow failure", then check the status.

---

## How to think about strings in sh2

1. **`"..."` is always literal.** Dollar signs, braces, asterisks—all text.
2. **`run(...)` never splits or globs.** Each argument is passed as-is.
3. **Use `&` to build strings.** `"Hello " & name` is explicit concatenation.
4. **Use `$"..."` for interpolation** *(v0.1.1+)*. Braces mark holes: `$"Hi {name}"`.
5. **Use `env.HOME` instead of `~`.** Tilde doesn't expand.
6. **`sh("...")` is the escape hatch.** Shell rules apply inside. Use sparingly.
7. **You can always check.** Run `sh2c --emit-sh script.sh2` to see the generated Bash.

---

## Comparison table

| Bash behavior | Typical footgun | sh2 behavior | How to do it intentionally |
|---------------|-----------------|--------------|---------------------------|
| Word splitting (`$var` splits on spaces) | `rm $file` deletes wrong files | No splitting | Just use the variable: `run("rm", file)` |
| Globbing (`*` matches files) | `echo *` lists directory | No globbing | Use `sh("echo *")` if you need globbing |
| Parameter expansion (`$FOO`) | Unset variables become empty | Literal `$FOO` | Use `& env.FOO` or `$"{env.FOO}"` |
| Brace expansion (`${...}`) | Conflicts with format strings | Literal `${...}` | Just write it: `"${Package}"` |
| Tilde expansion (`~`) | Works sometimes, not others | Literal `~` | Use `env.HOME & "/path"` |
| Command substitution (`$(...)`) | Quoting nightmare | Use `capture(run(...))` | Explicit and safe |
| Unquoted special chars | `$5` becomes 5th argument | Literal `$5` | Just write it: `"Price: $5"` |
| Option injection (`-rf`) | Interpreted as flags | Remains an argument | `run(...)` quotes correctly |

---

## The bottom line

Bash's expansion rules made sense for interactive typing. You want `ls *.txt` to glob.

But in scripts? Those same rules become landmines. You review code, think it's correct, and then spaces or dollars or asterisks bite you at runtime.

sh2 flips the default:
- **Literal by default, expansion by intent.**
- You say what you mean, and it does what you said.

That's the whole idea.

---

# Docs

The GitHub repo is here:  
**[https://github.com/siu-mak/sh2lang](https://github.com/siu-mak/sh2lang)**

## Further Documentation

- [`docs/language.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/language.md) — full language reference (syntax + semantics)
- [`docs/sh2do.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/sh2do.md) — sh2do CLI documentation
- `tests/` — fixtures and integration tests (acts as an executable spec)
