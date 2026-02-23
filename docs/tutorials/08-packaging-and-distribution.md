---
title: "Packaging sh2 tools: compile targets, reviewable output, and distribution"
description: "How to ship sh2 scripts as normal .sh tools: sh2c output, bash vs posix targets, emit-only workflows, and team-friendly distribution."
---
<a href="https://github.com/siu-mak/sh2lang">
  <img src="../images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# Packaging and Distribution

This tutorial teaches you how to package sh2 tools for distribution: compiling to shell scripts, choosing targets, and shipping tools to teammates.

**What you'll learn:**
- How sh2c transforms `.sh2` into `.sh`
- Emit output for code review (CI, PRs)
- Choose between `--target bash` and `--target posix`
- Structure repos for sh2 tools
- Distribution strategies: source-only vs pre-compiled
- Version pinning and release hygiene

**Prerequisites:** Complete [Tutorial 01: Getting Started](01-getting-started.md).

---

## 1. What "Packaging" Means for sh2

sh2 doesn't have a package manager or runtime. It compiles to plain shell scripts.

**Distribution is simple:** You give someone a `.sh` file, and they run it. No dependencies beyond a POSIX or Bash shell.

The workflow:

```
source.sh2  →  sh2c  →  tool.sh  →  run anywhere
```

Your options:
- Ship the compiled `.sh` files directly
- Ship `.sh2` sources and let users compile
- Compile in CI and publish artifacts

---

## 2. Compile a Tool: `.sh2` → `.sh`

### Basic compilation

```bash
sh2c mytool.sh2 -o mytool.sh
```

This:
1. Parses `mytool.sh2`
2. Type-checks and validates
3. Emits `mytool.sh` (Bash by default)
4. Sets executable permission (`chmod +x`)

### Expected files

```
mytool.sh2     # Source
mytool.sh      # Compiled output (executable)
```

### Verify it works

```bash
./mytool.sh --help
```

### Compile multiple files

```bash
for f in tools/*.sh2; do
    sh2c "$f" -o "${f%.sh2}.sh"
done
```

---

## 3. Emit-Only for Review

In CI or code review workflows, you often want to generate output without running it.

### Using sh2c (recommended)

```bash
# Compile and emit to stdout (for inspection)
sh2c mytool.sh2

# Compile to file for commit/review
sh2c mytool.sh2 -o mytool.sh
```

### Check syntax only (no output)

```bash
sh2c --check mytool.sh2
```

This validates the script without emitting code. Useful for CI linting.

### CI workflow: generate and commit

```yaml
# .github/workflows/compile.yml
jobs:
  compile:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Build sh2c
        run: cargo build --release -p sh2c
      
      - name: Compile all tools
        run: |
          for f in tools/*.sh2; do
            ./target/release/sh2c "$f" -o "${f%.sh2}.sh"
          done
      
      - name: Check for changes
        run: git diff --exit-code tools/*.sh
```

If the generated `.sh` files differ from what's committed, the check fails—ensuring the compiled output stays in sync.

---

## 4. Choosing Targets: Bash vs POSIX

### `--target bash` (default)

Full feature set:
- Arrays and maps
- `with log(...)` fan-out
- Full `try_run(...)` capture
- Process substitution helpers

```bash
sh2c --target bash mytool.sh2 -o mytool.sh
```

### `--target posix`

Maximum portability:
- Works on dash, ash, busybox sh
- No arrays or maps
- No `with log(...)`
- Some features restricted

```bash
sh2c --target posix mytool.sh2 -o mytool.sh
```

### Compatibility checklist

| Feature | Bash | POSIX |
|---------|------|-------|
| `run(...)`, `capture(...)` | ✅ | ✅ |
| `allow_fail=true`, `status()` | ✅ | ✅ |
| `confirm(...)`, `sudo(...)` | ✅ | ✅ |
| `with cwd(...)`, `with env {...}` | ✅ | ✅ |
| `with redirect { stdout: file(...) }` | ✅ | ✅ |
| Multi-sink redirects `[file, inherit_stdout()]` | ✅ | ❌ |
| `with log(...)` | ✅ | ❌ |
| Lists `[a, b, c]` | ✅ | ❌ |
| Maps `{ k: v }` | ✅ | ❌ |
| `for x in list` | ✅ | ❌ (unless from `lines()`) |

### When to use POSIX

- Alpine/busybox containers
- Minimal Docker images
- Ancient servers without Bash
- Maximum portability requirements

### When to use Bash

- Most Linux desktops/servers
- macOS (ships with Bash)
- When you need arrays/maps
- When portability isn't critical

---

## 5. Recommended Repo Layout

```
my-tools/
├── tools/
│   ├── deploy.sh2
│   ├── deploy.sh       # Generated
│   ├── restart.sh2
│   └── restart.sh      # Generated
├── lib/
│   └── common.sh2      # Shared imports
├── tests/
│   └── test_deploy.sh
├── docs/
│   └── deploy.md
├── logs/               # Gitignored
│   └── .gitkeep
├── Makefile            # Or scripts/build.sh
├── README.md
└── .gitignore
```

### `.gitignore` example

```gitignore
# Logs directory (generated at runtime)
logs/*.log

# Optionally ignore generated .sh if compiling at install time
# tools/*.sh
```

### Makefile example

```makefile
SH2C := sh2c

SOURCES := $(wildcard tools/*.sh2)
TARGETS := $(SOURCES:.sh2=.sh)

.PHONY: all clean check

all: $(TARGETS)

tools/%.sh: tools/%.sh2
	$(SH2C) $< -o $@

check:
	@for f in $(SOURCES); do \
		echo "Checking $$f..."; \
		$(SH2C) --check "$$f"; \
	done

clean:
	rm -f $(TARGETS)
```

Usage:

```bash
make           # Compile all
make check     # Syntax check only
make clean     # Remove generated files
```

---

## 6. Distribution Strategies

### Strategy A: Commit only `.sh2`, compile in CI/release

**Workflow:**
1. Developers write `.sh2` files
2. CI compiles on each push
3. Release artifacts include compiled `.sh` files

**Pros:**
- No generated code in git history
- Single source of truth
- Forces compilation step (catches errors early)

**Cons:**
- Users need to compile or download release
- Extra CI step

**Best for:** Published tools, formal releases

### Strategy B: Commit both `.sh2` and `.sh`

**Workflow:**
1. Developers write `.sh2` files
2. Developers compile and commit `.sh` alongside
3. CI verifies they match (optional)

**Pros:**
- Users can run immediately (no build step)
- Easy to review diffs between source and output
- Works for teammates without sh2 installed

**Cons:**
- Potential for `.sh2` and `.sh` to get out of sync
- More files in repo

**Best for:** Team tools, internal utilities

### Strategy C: Publish via cargo install

For sh2c/sh2do themselves:

```bash
cargo install --git https://github.com/siu-mak/sh2lang sh2c
cargo install --git https://github.com/siu-mak/sh2lang sh2do
```

**Best for:** Developers who want the compiler itself

---

## 7. Versioning and Release Notes

### Pin to a specific version

Clone at a tag:

```bash
git clone --branch v0.1.2 https://github.com/siu-mak/sh2lang.git
```

Or in CI:

```yaml
- uses: actions/checkout@v4
  with:
    repository: siu-mak/sh2lang
    ref: v0.1.2
```

### Know which features exist

Check the release notes for your version:

- [v0.1.2 Release Notes](../releases/v0.1.2.md) — Range loops, job control, iterators
- [v0.1.1 Release Notes](../releases/v0.1.1.md) — `sudo(...)`, `confirm(...)`, semicolons
- [v0.1.0 Release Notes](../releases/v0.1.0.md) — Initial release

**Rule:** If a feature isn't in your version's release notes, don't use it.

### Keep docs stable

When distributing tools, include:
- The sh2 version used
- A link to the relevant docs
- Any known limitations

```markdown
# deploy.sh

Generated from `deploy.sh2` using sh2c v0.1.2.

Documentation: https://siu-mak.github.io/sh2lang/
```

---

## 8. Example: Complete Distribution Workflow

### Directory structure

```
ops-tools/
├── tools/
│   ├── deploy.sh2
│   └── restart.sh2
├── scripts/
│   └── build-all.sh
├── Makefile
└── README.md
```

### `scripts/build-all.sh`

```bash
#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

for f in tools/*.sh2; do
    echo "Compiling $f..."
    sh2c "$f" -o "${f%.sh2}.sh"
done

echo "Done. Compiled files:"
ls -la tools/*.sh
```

### `README.md` snippet

```markdown
## Building

Requires [sh2c](https://github.com/siu-mak/sh2lang) v0.1.2+.

```bash
# Build sh2c first
git clone https://github.com/siu-mak/sh2lang.git
cd sh2lang && cargo build --release -p sh2c
export PATH="$PWD/target/release:$PATH"

# Compile tools
cd /path/to/ops-tools
make
```

## Usage

```bash
./tools/deploy.sh production
./tools/restart.sh nginx
```
```

---

## 9. Mini Release Checklist

Before shipping an sh2 tool:

- [ ] **All `.sh2` files compile** — `sh2c --check` passes
- [ ] **Tests pass** — Run integration tests if you have them
- [ ] **Generated `.sh` files are up to date** — Recompile and check diff
- [ ] **Target is correct** — `--target bash` or `--target posix` as needed
- [ ] **Confirm defaults are CI-safe** — `default=false` for destructive ops
- [ ] **Sudo uses `n=true`** — No password prompts in automation
- [ ] **Logs directory exists or is created** — `run("mkdir", "-p", "logs")`
- [ ] **README documents dependencies** — sh2c version, shell requirements
- [ ] **Version is pinned** — Reference specific tag, not `main`
- [ ] **Release notes link is included** — Point to correct version docs
- [ ] **Example usage is provided** — Show 2–3 common invocations
- [ ] **License is clear** — If distributing externally

---

## 10. Quick Reference

### Compile commands

```bash
# Basic compile
sh2c tool.sh2 -o tool.sh

# Check syntax only
sh2c --check tool.sh2

# Target POSIX
sh2c --target posix tool.sh2 -o tool.sh

# Skip chmod +x
sh2c --no-chmod-x tool.sh2 -o tool.sh
```

### CI commands

```bash
# Compile all tools
for f in tools/*.sh2; do sh2c "$f" -o "${f%.sh2}.sh"; done

# Check all tools
for f in tools/*.sh2; do sh2c --check "$f"; done

# Verify no uncommitted changes
git diff --exit-code tools/*.sh
```

---

## Next Steps

You now know how to package and distribute sh2 tools.

### Related tutorials
- [Building a Real Tool](02-building-a-real-tool.md) — Build a complete tool
- [CI and Automation](06-ci-and-automation.md) — Run in CI/CD

### Reference
- [Language Reference](../language.md) — Full syntax
- [v0.1.2 Release Notes](../releases/v0.1.2.md) — Latest features

---

Happy shipping! 📦
