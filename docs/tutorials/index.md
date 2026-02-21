---
title: "Tutorials"
---

# Tutorials

Welcome to the sh2 learning path! These hands-on guides will take you from writing your first script to building robust, verifiable CLI tools.

---

**Source Code & Repository**

👉 **[https://github.com/siu-mak/sh2lang](https://github.com/siu-mak/sh2lang)**

---

## Recommended Learning Path

1.  **[Getting Started](01-getting-started.md)**
    *   **Summary**: Write, compile, and run your first sh2 script and snippet.
    *   **You'll learn**: Installation, `sh2c`/`sh2do` basics, and the "no implicit expansion" safety rule.
    *   **Prereqs**: Basic terminal usage.

2.  **[Building a Real Tool](02-building-a-real-tool.md)**
    *   **Summary**: Build a backup rotation tool with argument parsing and validation.
    *   **You'll learn**: `argc()`/`arg(n)`, `if/else`, and basic file operations.
    *   **Prereqs**: Getting Started.

3.  **[Error Handling & Status](03-error-handling-and-status.md)**
    *   **Summary**: Master the "fail-fast" model and graceful error recovery.
    *   **You'll learn**: `allow_fail=true`, checking `status()`, and `try/catch` blocks.
    *   **Prereqs**: Building a Real Tool.

4.  **[Files & Directories](04-files-and-directories.md)**
    *   **Summary**: Safe file I/O and directory traversal without the quoting headaches.
    *   **You'll learn**: `read_file`, `write_file`, scoped `cwd(...)`, and iteration.
    *   **Prereqs**: Error Handling.

5.  **[Pipelines & Text Processing](05-pipelines-and-text.md)**
    *   **Summary**: Structured pipelines and text manipulation.
    *   **You'll learn**: `|` operator, `capture(...)`, and `split`/`join`/`trim`.
    *   **Prereqs**: Files & Directories.

## Advanced Topics

*   **[CI & Automation](06-ci-and-automation.md)**: Running sh2 in GitHub Actions and other CI environments.
*   **[Refactoring Bash](07-refactor-a-bash-script.md)**: A step-by-step guide to converting legacy Bash scripts to sh2.
*   **[Packaging](08-packaging-and-distribution.md)**: Distributing your sh2 tools to users.

## Jump to Reference

*   [Language Reference](../language.md) — Full syntax and semantics.
*   [sh2do CLI](../sh2do.md) — Snippet runner documentation.
