## sh2 — A Semantic Shell Language

**Project Status:** Prototype / WIP (spec-first, implementation evolving)  
**Language Version:** 0.1 (Draft)

**Audience:** Compiler implementers, contributors, reviewers

`sh2` is the *language/spec*. `sh2c` is this repository’s *reference compiler* that compiles `.sh2` programs to shell scripts.

---

## 1. Project Overview

**sh2** is a spec-first language with an evolving formal definition.

sh2 exists to solve long-standing problems with traditional shell scripting:

* Meaning is encoded in punctuation (`>&2`, `$@`, `[[ ]]`)
* Scripts are difficult to read, review, and refactor
* Errors are subtle and often silent
* Tooling support is weak due to implicit semantics

sh2 replaces symbolic shell syntax with **structured language constructs** while preserving full compatibility with existing shell environments.

---

## 2. Project Goals

### Primary Goals

* Define a **human-readable shell language**
* Provide a **formal language specification**
* Compile to **POSIX shell by default**
* Support **full Bash expressiveness** via structured features and escape hatches
* Enable **multiple independent implementations**

### Explicit Non-Goals

* Replacing interactive shells
* Re-implementing all Bash internals
* Becoming a general-purpose programming language
* Runtime interpretation (sh2 is compiled)

---

## 3. Language Definition Status

The authoritative definitions are:

| Document              | Purpose                           |
| --------------------- | --------------------------------- |
| `docs/language.md`    | Normative language specification  |
| `docs/grammar.ebnf`   | Formal grammar                    |
| `docs/sh2_vs_shell.md`| rationale / comparison with traditional shell |

**Note:** The compiler implementation (`sh2c`) is not the language definition; it is one implementation that should track the spec and tests.

---

## 4. Language Scope (v0.1 Draft) 

Bash completeness policy: any Bash can be expressed via `sh(...)` / `raw { ... }` even if not all constructs are first-class yet:
* **Structurally**, using first-class constructs (`if`, `case`, `with`, pipes, redirects, loops), or
* **Explicitly**, using escape hatches (`sh("...")`, `raw { ... }`)

### Core Constructs

* Functions and entry point (`func main`)
* Sequential execution
* Command execution (`run`, `cmd`)
* Output (`print`, `print_err`)
* Conditionals (`if`, `elif`, `else`)
* Pattern matching (`case`)
* Loops (`while`, `for`)
* Scoped behavior (`with env`, `with cwd`, `with redirect`)
* Pipelines and background jobs
* Subshells and groups
* Escape hatches for raw shell

---

## 5. Architecture Overview

sh2 follows a **compiler pipeline architecture**:

```
.sh2 source
   ↓
Lexer
   ↓
Parser (AST)
   ↓
Lowering (IR)
   ↓
Codegen
   ↓
POSIX shell / Bash
```

### Design Principles

* Each phase has a single responsibility
* Semantics are encoded in IR, not syntax
* Code generation is mechanical and testable
* Shell quirks are isolated to codegen

---

## 6. Repository Structure

The repository is organized to reflect compiler phases and language boundaries:

```
.github/workflows/   CI
.vscode/             editor tasks (optional)
docs/                spec + grammar
src/                 compiler implementation
  parser/            parser modules
  bin/               (misc bin helpers, if any)
tests/
  common/            shared test helpers
  fixtures/          golden fixtures / test inputs
  *.rs               integration test suites
```

Rust implementation files live in `src/`:
- `lexer.rs`, `parser/`, `ast.rs`, `lower.rs`, `ir.rs`, `codegen.rs`
- `span.rs` for diagnostics/spans
- `loader.rs` for file/module loading
- `main.rs` / `lib.rs` for CLI + library entrypoints

---

## 7. Testing Philosophy

sh2 uses **language-level testing**, not just unit tests.

Test categories:

| Category   | Purpose                   |
| ---------- | ------------------------- |
| `parse`    | Grammar correctness       |
| `semantic` | Static rule enforcement   |
| `codegen`  | Shell output structure    |
| `exec`     | Runtime behavior          |
| `reject`   | Invalid program rejection |

Golden-file testing is mandatory for code generation.

---

## 8. Bash Completeness Policy

Bash completeness policy: any Bash can be expressed via `sh(...)` / `raw { ... }` even if not all constructs are first-class yet:

* Structured language constructs for common shell behavior
* Mandatory escape hatch support (`sh(...)` or `raw {}`)

This ensures:

* Any Bash script can be represented in sh2
* sh2 does not need to model every Bash quirk natively

---

## 9. Contribution Model

### Who Should Contribute

* Compiler engineers
* Language designers
* Shell users with production experience
* Tooling and CI engineers

### Contribution Expectations

* All language changes require:

  * specification updates
  * fixtures + integration tests
* Implementation changes without spec updates are discouraged
* Design discussion precedes implementation

---

## 10. Language Evolution Policy

* sh2 follows **spec-first development**
* New syntax requires:

  * a spec proposal
  * grammar update
  * fixtures + integration tests
* Breaking changes require a version bump

---

## 11. Success Criteria

The project is considered successful if:

* The language can be reimplemented from the spec alone
* Multiple independent compilers can exist
* Generated shell scripts are readable and reviewable
* Contributors reason about semantics, not shell punctuation

---

## 12. Project Status Summary

| Area               | Status      |
| ------------------ | ----------- |
| Language Spec      | Defined     |
| Grammar            | Defined     |
| Reference Compiler | Prototype   |
| CI Integration     | In progress |

---

