<a href="https://github.com/siu-mak/sh2lang">
  <img src="images/logo/sh2logo_256.png" alt="sh2 logo" width="128" />
</a>

# The sh2 Language Reference

[sh2](https://github.com/siu-mak/sh2lang) is a small, structured shell language designed to bring safety, clarity, and modern programming constructs to shell scripting. Scripts written in sh2 are compiled by **sh2c** into either **bash** (feature-rich) or **POSIX sh** (portable).

> **Note**: **sh2do** is a wrapper tool that compiles and executes sh2 snippets in one step. It does not change sh2 language semantics.

> **How to read this**: This is a technical reference. If you are new to sh2, start with the **[Tutorials](tutorials/index.md)**, specifically **[Getting Started](tutorials/01-getting-started.md)**.

---

## 1. Program Structure

A program consists of:

- zero or more `import "path" [as alias]` statements (must come first), and
- one or more `func ... { ... }` function definitions.

**Top-level executable statements are not allowed.** The compiler emits a shell entrypoint that invokes `main()`.

### 1.1 Imports

```sh2
import "lib/utils.sh2"
import "lib/fs.sh2" as fs
```

Imports are resolved relative to the current file. The `.sh2` extension is optional but recommended.
If you use `import "path" as alias`, you can call functions from that module using a **qualified call**.

- Imports must appear before any function definitions.
- Imports are resolved recursively.
- Import cycles are detected and reported.
- All imported functions share a single namespace; duplicate function names are an error (unless disambiguated by an alias).

### 1.2 Qualified Calls (Namespaced Functions)

When you import a file using an `as alias`, you can call its functions using the `alias.func(...)` syntax.

```sh2
import "lib/fs.sh2" as fs

func main() {
  # Statement form
  fs.mkdir("/tmp/foo")
  
  # Expression form (in an assignment)
  let home = fs.get_home()
  
  # In a capture/command substitution
  let files = capture(fs.list_dir("/tmp/foo"))
}
```

**Lazy Wrapper Emission**
sh2c emits mangled wrapper functions (e.g. `__imp_fs__mkdir`) for your qualified calls. This emission is **lazy**: wrappers are only generated for functions you actually reference. Importing a large library as an alias costs zero overhead if you don't use it.

**Restrictions on Qualified Calls:**
- **Only function calls are allowed.** You cannot access properties or fields (i.e. `fs.value` is an error).
- **No chaining.** You cannot chain namespaces (e.g. `a.b.c()`). You must import the target directly.
- **No bare references.** You cannot pass a function as a value (e.g. `let f = alias.func`).
- **No named arguments.** Qualified calls do not support named arguments (such as `allow_fail=true`), since user-defined functions only accept positional arguments.

### 1.2 Functions and Parameters

Functions are defined with **named parameters**, but arguments are passed **positionally**:

```sh2
func greet(name, title) {
  print("Hello, " & title & " " & name)
}
```

#### Argument Passing Policy

- **User-Defined Functions**: User-defined functions only accept **positional arguments**. Attempting to use `name=value` in a general function call will result in a compile error.
- **Builtins**: A specific set of builtins supports **named arguments** (options) for configuration. These include:
  - `run(...)`, `sudo(...)`, `sh(...)`
  - `capture(...)`
  - `confirm(...)`

Parameters are bound positionally for general functions (first param receives the first argument, etc.).

> **Compile-time validation**: Function calls in expression context are validated at compile time. Calling an undefined function (not user-defined or a builtin) produces a compile error with hints. To invoke external commands, use `run("cmd", ...)`.

The designated entry point is:

```sh2
func main() {
  run("echo", "hi")
}
```

### 1.3 Script Arguments

Top-level script arguments (the ones passed to the script itself) are accessed via `arg(n)` and `argc()`.

- **`argc()`**: Returns the number of arguments passed to the script.
- **`arg(n)`**: Returns the *n*-th argument (1-based index).

```sh2
func main() {
    print($"Script received {argc()} arguments")
    if argc() > 0 {
        print("First arg: " & arg(1))
    }
}
```

**Strict Validation**:
`arg(n)` enforces strict bounds and type checking at runtime to prevent injection and logic errors:
- **Index Type**: The index must be a valid integer. String values that do not look like integers (e.g. `"1a"`) cause a fatal error.
- **Bounds**: The index must be `>= 1` and `<= argc()`. Accessing an out-of-bounds index (e.g. `arg(0)` or `arg(argc()+1)`) aborts the script with a fatal error.
- **Safety**: `arg(i)` is safe to use with variable indices logic (e.g. `arg(i)` inside a loop). It uses robust internal helpers to prevent command injection even if the variable `i` is tampered with.

```sh2
let i = 1
while i <= argc() {
    print(arg(i))
    set i = i + 1
}
```

---

## 2. Core Syntax Rules

### 2.1 Statement separation
 
 Statements are separated by **newlines** or **semicolons** (`;`). Semicolons act as optional statement terminators.
 
 ✅ Correct:
 ```sh2
 func ok() {
   let a = "x"
   let b = "y"
   print("one"); print("two")
   print("three");
 }
 ```
 
 Semicolons are not allowed inside expressions:
 ❌ Incorrect:
 ```sh2
 let x = (1; 2)
 ```

### 2.2 Reserved identifiers

`env` is a reserved keyword (used for environment access like `env.HOME`) and cannot be used as a variable or function name.

✅
```sh2
let env_name = "dev"
```

❌
```sh2
let env = "dev"
```

### 2.3 Comments

Single-line comments start with `#`.

```sh2
# comment
let x = "hello" # inline comment
```

---

## 3. Data Types and Literals

### 3.1 Strings

Strings in sh2 are **strict literals**. They do not support implicit variable interpolation or globbing. Explicit syntax is required for dynamic content.

| Syntax | Example | Behavior |
| :--- | :--- | :--- |
| **Strict Literal** | `"hello $name"` | No expansion. Prints literal `$name`. Includes `${foo}`. |
| **Escaped Literal** | `"line\nbreak"` | Supports C-style escapes (`\n`, `\t`, `\\`, `\"`). |
| **Concatenation** | `"hello " & name` | Combines literal and variable value. |
| **Explicit Interp** | `$"hello {name}"` | Expands `{name}`. `$` remains literal text. |
| **Raw Shell** | `sh("echo $FOO")` | `sh` command executes string in shell (expands `$FOO`). |

#### Strict Literals
Standard double-quoted strings (`"..."`) treat `$` and `${...}` as normal characters.
```sh2
let name = "world"
print("Hello $name")  // Output: Hello $name
```

#### Explicit Interpolation
Use the `$` prefix (`$"..."`) to interpolate variables and **expressions** using `{...}` syntax.
Note that the `$` character itself inside `$"..."` is still a literal. To include a literal `{` or `}` inside the string, escape it as `\{` or `\}`.

```sh2
let user = "admin"
print($"User: {user}")     // Output: User: admin
print($"Cost: ${price}")   // Output: Cost: $100 (if price is 100)

// Expressions are supported:
print($"Sum: {1 + 2}")     // Output: Sum: 3
print($"Cwd: {pwd()}")     // Output: Cwd: /current/path
print($"Field: {obj.key}") // Output: Field: value

// Literal braces:
print($"Set: \{a, b\}")    // Output: Set: {a, b}
```

> **Known Limitation**: String literals (quoted text) inside interpolation holes are not supported due to lexer tokenization constraints. To work around this, build strings outside the interpolation and use variables:
> 
> ```sh2
> // NOT SUPPORTED: print($"Result: { "value" }")
> 
> // WORKAROUND:
> let val = "value"
> print($"Result: {val}")
> ```
> 
> This limitation will be addressed in a future release with lexer redesign.

#### Concatenation
Use the `&` operator to join strings and variables.
```sh2
print("Hello " & name & "!")
```

Multilines and raw strings (`r"..."`) are also supported. Raw strings treat backslashes as literals.

```sh2
let cooked = """
line 1
line 2 with \t tab
"""

let raw = r"""
this is raw
\n stays two chars
"""
```

### 3.2 Numbers

Integer literals (e.g. `0`, `42`). Arithmetic operators: `+ - * / %`.

### 3.3 Booleans

`true` and `false`.

Boolean expressions can be stored in variables and used in conditions:

```sh2
let ok = (sum == 42)
if ok {
  print("yes")
}
```

Stored booleans are represented as `"true"` or `"false"` internally.

Booleans can be used in string contexts (concatenation, `print`) and will automatically convert to their string representation:

```sh2
let ok = true
print("Status: " & ok)  # Output: Status: true
```

- **Restriction**: The path MUST be a string literal at compile time (Model 2 restriction). Computed paths (e.g. variables, concatenation) are rejected with a compile error.
   - To use a computed path, explicitly use the canonical safe pattern with `run("sh", "-c", ...)`.
   
   ```sh2
   # ✅ Supported
   with cwd("/tmp/build") { run("make") }
   
   # ❌ Rejected (compile time)
   let d = "/tmp"
   # with cwd(d) { ... } -> Error: cwd(...) requires a string literal path.
   
   # Workaround for computed paths (safe):
   # Pattern: run("sh", "-c", script, arg0_name, arg1_path)
   # We pass "sh2" as the script name ($0), and the path as the first argument ($1).
   # Note: use "\$1" to prevent sh2 from interpolating $1 as a variable.
   run("sh", "-c", "cd \"\$1\" && ls", "sh2", d)
   ```


### For Loops

Iterate over a list of items:

```sh2
for x in (1, 2, 3) {
    print(x)
}
```

> **Note**: The loop variable `x` is **implicitly declared** and function-scoped. It persists after the loop.
> - **Policy A**: The variable must not be already declared on the current execution path. Disjoint declarations are allowed.
> - **Zero-iteration**: If the list is empty (or range is invalid), the loop body does not run. The variable is initialized to `""` (or preserves its existing value if previously set in a disjoint/partial path).

Or a range of numbers (inclusive):

```sh2
for x in 1..10 {
    print(x)
}

for i in 1..argc() {
    print(arg(i))
}

# Parentheses are also supported:
for x in (1..10) {
    print(x)
}

# Spacing around operator is allowed:
for x in 1 .. 10 {
    print(x)
}
```

> **Note:** Range loops use the external `seq` command at runtime. Ensure `seq` is available in your environment (part of coreutils). The range is inclusive: `1..3` produces `1 2 3`.

### While Loops (Bash-only)

### 3.4 Lists (Bash-only)

```sh2
let xs = ["a", "b", "c"]
print(xs[0])
```

### 3.5 Maps (Bash-only)

```sh2
let m = { "k": "v" }
print(m["k"])
```

---

## 4. Variables and Assignment

### 4.1 Declaration: `let`

Variables must be declared with `let` before use.

```sh2
let msg = "hello"
```

**Scope**: Variables are function-scoped. Variables declared inside blocks (e.g., `if`, `while`) are visible in the rest of the function, provided they are guaranteed to be initialized on all control paths.
    - **Policy A (Strict Declaration)**: A variable can only be accessed if it is **definitely assigned** on all paths led to the usage point.
    - **Redeclaration**: Redeclaring a variable (via `let` or loop binder) is an error if it is already declared on the *same* execution path. However, disjoint declarations are allowed.

**Examples**:

1. **Disjoint branches (Constructive Initialization)**:
   ```sh2
   if status() == 0 {
       let x = 1
   } else {
       let x = 2
   }
   print(x) // OK: x is declared in both branches, so it is definitely assigned.
   ```

2. **Partial branches (Fresh Declaration)**:
   ```sh2
   if check() {
       let y = 1
       print(y)
   } else {
       # y is not declared here
   }
   # y is not accessible here (not definitely assigned).
   
   let y = 100 // OK: This is a fresh declaration of 'y'.
   print(y)    // 100
   ```

### `find_files()`

Recursively find files in a directory, returning a list of paths.

**Signature**: `find_files(dir=".", name="*") -> List[String]`

- **Parameters**:
  - `dir` (optional): The root directory to search. Defaults to current directory (`.`).
  - `name` (optional): A glob pattern for filenames. Defaults to all files (`*`).
- **Features**:
  - **NUL-Safe**: Handles filenames with spaces, newlines, and other special characters safely using `find ... -print0`.
  - **Sorted**: Returns paths sorted lexicographically for deterministic behavior.
  - **Recursive**: searches subdirectories.
- **Dependencies**:
  - Requires **Bash 4.3+** target. Not supported on POSIX sh.
  - Requires **GNU find** and **GNU sort** for `-print0` and `-z` flags.

**Example**:
```sh2
# Find all Rust files in src/
for f in find_files(dir="src", name="*.rs") {
    print("Found source file: {f}")
}

# Find all files in current dir (recursive)
let all_files = find_files()
```

### `glob()`

Expand a glob pattern in the current directory (non-recursive).

### 4.2 Reassignment: `set`

To update an existing variable, use `set`. The variable must already be declared.

```sh2
let n = 0
set n = n + 1
```

### 4.3 `try_run` Binding

The result of `try_run(...)` must be immediately bound to a variable via `let`. It cannot be used directly in complex expressions.

```sh2
let result = try_run("ls")
if result.status == 0 { ... }
```

### 4.3 Environment access

- Dot access: `env.HOME`
- Dynamic access: `env("HOME")`

```sh2
let base = env.HOME & "/sh2c/docker-rootless"
```

Environment mutation:

```sh2
set env.DEBUG = "1"
export("DEBUG")
unset("DEBUG")
```

---

## 5. Expressions and Operators

### 5.1 Operator precedence (lowest → highest)

1. `|` (pipeline)
2. `||`
3. `&&`
4. comparisons: `== != < <= > >=`
5. `&` (string concatenation)
6. `+ -`
7. `* / %`
8. unary: `!` and unary `-`
9. postfix: calls `f(...)`, indexing `x[i]`, member access `x.field`

### 5.2 Logical operators: `&&` / `||`

Use `&&` for logical AND and `||` for logical OR:

```sh2
if exists("a") && exists("b") {
  print("both")
}

if exists("a") || exists("b") {
  print("at least one")
}
```

### 5.3 Pipelines

Pipelines connect **stages** with `|`.

- They are broader than just `run(...) | run(...)`.
- Stages can be `run(...)`, `sudo(...)`, or statement blocks `{ ... }`.
- Implementations include pipeline stages that may be blocks / statements in pipe contexts.

> **Important**: When using flags with `sudo(...)`, always use named options (e.g. `n=true` for `-n`) rather than positional strings. sh2 inserts `--` after options, so `sudo("-n", "cat")` would result in `sudo -- -n cat` (treating `-n` as the command), whereas `sudo("cat", n=true)` correctly yields `sudo -n -- cat`.

```sh2
# Block as producer
pipe {
  print("line 1")
  print("line 2")
} | run("grep", "2")

# Block as consumer
run("ls") | {
  let output = capture(input(""))
  print("Captured: " & output)
}

# Mixed run/block
pipe run("echo", "data") | { run("cat") }
```

> **Note**: `print(...)` is a statement, not a pipeline stage. You usually want `run("echo", ...)` or `run("printf", ...)` if you need to feed data into a pipe.


## 6. Command Execution

### 6.0 `each_line` pipeline consumer (Bash-only)

> **Constraint**: `each_line` is only supported when targeting Bash. Compilation for POSIX sh will fail with the error: "each_line is only supported in Bash".
>
> **Constraint**: `each_line` must be the **last segment** of a pipeline.

> **Note**: The loop variable (e.g. `file`) is **function-scoped** and persists after the loop.
>
> **Declaration Rule**: The loop variable is implicitly declared. It must NOT be already declared on the current execution path (Policy A). However, if it was declared in a disjoint branch (e.g. inside an `if` block that has ended), redeclaration via `each_line` is allowed.
> 
> **Zero-iteration behavior**: If the pipeline produces no output (loop never runs):
> - If the loop variable was **unset** before the loop, it is initialized to an empty string `""`.
> - If the variable **already held a value** (e.g. from a partial branch or previous declaration in a disjoint path that is still visible at runtime), that value is **preserved**.

Use `each_line` to iterate over the output of a pipeline line-by-line. This is safer and more robust than `| while read` in Bash because:
1. It runs in the main shell process (via process substitution), so variables modified inside the loop persist.
2. It correctly propagates the exit status of the upstream pipeline command. Upon completion, `status()` reflects the exit code of the upstream pipeline (preserving non-zero codes even if `allow_fail` was used).

```sh2
let count = 0
run("ls", "-1") | each_line file {
  print("File: " & file)
  count = count + 1
}
print($"Total files: {count}")

# Upstream failure is propagated:
run("false") | each_line l { ... }
# status() is non-zero here
```

### 6.1 `stdin_lines()` (standard input iteration)

Iterate over lines from standard input (stdin). This is the sh2 equivalent of `while read -r line; do ... done`.

> **Restriction**: `stdin_lines()` is only valid as the iterable of a `for` loop (e.g. `for lines in stdin_lines()`). It cannot be used in expressions.
> **Arguments**: It takes no arguments.

**Features**:
- Safely handles whitespace and raw lines (uses `read -r`).
- Preserves empty lines.
- Handles lines without trailing newlines correctly.
- **Policy A**: The loop variable is initialized to `""` if the input is empty (0 iterations), or preserves its value if already set.

**Examples**:

1. **Simple filter** (grep-like):
   ```sh2
   for line in stdin_lines() {
       if line == "target" {
           print("Found it!")
       }
   }
   ```

2. **Parsing input**:
   ```sh2
   let count = 0
   for line in stdin_lines() {
       let parts = split(line, ",")
       if len(parts) > 0 {
           print("Column 1: " & parts[0])
           set count = count + 1
       }
   }
   print($"Processed {count} lines")
   ```

3. **Redirected input**:
   ```sh2
   with redirect { stdin: file("input.txt") } {
       for line in stdin_lines() {
           print(line)
       }
   }
   ```

### 6.1.1 `find0()` (streaming file discovery, Bash-only)

Iterate over files discovered by `find`, streaming results via NUL-delimited read. Safe for filenames with spaces and special characters. See [§10.9](#109-builtin-filesystem-helpers) for full documentation.

```sh2
for f in find0(dir="src", name="*.rs", type="f") {
    print(f)
}
```

### 6.2 `run(...)` (expression)

`run(...)` executes an external command with safely separated arguments. It is an **expression**, so it can be used:

- as a standalone statement (expression statement), and
- inside boolean logic (`&&`/`||`) and conditions.

```sh2
run("echo", "hello")

run("true") && run("echo", "only if true succeeded")
run("false") || run("echo", "only if false failed")
```

By default, failures abort the script (set -e-like behavior), unless you enable `allow_fail=true`.

```sh2
run("grep", "x", "missing.txt", allow_fail=true)
print("exit code was " & status())
```

### 6.2 `exec(...)` (statement)

Replaces the current process. Execution does not continue after `exec`.

```sh2
exec("bash")
```

<!-- sh2-docs:allow-sh-examples:start -->
### 6.3 `sh(expr)` (raw shell execution)

> [!WARNING]
> **Unsafe escape hatch**: `sh(expr)` interprets `expr` as raw shell code and is **injection-prone** if you build `expr` by concatenating or interpolating untrusted input. This is intentional—it provides an escape hatch for advanced use cases, not a safe API.

Executes a shell snippet by passing it to the target shell in a child process.

**Execution model:**
- **Isolated Child Process**: Runs in a fresh shell process (e.g., `bash -c "$cmd"`).
- **No Argument Inheritance**: The child shell does **not** inherit positional parameters (`$1`, `$@`, `$*`) from the parent script because arguments are not forwarded (the compiler does not pass `-- "$@"`).
- **Inherits environment**: Exported environment variables are inherited.
- **No persistence**: Local state changes (`cd`, `local var`) do not affect the parent script.

**Options:**
- `shell="bash"`: Specify the shell to use (e.g. `sh("...", shell="bash")`). Default is `sh` (or `bash` if target is bash).
- `args=args()`: Explicitly forward the parent script's positional parameters to the child shell.
- `allow_fail=true`: Suppress "Error in ..." messages if the command fails (but status is still captured in `status()`).

**Gotcha: Positionals are empty by default**
Because `sh(...)` starts a fresh shell, `$@` is empty inside it unless you use `args=args()`.

```sh2
# If script is run as: ./myscript.sh arg1 arg2
print(argc())          # Output: 2

# Default: sh(...) sees nothing
sh(r""" echo "Inside: $@" """)  # Output: Inside:

# With forwarding:
sh(r""" echo "Forwarded: $@" """, args=args())  # Output: Forwarded: arg1 arg2
```

**Probe semantics (non-fail-fast):**
- Updates `status()` with the command's exit code
- **Never** triggers fail-fast behavior or exits the script on non-zero status
- Returns control unconditionally

**Accepts any string expression:**
```sh2
sh("echo hello")
let cmd = "echo dynamic"
sh(cmd)
sh("echo " & cmd)
```

**Passing arguments safely:**
Since `$@` isn't forwarded, you must pass values explicitly.

**Option A: Use sh2 arguments (Recommended)**
Use `arg(n)` or `argv()` in your sh2 code instead of trying to access `$1` inside `sh(...)`.

```sh2
# Safe and clear
print("Processing " & arg(1))
```

**Option B: Avoid concatenation**
Do not concatenate untrusted input into `sh(...)`. It is difficult to quote correctly and easy to introduce injection vulnerabilities.

Instead, usage `run(...)` which passes arguments safely:

```sh2
let file = arg(1)

# Unsafe (injection risk):
# sh("ls -l " & file)

# Safe:
run("ls", "-l", file)
```

**Probe pattern** (explicit status check):
```sh2
sh("false")
if status() != 0 {
  print("Command failed as expected")
}
print("Script continues")  # Always executes
```

**Non-persistence example:**
```sh2
sh("cd /tmp")
# pwd() still returns original directory
# cd inside sh() does not affect parent script
```
<!-- sh2-docs:allow-sh-examples:end -->

#### Prefer structured primitives

For most use cases, prefer these safer options over `sh("...")`:

- **`run(...)`**: Argument-safe command execution with proper quoting
- **Native pipelines**: `run(...) | run(...)` — structured pipeline composition
- **`glob(pattern)`**: Non-recursive glob expansion in cwd (Bash-only). Replaces `sh("ls *.txt")`.
- **`find0(dir=, name=, type=, maxdepth=)`**: NUL-safe, quoting-safe streaming file discovery (Bash-only). Replaces `sh("find ... -print0 | ...")`.
- **`find_files(dir=, name=)`**: In-memory file discovery with `mapfile` (Bash-only).
- **`stdin_lines()`**: Iterate lines from stdin (portable). Replaces `sh("... | while read line")`.
- **`| each_line var { ... }`**: Pipeline consumer for line-by-line processing (Bash-only).
- **`spawn(run(...))` / `wait(pid)`**: Background job control. Replaces `sh("cmd &")`.
- **String helpers**: `lines(...)`, `split(...)`, `trim()`, `replace()` for text processing.

Use `sh()` only when you need raw shell syntax that cannot be expressed through safe APIs (e.g., process substitution `<(...)`, complex multi-tool pipelines, brace expansion).

### 6.4 `capture(...)` (capture stdout)

`capture(...)` captures stdout from a structured command/pipeline expression.

Typical examples:

```sh2
let who = capture(run("whoami"))
let n = capture(run("printf", "a\n") | run("wc", "-l"))

# With allowed failure (returns captured stdout even if command fails)
# The command's exit code is preserved in status() after capture returns.
let output = capture(run("ls", "missing"), allow_fail=true)
if status() != 0 {
    print("ls failed with status " & status())
}
```

> **Note**: `capture(run(..., allow_fail=true))` is also supported. The `allow_fail` option is "hoisted" from the inner `run` call to the capture behavior.

> **Restriction**: `capture(..., allow_fail=true)` is only valid in `let` assignments (e.g. `let x = capture(...)`) to ensure the exit status is correctly preserved and observable via `status()`.

### 6.5 `try_run(...)` → `RunResult`

Runs a command without aborting and returns a result object:

- `.status`
- `.stdout`
- `.stderr`

```sh2
let r = try_run("git", "rev-parse", "HEAD")
if r.status == 0 {
  print(r.stdout)
} else {
  print_err(r.stderr)
}
```

**Target note:** On `--target posix`, implementations may restrict or omit `.stdout` / `.stderr` capture (documented as target-dependent). `.status` is always available.

### 6.6 `sudo(...)` (privileged execution)

Structured wrapper for `sudo` command execution with type-safe options:

```sh2
# Basic usage
sudo("systemctl", "restart", "nginx")

# With user option
sudo("ls", "/root", user="admin")

# With environment preservation
sudo("env", env_keep=["PATH", "HOME"])
```

**Supported options:**
- `user` (string literal) — run as specified user (generates `-u`)
- `n` (boolean) — non-interactive mode (generates `-n`)
- `k` (boolean) — invalidate cached credentials (generates `-k`)
- `prompt` (string literal) — custom password prompt (generates `-p`)
- `E` (boolean) — preserve environment (generates `-E`)
- `env_keep` (list of string literals) — preserve specific variables (generates `--preserve-env=...`)
- `allow_fail` (boolean, statement-form only) — non-aborting execution

**Argument ordering:**
Mixed positional and named arguments are allowed:
```sh2
sudo(user="root", "ls")        # ✅
sudo("ls", user="root")        # ✅
sudo(n=true, "ls", user="root") # ✅
```

**Compile-time validation:**
- Option values must be literals:
  - `user`, `prompt`: string literals
  - `n`, `k`, `E`: boolean literals
  - `allow_fail`: boolean literal (statement-form only)
  - `env_keep`: list of string literals
- Duplicate options are rejected
- Unknown options are rejected
- `allow_fail` in expression context is rejected with specific diagnostic

**Lowering behavior:**
- Generates stable flag ordering: `sudo -u ... -n -k -p ... -E --preserve-env=... -- cmd args...`
- Mandatory `--` separator before command arguments
- Statement-form with `allow_fail=true` behaves like `run(..., allow_fail=true)`

**Expression-form restriction:**
```sh2
# ❌ Not allowed:
let x = capture(sudo("ls", allow_fail=true))

# ✅ Use capture's allow_fail instead:
let x = capture(sudo("ls"), allow_fail=true)
```

Error message: `"allow_fail is only valid on statement-form sudo(...); use capture(sudo(...), allow_fail=true) to allow failure during capture"`



## 7. Status, Errors, and `try/catch`

### 7.1 `status()`

`status()` returns the exit code of the most recent operation and is updated by:

- `run(...)` (including `allow_fail=true` inside `capture`)
- `try_run(...)`
- `sh("...")`
- filesystem predicates like `exists(...)`, `is_file(...)`, etc.

### 7.2 `try { ... } catch { ... }`

If a command fails inside `try`, control transfers to `catch`. Inside `catch`, `status()` contains the failing status code.

```sh2
try {
  run("false")
  print("won't run")
} catch {
  print_err("failed: " & status())
}
```

---

## 8. Control Flow

### 8.1 `if / elif / else`

```sh2
if status() == 0 {
  print("ok")
} elif status() == 2 {
  print("special")
} else {
  print("bad")
}
```

### 8.2 `while`

```sh2
let i = 0
while i < 5 {
  print(i)
  set i = i + 1
}
```

### 8.3 `for`

List iteration (Bash-only when lists are used):

```sh2
let xs = ["a", "b", "c"]
for x in xs {
  print(x)
}
```

Map iteration (Bash-only when maps are used):

```sh2
let m = { "k": "v" }
for (k, v) in m {
  print(k & "=" & v)
}
```

### 8.4 `break` / `continue`

```sh2
let i = 0
while true {
  set i = i + 1
  if i == 3 { continue }
  if i > 5 { break }
  print(i)
}
```

### 8.5 `case`

Case arms use `=>`. Patterns include:

- string literal patterns
- `glob("pattern")`
- `_` wildcard default

```sh2
let filename = "report.txt"
case filename {
  glob("*.txt") => { print("text") }
  "README.md" => { print("readme") }
  _ => { print("other") }
}
```

---

## 9. Scoped Blocks (`with`)

### 9.1 `with env { ... } { ... }`

Verified syntax includes **colon bindings**:

```sh2
with env { DEBUG: "1", HOME: env.HOME } {
  run("env")
}
```

### 9.2 `with cwd(expr) { ... }`

```sh2
with cwd("/tmp") {
  run("pwd")
}
```

> **Note**: `cwd(...)` requires a string literal argument (e.g., `"/path"`). Computed paths are not supported. If you need a dynamic working directory, use `cd(expr)` (scoped via `subshell { ... }` if needed) or `sh("cd ...")`.

### 9.3 `with redirect { ... } { ... }`

Configure file descriptors for the scoped block. Supports single targets and **multi-sink lists** (fan-out).

**Single Targets**:

```sh2
# stdout to file (overwrite)
with redirect { stdout: file("out.log") } { ... }

# append mode
with redirect { stdout: file("out.log", append=true) } { ... }

# stderr to stdout (merge)
with redirect { stderr: to_stdout() } { ... }
```

**Multi-Sink Lists (Fan-out)**:

You can provide a list of targets to duplicate output (similar to `tee`).

```sh2
# Write to file AND keep visible on terminal
with redirect { stdout: [file("out.log"), inherit_stdout()] } { ... }

# Write to multiple files (silent on terminal)
with redirect { stdout: [file("a.log"), file("b.log")] } { ... }
```

- `inherit_stdout()` / `inherit_stderr()`: Keeps the output visible on the parent stream. If omitted from a list, the output is not shown on the terminal.
- **Legacy Keywords**: `stdout` and `stderr` can be used as synonyms for `to_stdout()` / `to_stderr()` in single-target contexts, but function-style `to_stdout()` is preferred.

**Restrictions**:

1. **Mixed Append**: A list cannot mix append modes. `[file("a", append=true), file("b")]` is invalid. All files in a multi-sink list must share the same append setting.
2. **POSIX Limitation**: Multi-sink redirects (lists with >1 target or usage of `inherit_*` with a file) are **not supported** when compiling with `--target posix`.
   - Error: *"multi-sink redirect is not supported for POSIX target; use a single redirect target or switch to --target bash"*
   - Exception: A single-element list like `[file("out.log")]` is allowed on POSIX.

### 9.4 `with log(path, append=true|false) { ... }` (Bash-only)

```sh2
with log("activity.log", append=true) {
  run("echo", "hello")
}
```

On `--target posix`, `with log` is not available.

---

## 10. Built-in Functions (selected)

### 10.1 I/O statements

- `print(expr)`
- `print_err(expr)`

> `print`/`print_err` are statements, not pipeline stages.

### 10.2 Filesystem predicates

- `exists(path)`
- `is_dir(path)`
- `is_file(path)`
- `is_symlink(path)`
- `is_exec(path)`
- `is_readable(path)`
- `is_writable(path)`
- `is_non_empty(path)`

### 10.3 Helpers (as implemented)

- string/list: `split`, `join`, `lines`, `trim`, `replace`
- regex: `matches(text, regex)`
- envfiles: `load_envfile`, `save_envfile`
- JSON: `json_kv(...)`
- process/system: `pid()`, `ppid()`, `uid()`, `pwd()`, etc.

### 10.4 Argument Access

Scripts and snippets can access command-line arguments using:

- `argv()` or `args()`: Returns all arguments as a list.
- `arg(n)`: Returns the *n*-th positional argument (1-based).
  - If `n` is a literal number, it compiles to efficiency shell syntax like `$1`.
  - If `n` is an expression (e.g. `arg(i + 1)`), it compiles to a dynamic lookup.
  - The index expression **must** be an integer type (number, variable, or arithmetic). String literals or complex expressions like function calls are not allowed.
- `argc()`: Returns the total number of arguments.
- `argv0()`: Returns the script name / entry point.

`arg(n)` supports dynamic expressions:
```sh2
let i = 1
print(arg(i))
```

If `n` is out of bounds, `arg(n)` returns an empty string (it does not crash).

### 10.5 Interactive Helpers

#### `confirm(prompt, default=...)` → boolean

Interactive yes/no confirmation prompt:

```sh2
if confirm("Proceed with deployment?") {
    run("deploy.sh")
}

# With default value
if confirm("Delete files?", default=false) {
    run("rm", "-rf", "data/")
}
```

**Behavior:**
- Returns `true` for yes, `false` for no
- Accepts `y`, `yes`, `Y`, `YES` as affirmative (case-insensitive)
- Accepts `n`, `no`, `N`, `NO` as negative (case-insensitive)
- Optional `default=true` or `default=false` parameter

**Non-interactive mode:**
- If `default` is provided, uses that value when stdin is not a terminal
- If `default` is not provided, fails with error in non-interactive mode

**Environment overrides:**
- `SH2_YES=1` — always return `true`
- `SH2_NO=1` — always return `false`

**Example with default:**
```sh2
# Safe for CI/automation
if confirm("Apply changes?", default=false) {
    run("apply.sh")
}
```

#### `input(prompt)` → string

Read user input from stdin:

```sh2
let name = input("Enter your name: ")
print("Hello, " & name)
```

### 10.6 String and List Utilities

#### `starts_with(text, prefix)`

Boolean predicate that evaluates to `true` if `text` starts with `prefix`.

```sh2
if starts_with("foobar", "foo") { ... }
```

#### `contains_line(file, needle)`

Boolean predicate that evaluates to `true` if the file at `file` contains a line exactly equal to `needle`.

- **Exact-line match**: Uses `grep -Fqx -e` for literal, full-line comparison (no regex/glob/substring).
- **File contents**: Reads and searches the file at the path specified by `file` (not the string value itself).
- **Portable**: Works on both Bash and POSIX targets. Uses `-e` flag for POSIX compatibility and safe handling of needles starting with `-`.
- **Use case**: Ideal for checking registry trust, configuration files, or any line-oriented data.

```sh2
# Check if a registry is trusted
if contains_line("/etc/docker/daemon.json", "registry.example.com") {
    print("Registry already trusted")
} else {
    append_file("/etc/docker/daemon.json", "registry.example.com")
}

# Check command output
let tmpfile = "/tmp/ls_output.txt"
run("ls", stdout=tmpfile)
if contains_line(tmpfile, "Makefile") { ... }
```

#### `contains(haystack, needle)`

Type-directed inclusion check. Behavior depends on the static type of `haystack`:

| Haystack Type | Behavior | Target Support |
|---|---|---|
| **List** | Checks if `needle` is an element of the list. | **Bash Only** |
| **String** | Checks if `needle` is a substring of `haystack`. | Portable |

**List detection rules**:
- List literals: `["a", "b"]`
- List expressions: `split(...)`, `lines(...)`
- **Tracked Variables**: Variables assigned a list value (`let x = [...]`) are tracked as lists. All other variables (e.g. `let x = "s"`) are treated as strings.

```sh2
# String Substring:
if contains("host:5000", ":") { ... }

# List Membership (Bash-only):
let items = ["a", "b"]
if contains(items, "b") { ... }

# List Expression (Bash-only):
if contains(lines(text), "bar") { ... }
```

**Detailed Semantics Table**:

| Haystack Form              | Compile-time Class | Lowering Result / IR                | Runtime Mechanism            | Target |
|----------------------------|-------------------|-------------------------------------|------------------------------|--------|
| String literal             | String            | Val::ContainsSubstring              | printf '%s' … \| grep -Fq -e | Bash+Posix |
| Scalar var (untracked)     | String            | Val::ContainsSubstring              | printf '%s' … \| grep -Fq -e | Bash+Posix |
| Tracked scalar var         | String            | Val::ContainsSubstring              | printf '%s' … \| grep -Fq -e | Bash+Posix |
| List literal               | List              | materialize tmp → Val::ContainsList | __sh2_contains               | Bash-only |
| List var (tracked)         | List              | Val::ContainsList                    | __sh2_contains               | Bash-only |
| List expr (split/lines)    | List              | materialize tmp → Val::ContainsList | __sh2_contains               | Bash-only |
| Unknown var (not tracked)  | String (default)  | Val::ContainsSubstring              | printf '%s' … \| grep -Fq -e | Bash+Posix |

**String Substring Details**:
- Fixed-string search (no regex): `-F` flag
- Quiet mode (exit code only): `-q` flag  
- POSIX-compliant pattern specification: `-e` flag (handles needles starting with `-`)
- Portable: works on both Bash and POSIX sh targets
- Special characters safe: `$`, `[`, `]`, `*`, `\`, etc. are treated literally

#### `glob(pattern)` → list (Bash-only)

Returns a list of filesystem paths matching the glob pattern. Must be used in `let` assignment or `for` loop context.

```sh2
# Basic usage
let files = glob("*.conf")
for f in files {
  print(f)
}

# Direct iteration
for f in glob("*.log") {
  run("rm", f)
}

# Check for matches
let xs = glob("*.nope")
if count(xs) == 0 {
  print("no matches")
}
```

**Behavior**:
- Returns paths sorted lexicographically for determinism (uses `LC_ALL=C` sort stability)
- Empty matches or empty pattern `""` return an empty list (no error, unlike raw shell)
- Pattern is treated as a filesystem glob, not shell-evaluated
- Uses `compgen -G` internally (safe, no `eval`)

**Target support**: 
- Bash: ✓ (requires Bash 4.3+ for `local -n` nameref)
- POSIX: ✗ (compile-time error: "glob() requires bash target")

**Filename limitations**: Not NUL-safe. Paths containing newlines may behave unexpectedly (consistent with `lines()` and shell conventions).



### 10.7 File I/O

#### `read_file(path)` → string

Reads the contents of a file and returns it as a string. Must be used in an expression context (cannot be used as a statement).

```sh2
let content = read_file("config.txt")
```

- **Error behavior**: If the file does not exist or cannot be read, the script exits with a non-zero status (fail-fast).
- **Newlines**: Content is returned exactly as stored, including trailing newlines.
- **Portable**: Works on both Bash and POSIX targets.

#### `write_file(path, content)`

Creates or truncates `path` and writes `content` exactly as provided. This is a **statement**, not an expression.

```sh2
write_file("output.txt", "hello")
write_file("data.txt", content & "\n")  # explicit newline
```

- **No implicit newline**: Content is written exactly; add `\n` explicitly if needed.
- **Error behavior**: If the file cannot be written (e.g., path is a directory), the script exits with a non-zero status.
- **Portable**: Works on both Bash and POSIX targets.

#### `append_file(path, content)`

Appends `content` to `path`, creating the file if it does not exist. This is a **statement**, not an expression.

```sh2
append_file("log.txt", "entry\n")
```

- **No implicit newline**: Content is appended exactly; add `\n` explicitly if needed.
- **Error behavior**: Same as `write_file`.
- **Portable**: Works on both Bash and POSIX targets.

### 10.8 Path Lookup

#### `which(name)` → string

Searches the system's `$PATH` for an executable and returns its path (or an empty string if not found).

**Return value:**
- Returns the first matching executable path from `$PATH` if found
- Returns an empty string `""` if not found
- The returned path may be relative if `$PATH` contains relative entries

**Exit status:**
- Returns exit code `0` when the command is found
- Returns exit code `1` when the command is not found
- This allows branching on the result: `if which("git") { ... }`

**Usage patterns:**

```sh2
# Pattern 1: Branch on exit status (recommended)
if which("docker") {
    print("Docker is available")
}

# Pattern 2: Check the returned path
let docker_path = which("docker")
if docker_path != "" {
    print("Docker found at: " & docker_path)
}

# Pattern 3: Use the path directly
let sh_path = which("sh")
run(sh_path, "-c", "echo hello")
```

**Implementation details:**
- If `name` contains a slash (e.g. `"/bin/sh"` or `"./script"`), it checks that path directly
- Otherwise, it searches directories in `$PATH`, preserving empty segments (which mean `.`)
- Returns the first match that is an executable file (or symlink to one)
- **Non-aborting**: `which()` returning 1 (not found) does not abort the script—it is a query builtin
- Portable: Works on both Bash and POSIX targets without external `which` dependency



### 10.9 Builtin Filesystem Helpers

#### `find_files(dir=".", name="*")` → list (Bash-only)

Recursively find files in a directory, returning a list of paths.

- **Parameters**:
  - `dir` (optional): The root directory to search. Defaults to current directory (`.`).
  - `name` (optional): A glob pattern for filenames. Defaults to all files (`*`).
- **Features**:
  - **NUL-Safe**: Handles filenames with spaces, newlines, and other special characters safely using `find ... -print0`.
  - **Sorted**: Returns paths sorted lexicographically for deterministic behavior.
  - **Recursive**: searches subdirectories.
- **Dependencies**:
  - Requires **Bash 4.3+** target. Not supported on POSIX sh.
  - Requires **GNU find** and **GNU sort** for `-print0` and `-z` flags.

**Example**:
```sh2
# Find all Rust files in src/
for f in find_files(dir="src", name="*.rs") {
    print("Found source file: {f}")
}

# Find all files in current dir (recursive)
let all_files = find_files()
```

#### `find0(dir=".", name=?, type=?, maxdepth=?)` — streaming file discovery (Bash-only)

Iterates over files found by `find` using NUL-delimited streaming. Unlike `find_files()`, which returns a Bash array, `find0()` is used as a `for`-loop iterable and streams results one-by-one — suitable for large directory trees.

- **Parameters** (all named, all optional):
  - `dir`: Root directory to search. Defaults to `"."`. The root directory itself is excluded from results.
  - `name`: Glob pattern for filenames (maps to `find -name`).
  - `type`: File type filter — must be literal `"f"` (files) or `"d"` (directories). Compile-time validated.
  - `maxdepth`: Maximum search depth — must be a non-negative integer literal. Compile-time validated.
- **Features**:
  - **Quoting-Safe**: Arguments are passed as separate argv elements with `--` separator; no shell splitting or globbing.
  - **Path Format**: Returned paths include the `dir` prefix (e.g. `find0(dir="src")` yields `src/foo.rs`).
  - **NUL-Safe**: Uses `find -print0 | sort -z | while read -d ''` to handle filenames with spaces, newlines, and special characters.
  - **Deterministic**: Results are sorted lexicographically via `LC_ALL=C sort -z`.
  - **Error suppression**: `find` permission errors are suppressed (`2>/dev/null`).
  - **Zero-iteration safe**: If no files match, the loop body simply doesn't execute.
- **Restrictions**:
  - `find0()` is only valid as a `for`-loop iterable. It cannot be used in expressions.
  - Positional arguments are not accepted.
  - Requires **Bash** target. Compilation for POSIX sh fails with a compile error.
  - Requires `find` and `sort` with NUL-delimiter support (GNU coreutils / BSD).

**Examples**:
```sh2
# Find all .rs files in src/
for f in find0(dir="src", name="*.rs", type="f") {
    print(f)
}

# Find directories only, max 2 levels deep
for d in find0(dir=".", type="d", maxdepth=2) {
    print(d)
}

# Minimal: find everything under current dir
for entry in find0() {
    print(entry)
}
```

#### `glob(pattern)` → list (Bash-only)

Expand a glob pattern in the current directory (non-recursive). Uses `compgen -G`.

- **Parameters**:
  - `pattern`: The glob pattern (e.g. `*.txt`).
- **Target**: Bash-only.

```sh2
for f in glob("*.txt") { ... }
```

---

## 11. Job Control

Concurrent execution of commands is supported via `spawn(...)` and `wait(...)`.

### 11.1 Spawning Background Jobs

`spawn(run(...))` starts a command in the background and returns its PID (as a string).

```sh2
let pid = spawn(run("sleep", "10"))
```

- **Restricted Argument**: `spawn` only deals with `run(...)` or `sudo(...)` expressions. It does not accept arbitrary blocks.
- **Return Value**: Returns the PID of the spawned process.

### 11.2 Waiting for Jobs

`wait(pid)` waits for a process to complete and returns its exit code.

```sh2
let rc = wait(pid)
```

- **Return Value**: The exit code (0-255).
- **Status**: The `status()` global is also updated.
- **Fail-fast**: By default, if the job exits with non-zero, the script aborts (like `run`).
- **Allow Failure**: Use `allow_fail=true` to prevent aborting on non-zero exit code.

```sh2
let rc = wait(pid, allow_fail=true)
if rc != 0 {
    print("Job failed with: " & rc)
}
```

### 11.3 Waiting for Multiple Jobs

`wait_all(pids)` waits for all processes in a list and returns the first non-zero exit code (in list order) or 0 if all succeed.

```sh2
let pids = [spawn(run("task1")), spawn(run("task2"))]
let rc = wait_all(pids)
```

- **Return Value**: First non-zero exit code in **list order**, or 0 if all succeed.
- **Status**: The `status()` global is set to the returned value.
- **Fail-fast**: By default, if any job exits non-zero, the script aborts after all jobs are waited.
- **Allow Failure**: Use `allow_fail=true` to suppress the abort.
- **POSIX Restriction**: On `--target posix`, only inline list literals are supported (e.g., `wait_all([p1, p2])`). List variables are not supported.

```sh2
let rc = wait_all(pids, allow_fail=true)
if rc != 0 {
    print($"First failure code: {rc}")
}
```

---

## 12. Targets and Portability

### `--target bash` (default)
Supports the full implemented feature set, including lists/maps, `with log`, interactive helpers (if enabled), and full `try_run` capture.

### `--target posix`
Prioritizes portability. Bash-only features (lists/maps, `with log`, and potentially full `.stdout/.stderr` capture) are restricted.

---
# Docs

The GitHub repo is here:  
**[https://github.com/siu-mak/sh2lang](https://github.com/siu-mak/sh2lang)**

## Further Documentation

- [`docs/language.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/language.md) — full language reference (syntax + semantics)
- [`docs/sh2do.md`](https://github.com/siu-mak/sh2lang/blob/main/docs/sh2do.md) — sh2do CLI documentation
- `tests/` — fixtures and integration tests (acts as an executable spec)
---

### Summary

sh2 provides a structured, test-validated shell language with explicit control flow, safer command execution, predictable error handling, and dual-target compilation to bash or POSIX sh.

---

## Next Steps

To go deeper, check out these articles:

*   **[No Implicit Expansion](articles/features/13-no-implicit-expansion.md)**: Why string behavior is different (and safer).
*   **[Error Handling](articles/features/15-error-handling.md)**: Patterns for robust scripts.
*   **[sh2do CLI](sh2do.md)**: Using the snippet runner.
