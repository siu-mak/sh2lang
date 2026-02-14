---
title: "Articles"
---

# Articles

Deep dives into the design, features, and philosophy of sh2.

---

**Source Code & Repository**

👉 https://github.com/siu-mak/sh2lang

---

## Introduction

Why sh2 exists and how it compares to other tools.

*   **[Diff: sh2 vs Bash](introduction/01-diff-sh2-vs-bash.md)**
    *   Side-by-side code comparisons showing how sh2 handles common tasks differently (and safer) than Bash.
*   **[Bash One-Liners Ironed Out](introduction/02-bash-oneliners-ironed-out.md)**
    *   How sh2 transforms fragile shell idioms into robust, readable code.
*   **[From One-Liner to Tool](introduction/03-from-oneliner-to-tool.md)**
    *   A walkthrough of growing a simple command into a full CLI tool.
*   **[Where Bash Still Wins](introduction/04-where-bash-still-wins.md)**
    *   Honest assessment of when stick with pure Bash vs switching to sh2.
*   **[The Mental Model](introduction/05-sh2-mental-model.md)**
    *   How to think in sh2: separation of concerns, side-effect scoping, and data flow.
*   **[Bash vs Python vs sh2](introduction/06-bash-vs-python-vs-sh2.md)**
    *   Choosing the right tool for the job: glue code vs application logic.

## Features

Detailed guides for specific language capabilities.

*   **[sudo Builtin](features/11-sudo-builtin.md)**
    *   Using the structured `sudo(...)` wrapper for safe privilege escalation.
*   **[confirm Helper](features/12-confirm-helper.md)**
    *   Interactive prompts and CI/automation handling.
*   **[No Implicit Expansion](features/13-no-implicit-expansion.md)**
    *   The core safety rule: why strings are strict literals.
*   **[Named Arguments](features/14-named-arguments.md)**
    *   How sh2 uses named parameters for clarity and safety.
*   **[Error Handling](features/15-error-handling.md)**
    *   Fail-fast defaults, `allow_fail`, and `try/catch`.
*   **[Logging & Redirects](features/16-logging-and-redirects.md)**
    *   Scoped I/O redirection and structured logging.

## Case Studies

Real-world examples and post-mortems.

*   **[The Hidden Tax of Reviewing Bash](case-studies/21-hidden-tax-reviewing-bash.md)**
    *   Why reviewing shell scripts is hard and how structure helps.
*   **[The Dollar Expansion Bug](case-studies/22-dollar-expansion-bug.md)**
    *   An analysis of a common shell bug prevented by sh2 design.
*   **[Video Randomizer](case-studies/23-video-randomizer-oneliner.md)**
    *   Building a fun, non-trivial tool with sh2.

## Suggested Reading Routes

### New to Shell Scripting

1.  Start with **[Tutorials: Getting Started](../tutorials/01-getting-started.md)**.
2.  Read **[The Mental Model](introduction/05-sh2-mental-model.md)**.
3.  Explore **[Error Handling](features/15-error-handling.md)**.

### Bash Power User

1.  Check **[Diff: sh2 vs Bash](introduction/01-diff-sh2-vs-bash.md)**.
2.  Read **[Where Bash Still Wins](introduction/04-where-bash-still-wins.md)**.
3.  Deep dive into **[No Implicit Expansion](features/13-no-implicit-expansion.md)**.
4.  See **[The Hidden Tax of Reviewing Bash](case-studies/21-hidden-tax-reviewing-bash.md)**.

### Script Maintainer / Reviewer

1.  Review **[The Hidden Tax of Reviewing Bash](case-studies/21-hidden-tax-reviewing-bash.md)**.
2.  Learn about **[Error Handling](features/15-error-handling.md)** and **[Logging](features/16-logging-and-redirects.md)**.
3.  See **[CI & Automation](../tutorials/06-ci-and-automation.md)** (Tutorial).
