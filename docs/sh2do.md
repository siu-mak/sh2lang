# sh2do — sh2 Snippet Runner

sh2do is a thin CLI wrapper around sh2c that compiles and executes sh2 snippets in one step. It wraps your snippet into `func main() { ... }`, invokes sh2c, and runs the generated shell script. sh2do does not change sh2 language semantics.

## Install

From the workspace root:

```bash
cargo build
```

During development:

```bash
target/debug/sh2do --help
```

## Usage

### Snippet as argument

```bash
sh2do 'print("hello world")'
```

### Snippet from stdin

```bash
echo 'print("hello")' | sh2do -
```

### With flags and arguments

```bash
sh2do 'print(arg(1))' --target posix -- myarg
```

## Flags

### `--emit-sh`

Compile and emit the generated shell script to stdout without executing it.

```bash
sh2do 'print("hi")' --emit-sh
```

### `--no-exec`

Alias of `--emit-sh`. Behaves identically.

### `--target <bash|posix>`

Select the target shell dialect. Default is `bash`.

```bash
sh2do 'print("test")' --target posix
```

### `-h, --help`

Show help text and exit.

```bash
sh2do --help
```

## Arguments Passthrough

Everything after `--` is passed verbatim to the executed script. These arguments are accessible via `arg(n)` and `argc()` in your sh2 snippet.

```bash
sh2do 'print(arg(1))' -- hello
# Output: hello

sh2do 'print(argc())' -- a b c
# Output: 3
```

Arguments are ignored in `--emit-sh` / `--no-exec` mode (no execution occurs).

## Exit Status

### Compile errors

If sh2c fails to compile the snippet, sh2do exits with sh2c's exit code and forwards stderr unchanged.

### Runtime errors

If the generated script executes and fails, sh2do exits with the script's exit code.

### Success

Exit code 0 indicates successful compilation and execution (or successful emit-only mode).

## Examples

### Basic execution

```bash
sh2do 'print("hello world")'
```

### Using arguments

```bash
sh2do 'print("Hello, " & arg(1))' -- Alice
```

### Emit shell without execution

```bash
sh2do 'print("test")' --emit-sh > script.sh
```

### POSIX target

```bash
sh2do 'print("portable")' --target posix
```

### Stdin mode with arguments

```bash
echo 'print(arg(1) & " " & arg(2))' | sh2do - -- foo bar
```

### Check generated shell

```bash
sh2do 'run("echo", "hi")' --emit-sh | head -20
```

## Non-Goals

sh2do is intentionally minimal:

- **No REPL**: sh2do is not an interactive shell
- **No new syntax**: sh2do does not extend sh2 language
- **No implicit helpers**: No automatic imports or magic variables
- **No smart rewriting**: Snippets are wrapped as-is into `func main() { ... }`

For full control over compilation, use `sh2c` directly.
