## sh2 — A Semantic Shell Language

**Project Status:** Active / Language-defined
**Language Version:** 1.0 (Draft)
**Audience:** Compiler implementers, contributors, reviewers

---

## 1. Project Overview

**sh2** is a **compiled shell language** designed to express shell programs using
**explicit, semantic constructs**, compiling them into **portable POSIX shell** or **Bash**.

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

sh2 is a **formally defined language**.

The authoritative definitions are:

| Document              | Purpose                           |
| --------------------- | --------------------------------- |
| `docs/language.md`    | Normative language specification  |
| `docs/grammar.ebnf`   | Formal grammar                    |
| `docs/conformance.md` | Conformance and test requirements |

The compiler implementation is **not** the language definition; it is one possible implementation.

---

## 4. Language Scope (v1.0)

sh2 v1.0 can express the **entire Bash scripting space**, either:

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
src/
├── lexer.rs
├── parser.rs
├── ast.rs
├── ir.rs
├── lower.rs
├── codegen.rs
├── lib.rs
└── main.rs

docs/
├── language.md
├── grammar.ebnf
├── conformance.md
└── rationale.md

tests/
├── conformance/
│   ├── core/
│   ├── posix/
│   └── bash/
```

This structure is **normative** for new implementations.

---

## 7. Conformance Model

sh2 defines **formal conformance levels**:

| Level   | Description             |
| ------- | ----------------------- |
| `core`  | Mandatory language core |
| `posix` | POSIX shell emission    |
| `bash`  | Bash extensions         |

An implementation must declare which levels it supports and pass the corresponding test suites.

### Conformance Authority

If there is disagreement between:

* the implementation, and
* the language specification or tests

→ **The specification and conformance tests win.**

---

## 8. Testing Philosophy

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

## 9. Bash Completeness Policy

sh2 guarantees **Bash completeness** via:

* Structured language constructs for common shell behavior
* Mandatory escape hatch support (`sh(...)` or `raw {}`)

This ensures:

* Any Bash script can be represented in sh2
* sh2 does not need to model every Bash quirk natively

---

## 10. Contribution Model

### Who Should Contribute

* Compiler engineers
* Language designers
* Shell users with production experience
* Tooling and CI engineers

### Contribution Expectations

* All language changes require:

  * specification updates
  * conformance tests
* Implementation changes without spec updates are discouraged
* Design discussion precedes implementation

---

## 11. Language Evolution Policy

* sh2 follows **spec-first development**
* New syntax requires:

  * a spec proposal
  * grammar update
  * conformance tests
* Breaking changes require a version bump

---

## 12. Success Criteria

The project is considered successful if:

* The language can be reimplemented from the spec alone
* Multiple independent compilers can exist
* Generated shell scripts are readable and reviewable
* Contributors reason about semantics, not shell punctuation

---

## 13. Project Status Summary

| Area               | Status      |
| ------------------ | ----------- |
| Language Spec      | Defined     |
| Grammar            | Defined     |
| Conformance        | Defined     |
| Reference Compiler | Prototype   |
| CI Integration     | In progress |

---

