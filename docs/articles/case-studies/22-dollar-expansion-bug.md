---
title: "Bug story: when \"$FOO\" expanded — why strict literals matter"
description: "A real sh2 correctness bug, the fix, and the practical rules for writing safe strings going forward."
---
<a href="https://github.com/siu-mak/sh2lang">
  <img src="../../images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# Bug story: when "$FOO" expanded — why strict literals matter

A user ran this sh2 snippet:

```sh2
print("$FOO")
```

They expected it to print the literal text `$FOO`.

Instead, with `FOO=EXPANDED` in the environment, it printed `EXPANDED`.

This wasn't supposed to happen. sh2's core promise is **no implicit expansion**: what you write is what you get. But the generated Bash code was allowing shell parameter expansion to slip through.

This is the story of a P0 correctness bug, the fix, and the rules for writing strings going forward.

---

## The intended behavior

sh2's design says:

> String literals (`"..."`) are **strict literals**. They do not expand variables, globs, or shell metacharacters.

This means:

```sh2
print("$FOO")           // Should print: $FOO
print("${Package}")     // Should print: ${Package}
print("Price: $5")      // Should print: Price: $5
run("echo", "*")        // Should pass literal * to echo
```

The user writes `"$FOO"`, the script prints `$FOO`. Simple.

---

## The minimal repros

### Repro 1: print("$FOO")

**File:** `repro_dollar_expansion_print.sh2`
```sh2
func main() {
    print("$FOO")
}
```

**Running with the bug:**
```bash
FOO=EXPANDED sh2do repro_dollar_expansion_print.sh2
```

**Expected output:**
```
$FOO
```

**Actual output (before fix):**
```
EXPANDED
```

### Repro 2: dpkg-query format string

**File:** `string_braced_no_expand_run_dpkg_query.sh2`
```sh2
func main() {
    run("dpkg-query", "-W", "-f", "${Package}\n", "bash", allow_fail=true)
    print("status=" & status())
}
```

**Running with the bug:**
```bash
Package=BAD sh2do string_braced_no_expand_run_dpkg_query.sh2
```

**Expected output:**
```
bash
status=0
```

**Actual output (before fix):**
```
BAD
status=0
```

The format string `${Package}` was being expanded as a shell variable instead of being passed literally to `dpkg-query`.

---

## What went wrong

The root cause was in code generation. When sh2 compiled string literals to Bash, it was emitting them in a form that still allowed shell expansion.

For example, `print("$FOO")` was generating something like:

```bash
echo "$FOO"
```

The double quotes in Bash allow parameter expansion. `$FOO` gets replaced with the environment value.

The fix was to emit string literals in **single quotes** (or with proper escaping), so the output becomes:

```bash
echo '$FOO'
```

Single quotes in Bash prevent *all* expansion. The literal text passes through unchanged.

---

## Why this matters: real strings containing `$`

The `$` character appears in many legitimate contexts:

### 1. dpkg-query format strings

```sh2
run("dpkg-query", "-W", "-f", "${Package} ${Version}\n", "bash")
```

dpkg uses `${...}` as its own template syntax. If shell expansion happened, the query would break.

### 2. Template snippets

```sh2
let template = "Hello ${name}, your balance is $${amount}"
```

Generating templates for other languages (Terraform, CloudFormation, etc.) often uses `${}` syntax.

### 3. Regex patterns

```sh2
run("grep", "-E", "\\$[0-9]+", "prices.txt")
```

Regex patterns frequently contain `$` for end-of-line or literal dollar signs.

### 4. JSON with dollar signs

```sh2
let json = "{\"price\": \"$5.00\", \"expr\": \"${item}\"}"
```

JSON payloads may contain dollar signs as data.

### 5. Console output

```sh2
print("Price: $5 per unit")
print("Variable syntax: $VAR or ${VAR}")
```

User-facing messages should print exactly what you wrote.

In all these cases, the user expects literal output. Unexpected expansion is a correctness bug.

---

## The fix

From the v0.1.1 release notes:

> **P0 Fix (Breaking Change / Correctness Fix)**: String literals (`"..."`) are now **strict literals**. They do **not** support implicit variable interpolation or Bash parameter expansion.
>
> - `"$foo"` and `${bar}` in string literals are preserved as literal text (e.g. `print("$foo")` prints `$foo`).
> - To use variables, use **concatenation** (`"Hello " & name`) or **explicit interpolation** (`$"Hello {name}"`).
> - This change ensures that strings like `"$5"` or `"*"` are strictly safe and never trigger unintended Bash behavior.

The fix enforces the original design: strings are literal, expansion is explicit.

---

## How to write strings now

### Literal strings (no expansion)

These just work:

```sh2
print("$FOO")                    // Prints: $FOO
print("${Package}")              // Prints: ${Package}
print("Price: $5")               // Prints: Price: $5
run("echo", "*")                 // Passes literal * to echo
```

### Dynamic strings (with variables)

Use **concatenation** with `&`:

```sh2
let name = "Alice"
print("Hello " & name)           // Prints: Hello Alice
print("User: " & name & "!")     // Prints: User: Alice!
```

### Dynamic strings with interpolation *(v0.1.1+)*

Use **explicit interpolation** with `$"..."`:

```sh2
let name = "Alice"
print($"Hello {name}")           // Prints: Hello Alice
print($"Sum: {1 + 2}")           // Prints: Sum: 3
```

The `$` prefix signals interpolation. Braces `{...}` mark expression holes.

> **Note:** `$"..."` interpolation was added in v0.1.1.

### Mixing literals and variables

```sh2
let user = "alice"
let msg = "User=" & user & ", HOME=$HOME"
print(msg)
// Prints: User=alice, HOME=$HOME
// (The literal $HOME is NOT expanded)
```

### Shell parsing escape hatch

If you genuinely need shell expansion, use `sh("...")`:

```sh2
# sh(...) because: intentionally demonstrating shell variable expansion
sh("echo $HOME")
// Prints the actual value of HOME (shell expansion inside sh)
```

Inside `sh("...")`, you're in shell-land. Expansion happens. Use this when you want it.

---

## Before and after comparison

### Example 1: print("$FOO")

**Before fix:**
```
$ FOO=EXPANDED sh2do -e 'func main() { print("$FOO") }'
EXPANDED
```

**After fix:**
```
$ FOO=EXPANDED sh2do -e 'func main() { print("$FOO") }'
$FOO
```

### Example 2: dpkg-query format string

**Before fix:**
```
$ Package=BAD sh2do string_braced_no_expand.sh2
BAD
status=0
```

**After fix:**
```
$ Package=BAD sh2do string_braced_no_expand.sh2
bash
status=0
```

### Example 3: Asterisk literal

**Before fix (potential):**
```sh2
run("echo", "*")
// Might glob to: file1.txt file2.txt ...
```

**After fix:**
```sh2
run("echo", "*")
// Prints: *
```

---

## Tests that lock this down

The following tests exist to prevent regression:

### 1. `test_string_dollar_no_expand_print`

```sh2
// repro_dollar_expansion_print.sh2
func main() {
    print("$FOO")
}
```

**Assertion:** With `FOO=EXPANDED`, output must be `$FOO`.

### 2. `test_string_braced_no_expand_run_dpkg_query`

```sh2
// string_braced_no_expand_run_dpkg_query.sh2
func main() {
    run("dpkg-query", "-W", "-f", "${Package}\n", "bash", allow_fail=true)
    print("status=" & status())
}
```

**Assertion:** With `Package=BAD`, output must contain `bash` and NOT contain `BAD`.

### 3. `test_string_braced_no_expand_run_printf`

```sh2
// string_braced_no_expand_run_printf.sh2
func main() {
    run("printf", "%s\n", "${Package}")
}
```

**Assertion:** With `Package=BAD`, output must be `${Package}`.

### 4. Hostile string guardrails

```sh2
// guardrail_hostile_strings.sh2
let dollar_str = "$HOME and $USER"
run("echo", dollar_str)
```

**Assertion:** Output is literally `$HOME and $USER`.

---

## The rules, summarized

| What you write | What happens |
|----------------|--------------|
| `"$FOO"` | Literal `$FOO` (no expansion) |
| `"${Package}"` | Literal `${Package}` (no expansion) |
| `"*"` | Literal `*` (no glob) |
| `"~"` | Literal `~` (no tilde expansion) |
| `"Hello " & name` | Concatenation: `Hello ` + value of `name` |
| `$"Hello {name}"` | Interpolation: `Hello ` + value of `name` |
| `sh("echo $FOO")` | Shell expansion: actual value of `FOO` |

**The principle:**

1. **Strings are literal.** What you write is what you get.
2. **Interpolation is explicit.** Use `&` or `$"..."`.
3. **`sh("...")` is the escape hatch.** You're opting into shell parsing.

---

## Closing

This bug was a violation of sh2's core design. The fix restores the original intent: a string literal should be literal.

If you're migrating older sh2 code that relied on implicit expansion, the fix is straightforward: use `&` concatenation or `$"..."` interpolation where you want dynamic values.

And if you're writing new code: just write your strings. They'll do what you expect.

---

# Docs

The GitHub repo is here:  
**[https://github.com/siu-mak/sh2lang](https://github.com/siu-mak/sh2lang)**

## Further Documentation

- [`docs/language.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/language.md) — full language reference (syntax + semantics)
- [`docs/releases/v0.1.1.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/releases/v0.1.1.md) — release notes including this fix
- `tests/` — fixtures and integration tests (acts as an executable spec)
