---
title: "What’s the difference between sh2 and bash?"
description: "A practical comparison of sh2 vs bash: quoting, args, structure, and error handling."
---
<a href="https://github.com/siu-mak/sh2lang">
  <img src="../../images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>


# What's the difference between sh2 and bash?

If you’ve ever written a bash script that “worked on your machine” and then exploded in CI, on a different distro, or inside a container, you already know the core problem:

**Bash is powerful, but it’s permissive in ways that make small mistakes turn into big surprises.**
**sh2 tries to keep the power of shell scripting, but adds structure so the same script is harder to misread, misquote, or mis-handle.**

Below is a practical, developer-facing breakdown of the differences.

---

## 1) Language goal: “everything is a string” vs “structure first”

### Bash

Bash grew as an interactive shell first. Its scripting model inherits a lot of “do what I mean” behaviors:

* words split by spaces unless quoted
* lots of implicit expansions
* command pipelines and conditionals are syntax-heavy and easy to get subtly wrong

This makes bash fast to write, but also easy to write **ambiguous code**.

### sh2

sh2 is aiming at **structured shell scripting**:

* statements like `let`, `if {}`, `while {}`, `case {}`, `for … in (…) {}` are explicit
* command execution is explicit via `run(...)`
* string building is explicit via `+`
* command substitution is explicit via `$(run(...))`

That “forced explicitness” is the whole point: **make intent visible and reduce accidental complexity.**

---

## 2) Command execution: free-form text vs explicit `run(...)`

### Bash

In bash, everything *looks* like text but is interpreted through multiple phases:

```bash
echo $x
grep "$pattern" file.txt
```

Tiny quoting differences can change meaning.

### sh2

In sh2, commands are constructed as **structured argument lists**:

```sh2
run("grep", pattern, "file.txt")
```

This tends to eliminate a huge class of issues:

* forgetting quotes
* accidental word splitting
* arguments that contain spaces

It also makes refactoring easier: reordering arguments is less error-prone than editing a long shell line.

---

## 3) String concatenation: bash expands everywhere, sh2 uses `+`

### Bash

In bash, string composition often mixes quoting rules and expansions:

```bash
path="$HOME/repos/$name"
```

You must remember when `$...` expands, when it doesn’t, and which quotes do what.

### sh2

sh2 makes concatenation explicit:

```sh2
let path = env.HOME + "/repos/" + name
```

The win is readability and fewer “wait, was that quoted?” moments.

---

## 4) Command substitution: `$(...)` vs `$(run(...))`

### Bash

Command substitution is built-in and very flexible:

```bash
out="$(whoami)"
```

…but bash will happily expand things you didn’t mean to expand if quoting isn’t right.

### sh2

sh2 makes it explicit that you’re capturing output from a command:

```sh2
let user = capture(run("whoami"))
```

---

## 5) Control flow: braces and blocks vs bash keywords and syntax traps

### Bash

Bash has multiple conditional syntaxes (`test`, `[ ]`, `[[ ]]`, arithmetic contexts) and they differ in behavior. It’s powerful, but inconsistent:

```bash
if [ "$x" = "go" ]; then
  ...
fi
```

### sh2

sh2 uses block structure consistently:

```sh2
while x == "go" {
  print("loop")
  let x = "stop"
}
```

This is easier to parse visually and makes nesting safer.

---

## 6) `case`: both have it, sh2’s is cleaner to write

### Bash

```bash
case "$x" in
  a) echo A ;;
  b|c) echo "B or C" ;;
  *) echo default ;;
esac
```

### sh2

```sh2
case x {
  "a" => { print("A") }
  "b" | "c" => { print("B or C") }
  _ => { print("default") }
}
```

Same concept, but sh2 reads more like a modern language.

---

## 7) Loops: explicit list iteration

### Bash

```bash
for x in "a" "b"; do
  echo "$x"
done
```

### sh2

```sh2
for x in ("a", "b") {
  print(x)
}
```

Again: fewer quoting foot-guns, clearer structure.

---

## 8) `return`, `break`, `continue`: similar idea, but sh2 reduces “where am I?” confusion

Bash lets you `return` from a function and `exit` the script; mixing those can be confusing in bigger scripts.

sh2 adopts explicit statements (`return`, `break`, `continue`) in the same structured style as other constructs, which helps readability.

One caution though: **Bash `return` expects a numeric status (0–255).** If sh2 allows `return "0"` and lowers it to `return "0"` in bash, that *usually works* because bash coerces, but it’s not a great semantic match. A stricter sh2 might:

* only allow numeric literals for `return`, or
* provide `exit(...)` / `status(...)` semantics explicitly

---

## 9) Error handling philosophy

### Bash

Error handling is optional and easy to forget. People use:

* `set -euo pipefail` (with caveats)
* `||` and `&&` chains
* manual `$?` checks

### sh2

Your current sh2 subset is trending toward:

* explicit constructs (`if`, comparisons, structured exec)
* future possibility of standard patterns for status propagation

This is where sh2 can become dramatically safer than bash, but only if the compiler/runtime semantics are nailed down.

---

## When should you use which?

### Bash is great when…

* it’s a tiny glue script
* you’re operating interactively
* you need maximal compatibility with existing shell idioms
* the script is short enough that “implicit bash rules” stay in your head

### sh2 is great when…

* scripts grow beyond a screen or two
* you need maintainability and refactoring safety
* lots of arguments/paths contain spaces or special characters
* you want predictable behavior across environments
* you’re building a “tooling language” on top of shell primitives

---

## A quick mental model

* **Bash**: a powerful *text-based* language with many implicit phases (tokenizing, expansion, splitting, globbing).
* **sh2**: a *structured* language that compiles to shell, trying to minimize implicit behavior by forcing intent into syntax.

If bash is a *knife*, sh2 is a *knife with a handle guard*: slightly more friction, fewer accidents.

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
- [v0.1.0](https://github.com/siu-mak/sh2lang/blob/main/docs/releases/v0.1.0.md) — First public release of the sh2 structured shell language.