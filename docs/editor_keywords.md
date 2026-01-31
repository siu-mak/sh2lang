# sh2 Editor Keywords Reference

This document defines the canonical list of keywords, literals, and operators that editor integrations (VS Code, etc.) must highlight. The test suite validates VS Code artifacts against this list.

---

## Keywords (Control Flow / Declaration)

```
func let set if elif else for while break continue return
with try catch spawn wait case import exec run sh confirm
```

| Keyword | Category |
|---------|----------|
| `func` | function declaration |
| `let` | variable declaration |
| `set` | variable assignment |
| `if`, `elif`, `else` | conditionals |
| `for`, `while` | loops |
| `break`, `continue` | loop control |
| `return` | function return |
| `with` | scoped blocks (env, cwd, log, redirect) |
| `try`, `catch` | error handling |
| `spawn`, `wait` | background execution |
| `case` | pattern matching |
| `import` | module import |
| `exec` | process replacement |
| `run` | command execution |
| `sh` | raw shell |
| `confirm` | user confirmation |

---

## Boolean Literals

```
true false
```

---

## Operators / Syntax Tokens

```text
=> |> | & == != < > <= >= && || !
```

| Token | Meaning |
|-------|---------|
| `=>` | case arm arrow |
| `\|>` | block pipe |
| `\|` | command pipe |
| `&` | string concat |
| `==`, `!=` | equality |
| `<`, `>`, `<=`, `>=` | comparison |
| `&&`, `\|\|` | logical and/or |
| `!` | logical not |

---

## Comment Marker

```
# single-line comment
```

---

## Maintenance

When adding new keywords to the language:
1. Update this file
2. Update `editors/vscode/syntaxes/sh2.tmLanguage.json`
3. Run `cargo test -p sh2c --test editor_vscode_regression`

---
# Docs

The GitHub repo is here:  
**[https://github.com/siu-mak/sh2lang](https://github.com/siu-mak/sh2lang)**

## Further Documentation

- [`docs/language.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/language.md) — full language reference (syntax + semantics)
- [`docs/sh2do.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/sh2do.md) — sh2do CLI documentation
- `tests/` — fixtures and integration tests (acts as an executable spec)