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

## Fixture naming policy

### Convention
- Use **snake_case** for fixture directories and file basenames (e.g. `capture_status`, not `capture-status`).
- Avoid legacy suffix patterns like `.fs` in directory names unless there is a current technical requirement for them.

### Rename-on-touch
When you are already editing an existing fixture for another ticket, it is acceptable to opportunistically rename it to the preferred convention **in the same PR**, provided:
- All snapshot paths, test references, and expected-output files are updated together.
- The diff stays reviewable (the rename should be low-noise relative to the rest of the change).

Do **not** submit standalone bulk-rename PRs just to normalize old fixtures.

### Rationale
- Keeps git history and review noise low.
- Gradually improves naming consistency over time.
- Avoids mixing mechanical churn with behavior changes.

### Example
If you are already editing fixture `some-old-name.fs/` for another ticket, it is acceptable to rename it to `some_old_name/` in the same PR, as long as snapshots, paths, and tests are updated and the diff stays reviewable.

### Reviewer note
Reviewers should prefer opportunistic normalization over separate cleanup-only rename campaigns. If a PR touches a fixture that doesn't follow the convention, suggest renaming it in the same change rather than filing a follow-up ticket.
