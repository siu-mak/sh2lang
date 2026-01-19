# Contributing to sh2lang

Thanks for helping improve sh2c/sh2do.

## Where to report issues
- Bugs / feature requests: GitHub Issues (use the templates).

## What to include in bug reports
Please include:
- **Commit / version**:
  - If built from source: `git rev-parse --short HEAD`
  - If installed via cargo: `cargo install --list | grep -E '^sh2c|^sh2do'` (if applicable)
- **Target**: `--target bash` or `--target posix`
- **OS / shell**:
  - `uname -a`
  - `bash --version` (or `dash --version` if relevant)
- **Minimal repro**: the smallest `.sh2` that reproduces it
- **Exact command run** and **stdout/stderr**
- **Expected vs actual behavior**

## Local development

### Build
```bash
cargo build --workspace
```

### Fast tests (during iteration)
Run only what you touched:

```bash
cargo test -p sh2c --test <test_name>
cargo test -p sh2do --test <test_name>
```

### Release gate (before opening a PR)
```bash
cargo test --workspace --all-features
cargo test -p sh2c --test ci_posix_shell_matrix
cargo test -p sh2c --test cli_target
```

## Pull requests
- Keep changes focused and small.
- Add/adjust fixtures and tests for any behavior change.
- Prefer precise error messages over panics.
- Avoid introducing bash-only behavior in `--target posix` paths unless documented.
