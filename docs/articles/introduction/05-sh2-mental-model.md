---
title: "sh2 in plain English: the mental model (strings, arguments, and interpolation)"
description: "A practical mental model for predicting what sh2 will do: strict literals, explicit interpolation, safe arguments, and escape hatches."
---
<a href="https://github.com/siu-mak/sh2lang">
  <img src="../../images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# sh2 in plain English: the mental model

Have you ever written a Bash script that worked perfectly—until someone ran it with a filename that had a space? Or spent an hour debugging why `${Package}` got mangled by dpkg's format string?

```bash
# Looks fine. Explodes with spaces.
file="my report.txt"
grep pattern $file   #  Bash turns this into: grep pattern my report.txt
```

sh2 is designed so you can **predict what will happen** without memorizing Bash's quoting rules. This article gives you the mental model.

---

## The three rules

1. **Arguments are arguments.** If you write `run("cmd", a, b)`, the command receives exactly two arguments—no matter what `a` and `b` contain.
2. **Strings are strict literals.** What you type is what you get. `"*"` stays `"*"`. `"$FOO"` stays `"$FOO"`.
3. **Interpolation is explicit.** You opt in with `$"..."` or `&` concatenation. There's no magic.

The rest of this article shows each rule in action.

---

## Rule 1: Arguments are arguments

### Example 1: Spaces don't split

```sh2
let file = "my report.txt"
run("grep", "pattern", file)
# grep receives exactly 2 args: "pattern" and "my report.txt"
```

In Bash, you'd need `"$file"` and hope you didn't forget. In sh2, `run(...)` always passes each argument as a single value.

### Example 2: Wildcards don't glob

```sh2
run("echo", "*")
# Prints: *
```

There's no secret expansion. The asterisk goes to `echo` as-is.

### Example 3: Tilde is just a character

```sh2
run("ls", "~/Documents")
# ls receives the literal string "~/Documents"
# (This will fail because the path doesn't exist!)
```

If you want the home directory, use `env.HOME`:

```sh2
run("ls", env.HOME & "/Documents")
```

---

## Rule 2: Strings are strict literals

### Example 4: Dollar signs stay literal

```sh2
print("Price: $5")
# Output: Price: $5
```

### Example 5: Braced variables stay literal

```sh2
run("echo", "Current shell is ${SHELL}")
# echo receives the literal string "Current shell is ${SHELL}"
```

This is a lifesaver when you're passing format strings to tools or generating templates. No escaping required.

### Example 6: Both `$FOO` and `${FOO}` are safe

```sh2
let msg = "Hello $USER and ${HOME}"
print(msg)
# Output: Hello $USER and ${HOME}
```

sh2 never expands `$...` inside regular `"..."` strings.

---

## Rule 3: Interpolation is explicit

When you *want* variables inside strings, you ask for it.

### Option A: Use `&` (concatenation)

```sh2
let name = "Alice"
print("Hello " & name & "!")
# Output: Hello Alice!
```

### Option B: Use `$"..."` (explicit interpolation) — *v0.1.1+*

```sh2
let user = "admin"
print($"Welcome, {user}!")
# Output: Welcome, admin!
```

The `$"..."` syntax signals intent. Braces mark where variables go. No ambiguity.

### Example 7: Expressions in `$"..."`

```sh2
print($"Sum: {1 + 2}")
# Output: Sum: 3

print($"Current dir: {pwd()}")
# Output: Current dir: /path/to/here
```

### Example 8: Literal braces

```sh2
print($"Set notation: \{a, b\}")
# Output: Set notation: {a, b}
```

Use `\{` and `\}` to escape braces when you don't want interpolation.

> **Note:** String literals inside interpolation holes are not yet supported (e.g., `$"X: { "val" }"`). Use a variable as a workaround: `let v = "val"; print($"X: {v}")`.

---

## Named arguments: readable options

Bash flags are positional and cryptic. sh2 uses named arguments for clarity.

### Example 9: `confirm(default=false)`

```sh2
if confirm("Delete everything?", default=false) {
    run("rm", "-rf", "data/")
}
```

- If the script runs in CI (non-interactive), it proceeds with `false` (no deletion).
- You can override interactively.
- Environment variables `SH2_YES=1` or `SH2_NO=1` force the answer.

### Example 10: `sudo(...)` with named options (v0.1.1)

```sh2
sudo("apt-get", "update", n=true, user="root")
```

Instead of remembering `-n` vs `-u`, you write `n=true` and `user="root"`. The compiler generates `sudo -n -u root -- apt-get update` with the `--` separator automatically.

### Example 11: `env_keep=[...]`

```sh2
sudo("env", env_keep=["PATH", "HOME"])
# Generates: sudo --preserve-env=PATH,HOME -- env
```

Named arguments scale: add options without reordering positional flags.

---

## Capturing output and handling failure

### Example 12: `capture(...)` with `allow_fail=true` — *v0.1.1+*

```sh2
let out = capture(run("ls", "missing/"), allow_fail=true)
if status() != 0 {
    print("ls failed with code " & status())
} else {
    print(out)
}
```

- The script doesn't abort when `ls` fails.
- `status()` holds the exit code.
- `out` contains whatever `ls` wrote before failing.

---

## Before / After: A real footgun

### Bash (common bugs)

```bash
pattern="foo bar"
file="my data.txt"
msg='Price: $5'

grep $pattern "$file"                              # Bug 1: $pattern splits
echo "$msg"                                        # Bug 2: $5 expands to empty
rm *.bak                                           # Bug 3: glob might match nothing
```

**What can go wrong:**
1. `$pattern` becomes two arguments (`foo` and `bar`) unless quoted.
2. Bash tries to expand `$5` (empty).
3. `rm` behavior depends on shell options if no files match.

### sh2 (these bugs can't happen)

```sh2
let pattern = "foo bar"
let file = "my data.txt"
let msg = "Price: $5"

run("grep", pattern, file)                         # ✅ 2 args, no splitting
print(msg)                                         # ✅ $5 is literal
run("rm", "*.bak", allow_fail=true)                # ✅ "*.bak" is literal; passed to rm
```

**What disappeared:**
1. No quoting gymnastics.
2. No escaping `$` in strings.
3. No globbing surprises (unless you use `sh()`).

---

## The escape hatch: `sh("...")`

Sometimes you genuinely need shell features: pipes, process substitution, globs.

### Example 13: When you need real shell parsing

```sh2
# sh(...) because: glob expansion *.log
let summary = capture(sh("ls *.log | wc -l"), allow_fail=true)
if status() == 0 {
    print($"Found {trim(summary)} log files")
}
```

Inside `sh(...)`, you're back in shell-land. Globs expand. Variables expand if you write `$FOO`. The trade-off is you lose sh2's safety guarantees for that snippet.

Use `sh()` when:
- You need glob expansion (`*.log`)
- You need shell pipelines that sh2 doesn't yet express
- You're porting Bash incrementally

Avoid `sh()` when:
- You're handling user input (injection risk)
- You can express it with `run()` and structured pipes

---

## Rules you can remember

1. **If you see `run(...)`, you're safe from word splitting.** Each argument is exactly one argument.

2. **If you see `"..."`, nothing expands.** Dollar signs, braces, asterisks, tildes—all literal.

3. **If you want variables inside strings, use `$"..."` or `&`.** You choose when to interpolate.

4. **Use `env.HOME` instead of `~`.** Tilde is just a character.

5. **Named arguments replace flags.** `n=true, user="root"` instead of `-n -u root`.

6. **`allow_fail=true` makes failures checkable.** Combine with `status()` to handle errors explicitly.

7. **If you see `sh("...")`, you're back in shell-land.** All Bash rules apply inside that string.

8. **When in doubt, check what the compiler generates.** Run `sh2c --emit-sh your_script.sh2` to see the output.

---

# Docs

The GitHub repo is here:  
**[https://github.com/siu-mak/sh2lang](https://github.com/siu-mak/sh2lang)**

## Further Documentation

- [`docs/language.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/language.md) — full language reference (syntax + semantics)
- [`docs/sh2do.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/sh2do.md) — sh2do CLI documentation
- `tests/` — fixtures and integration tests (acts as an executable spec)
