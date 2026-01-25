# sh2 Feature → Test Matrix

This document maps implemented features to their proving test files, ensuring documentation stays in sync with tested behavior.

---

## Core Language

| Feature | Description | Test File(s) |
|---------|-------------|--------------|
| Program structure | imports + func only | `syntax_toplevel.rs`, `syntax_import.rs` |
| Named parameters | `func foo(a, b)` | `syntax_functions.rs`, `syntax_proc_params.rs` |
| Statement separation | semicolons or newlines | `syntax_toplevel.rs`, `syntax_semicolon.rs` |
| Comments | `# ...` | `syntax_toplevel.rs` |
| Heredocs | `<<EOF ... EOF` | `syntax_heredoc.rs` |

---

## Data Types

| Feature | Description | Test File(s) | Target |
|---------|-------------|--------------|--------|
| Strings | `"..."`, interpolation | `syntax_interp_*.rs`, `syntax_interpolated_*.rs`, `syntax_quoting*.rs` | both |
| Multiline strings | `"""..."""` | `syntax_multiline_strings.rs` | both |
| Numbers | `42`, arithmetic | `syntax_arith_exec.rs` | both |
| Numeric comparisons | `<`, `>`, `<=`, `>=` | `syntax_numeric_compare.rs` | both |
| Booleans | `true`/`false` | `syntax_bool_*.rs` | both |
| Lists | `[a, b]` | `syntax_list_ops.rs` | bash only |
| Maps | `{"k": v}` | `syntax_map.rs` | bash only |

---

## Command Execution

| Feature | Description | Test File(s) | Notes |
|---------|-------------|--------------|-------|
| `run(...)` | safe command exec | `syntax_run_allow_fail.rs`, `syntax_cmd_discovery.rs` | expression |
| `run(..., allow_fail=true)` | non-aborting | `syntax_run_allow_fail.rs` | |
| `exec(...)` | process replace | `syntax_exec*.rs` | statement |
| `sh(expr)` | raw shell (any expr) | `syntax_sh_expr_probe.rs`, `syntax_sh_probe_semantics.rs` | probe semantics |
| `sh { block }` | raw shell block | `syntax_sh_block_semantics.rs` | fail-fast |
| `capture(...)` | stdout capture | `syntax_capture_pipe.rs`, `syntax_cmd_sub.rs` | |
| `capture(..., allow_fail=true)` | non-aborting capture | `syntax_capture.rs` (fixture: `capture_allow_fail.sh2`) | `.status`, `.stdout`, `.stderr` |
| `try_run(...)` | result object | `syntax_try_run.rs` | `.status`, `.stdout`, `.stderr` |
| `sudo(cmd, ...)` | sudo wrapper | `syntax_sudo.rs` | statement & capture support |

---

## Pipelines

| Feature | Description | Test File(s) |
|---------|-------------|--------------|
| `cmd \| cmd` | command pipes | `syntax_pipe.rs`, `syntax_pipe_propagation.rs` |
| `{ block } \|> { block }` | block pipes | `syntax_pipe_blocks_basic.rs`, `syntax_pipe_blocks_mixed_left_stmt.rs`, `syntax_pipe_blocks_mixed_right_stmt.rs` |

---

## Spawn and Wait

| Feature | Description | Test File(s) |
|---------|-------------|--------------|
| `spawn { }` | background execution | `syntax_spawn_block_basic.rs`, `syntax_spawn_wait_status_pid.rs`, `syntax_pid_wait_spawn.rs` |
| `wait` / `wait pid` | wait for jobs | `syntax_wait_list_basic.rs`, `syntax_wait_pid_basic.rs` |

---

## Status and Errors

| Feature | Description | Test File(s) |
|---------|-------------|--------------|
| `status()` | last exit code | `syntax_status*.rs` |
| `try { } catch { }` | error handling | `syntax_error_handling.rs` |

---

## Control Flow

| Feature | Description | Test File(s) |
|---------|-------------|--------------|
| `if/elif/else` | conditionals | `syntax_control_flow.rs` |
| `while` | loops | `syntax_control_flow.rs` |
| `for x in expr` | iteration | `syntax_control_flow.rs` |
| `for (k,v) in map` | map iteration | `syntax_map.rs` |
| `break/continue` | loop control | `syntax_control_flow.rs` |
| `case` with `=>` | pattern match | `syntax_case.rs`, `syntax_case_glob.rs` |
| `and`/`or` operators | boolean logic | `syntax_and_or_stmt.rs` |

---

## Variable Assignment

| Feature | Description | Test File(s) |
|---------|-------------|--------------|
| `let x = expr` | variable binding | `syntax_arith_exec.rs` |
| `set x = expr` | variable update | `syntax_set.rs`, `syntax_set_var_basic.rs` |
| `set env.X = expr` | env update | `syntax_set_env_basic.rs`, `syntax_set_env_and_read.rs` |
| `arg(expr)` | dynamic argument access | `syntax_arg_dynamic.rs` (fixture: `arg_dynamic_loop.sh2`) |
| `sh(cmd, ...)` | shell command helper | `syntax_sh.rs` (fixtures: `sh_basic.sh2`, `sh_allow_fail_capture.sh2`, `sh_allow_fail_stmt.sh2`, `sh_shell_option.sh2`) |
| `confirm(prompt, default=...)` | yes/no prompt | `syntax_confirm.rs` (fixtures: `confirm_noninteractive_default_false.sh2`, `confirm_noninteractive_default_true.sh2`, `confirm_env_override_yes.sh2`, `confirm_env_override_no.sh2`) |

---

## Scoped Blocks (`with`)

| Feature | Description | Test File(s) | Target |
|---------|-------------|--------------|--------|
| `with env {...}` | scoped env | `syntax_with_env_colon_basic.rs` | both |
| `with cwd(...)` | scoped cwd | `syntax_io.rs`, `syntax_with_cwd.rs` | both |
| `with redirect {...}` | I/O redirect | `syntax_redirect_io.rs` | both |
| `with log(...)` | fan-out logging | `syntax_with_log.rs`, `syntax_logging.rs` | bash only |

---

## File I/O

| Feature | Description | Test File(s) | Target |
|---------|-------------|--------------|--------|
| `read_file(path)` | read file contents | `syntax_file_io.rs`, `syntax_read_file.rs` | both |
| `write_file(path, content)` | write/truncate | `syntax_file_io.rs`, `syntax_write_file.rs` | both |
| `append_file(path, content)` | append | `syntax_file_io.rs` | both |

---

## Filesystem Predicates

| Feature | Test File(s) |
|---------|--------------|
| `exists(path)` | `syntax_exists_isdir_isfile_basic.rs`, `syntax_fs_*.rs` |
| `is_dir(path)` | `syntax_fs_*.rs` |
| `is_file(path)` | `syntax_fs_*.rs` |
| `is_symlink(path)` | `syntax_fs_exists_dir_file_symlink_basic.rs` |
| `is_exec(path)` | `syntax_fs_exec_basic.rs` |
| `is_readable(path)` | `syntax_fs_readable_writable_basic.rs` |
| `is_writable(path)` | `syntax_fs_readable_writable_basic.rs` |
| `is_non_empty(path)` | `syntax_fs_non_empty_basic.rs` |

---

## String/List Utilities

| Feature | Test File(s) |
|---------|--------------|
| `split(str, sep)` | `syntax_split.rs` |
| `join(list, sep)` | `syntax_list_args_utils.rs` |
| `lines(str)` | `syntax_lines.rs` |
| `trim(str)` | `syntax_string_utils.rs` |
| `replace(str, old, new)` | `syntax_string_utils.rs` |
| `contains_line(text, needle)` | `syntax_contains_line.rs` |
| `contains(list, value)` | `syntax_contains.rs` |
| `matches(text, regex)` | `syntax_matches.rs` |

---

## Input/Confirm

| Feature | Description | Test File(s) |
|---------|-------------|--------------|
| `input(prompt)` | read user input | `syntax_input_confirm.rs` |
| `confirm(prompt)` | y/n confirmation | `syntax_input_confirm.rs` |

---

## JSON Utilities

| Feature | Description | Test File(s) | Target |
|---------|-------------|--------------|--------|
| JSON functions | parsing/encoding | `syntax_json.rs` | bash only |

---

## Argument Parsing

| Feature | Description | Test File(s) |
|---------|-------------|--------------|
| `parse_args()` | structured arg parsing | `syntax_parse_args.rs` |

---

## Path Utilities

| Feature | Description | Test File(s) |
|---------|-------------|--------------|
| `home()`, `path_join()` | path helpers | `syntax_path_helpers.rs` |

---

## Environment Variables

| Feature | Description | Test File(s) |
|---------|-------------|--------------|
| `env.X` | env access | `syntax_env.rs`, `syntax_envdot_shadow.rs` |
| `export/unset/source` | shell env ops | `syntax_env_export_unset_source.rs` |
| `load_envfile/save_envfile` | .env files | `syntax_envfile.rs` |

---

## Introspection Builtins

| Feature | Description | Test File(s) |
|---------|-------------|--------------|
| `type_of()`, `len()` | introspection | `syntax_introspection_builtins.rs` |

---

## Process/System Builtins

| Feature | Test File(s) |
|---------|--------------|
| `pid()`, `ppid()`, `uid()` | `syntax_uid_ppid_pwd_basic.rs`, `syntax_pid.rs` |
| `pwd()` | `syntax_uid_ppid_pwd_basic.rs` |
| `argc()`, `argv0()`, `arg(n)` | `syntax_argc_argv0_basic.rs`, `syntax_args_ops.rs` |
| `self_pid()` | `syntax_self_pid_arith.rs` |

---

## Target Differences (bash vs posix)

| Feature | Bash | POSIX | Test File(s) |
|---------|------|-------|--------------|
| Lists/Maps | ✓ | ✗ | `syntax_target.rs`, `cli_target.rs` |
| `with log` | ✓ | ✗ | `syntax_with_log.rs` |
| `env(expr)` dynamic | ✓ | ✗ | `syntax_env.rs` |
| `try_run` capture | full | limited | `syntax_try_run.rs` |

---

## CLI (sh2c)

| Feature | Test File(s) |
|---------|--------------|
| `--target bash/posix` | `cli_target.rs` |
| `-o/--out` file output | `cli_out_mode.rs` |
| `--check` syntax only | `cli_args.rs` |
| `--emit-ast/ir/sh` | `cli_introspection_flags.rs` |
| `--no-diagnostics` | `cli_no_diagnostics.rs` |
| `--help` | `cli_help_usage.rs` |

---

## CLI (sh2do)

| Feature | Test File(s) |
|---------|--------------|
| Snippet as argument | `sh2do_exec.rs` |
| Stdin mode (`-`) | `sh2do_exec.rs` |
| `--emit-sh` / `--no-exec` | `sh2do_emit.rs` |
| `--target` | `sh2do_emit.rs` |
| `-- args` passthrough | `sh2do_args.rs` |
| `-h/--help` | `sh2do_help.rs` |
