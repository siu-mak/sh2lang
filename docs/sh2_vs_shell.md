# sh2 vs Shell: A Comparison Guide

This document provides a detailed comparison between `sh2` and traditional shells (POSIX sh / Bash). It assesses the current capabilities of the `sh2` compiler and demonstrates how common shell patterns are expressed in `sh2`.

## 1. Feature Support Matrix

The following table estimates the coverage of standard shell features by the `sh2` language (v1.0).

| Feature Category | POSIX sh Capability | Bash Capability | sh2 Native Support | sh2 Approach |
| :--- | :--- | :--- | :--- | :--- |
| **Variables** | String only | String, Array, Assoc Array | String, Array (Indexed) | Strongly typed-ish (Let/Set), no implicit splitting. |
| **Variable Mutation**| `v=val` | `v[i]=val`, `v+=val` | `set v = val` | **Gap**: No single-element array assignment (`v[0]=1`). |
| **Control Flow** | `if`, `while`, `for`, `case` | + C-style loops | `if`, `while`, `for`, `case`, `try` | Clean C-like syntax. `try/catch` wraps blocks in `if ! (...)`. |
| **Conditionals** | `[ ... ]` | `[[ ... ]]` | `if expr` | First-class boolean expressions. `==` maps to `[ = ]`. |
| **Functions** | Basic | + `local` | `func name() { ... }` | Named parameters. Bash uses `local`; POSIX leaks vars globally. |
| **I/O** | `>`,`<`,`|` | + `<<<`, `<()`, `&>`, `|&` | `|`, `with redirect` | Structured blocks. **Gap**: No generic `<(process)` substitution. |
| **String Ops** | `${v#p}`, `${v%p}` | `${v/s/r}`, `${v:o:l}` | `len(v)` | **Gap**: `len()` uses `awk`. No native substring/replace yet. |
| **Arithmetic** | `$(( ... ))` | `$(( ... ))` | `$(( ... ))` | Use `expr`. No `++` / `--` operators; use `set i = i + 1`. |
| **Process Control** | `&`, `wait`, `trap` | `disown`, `coproc` | `spawn`, `wait` | **Gap**: No native `trap` yet (use `raw`). |

## 2. The Rosetta Stone: Syntax Comparison

### 2.1 Variables & Assignments

**Bash/POSIX:**
```sh
# Helper to avoid space issues
full_name="John Doe"
count=5

# Arrays (Bash only)
files=("a.txt" "b.txt")
file_1="${files[1]}"
```

**sh2:**
```sh2
# Strings are always quoted
let full_name = "John Doe"
let count = 5

# Arrays are first-class lists
let files = ["a.txt", "b.txt"]
let file_1 = files[1]
```
*Note: sh2 variables effectively behave like they are always double-quoted in Bash.*

### 2.2 Conditionals

**Bash/POSIX:**
```sh
if [ "$name" = "admin" ] && [ -f "/tmp/lock" ]; then
    echo "Access granted"
elif [ -z "$name" ]; then
    echo "No name" >&2
else
    echo "Access denied"
fi
```

**sh2:**
```sh2
if name == "admin" && is_file("/tmp/lock") {
    print("Access granted")
} elif !name {
    print_err("No name")
} else {
    print("Access denied")
}
```
*Improvements: No `[` or `[[` confusion. `!name` checks for empty string.*

### 2.3 Loops

**Bash/POSIX:**
```sh
# Iterate args
for arg in "$@"; do
    echo "Arg: $arg"
done

# C-style (Bash)
for ((i=0; i<10; i++)); do
    echo $i
done
```

**sh2:**
```sh2
# Iterate args
for arg in args {
    print($"Arg: {arg}")
}

# C-style isn't native, use while
let i = 0
while i < 10 {
    print(i)
    set i = i + 1
}
```

### 2.4 Command Execution & I/O

**Bash/POSIX:**
```sh
# Simple command
ls -la /tmp

# Silencing output
rm file.txt > /dev/null 2>&1

# Complex grouping
{
  echo "Start"
  cat file.txt
} > output.log
```

**sh2:**
```sh2
# Simple command (explicit strings)
run("ls", "-la", "/tmp")

# Silencing output
with redirect {
    stdout: file("/dev/null"),
    stderr: stdout
} {
    run("rm", "file.txt")
}

# Complex grouping
with redirect { stdout: file("output.log") } {
    print("Start")
    run("cat", "file.txt")
}
```

### 2.5 Error Handling

**Bash/POSIX:**
```sh
# Stop on error
set -e
cp src dst

# Check specific failure
if ! cp src dst; then
    echo "Copy failed"
fi
```

**sh2:**
```sh2
# sh2 does NOT imply 'set -e' by default (design choice), 
# but checks are explicit.

# Check status
run("cp", "src", "dst")
if status != 0 {
    print("Copy failed")
}
# OR use a try/catch construct (future feature) or conventional checks
```

## 3. Assessment of Gaps (Current Status)

### 3.0 Quantitative Assessment
*   **POSIX Scripts:** ~95% native coverage. High confidence for production use.
*   **Bash Scripts:** ~80% native coverage. Requires `sh()` escape hatches for specific advanced features.

### 3.1 What sh2 replaces 100%
*   **Structural Logic**: Any logic involving standard control flow (`if`/`else`/`loops`) is better in sh2.
*   **Safe Variable Handling**: Code that deals with filenames with spaces is trivial in sh2, painful in Bash.
*   **Argument Parsing**: Iterating `args` is safe and clean.

### 3.2 What requires Escape Hatches (`sh` / `raw`)
Certain "Power Bash" features are not yet native in sh2 v1.0.

#### 1. Associative Arrays (Maps)
**Bash:** `declare -A map; map[key]=val`
**sh2 Workaround:**
```sh2
raw {
    declare -A my_map
    my_map[key]="val"
}
```

#### 2. Process Substitution
**Bash:** `diff <(sort a) <(sort b)`
**sh2 Workaround:**
```sh2
# Must use raw execution for <(...) syntax
sh("diff <(sort a) <(sort b)")
```

#### 3. String Manipulation
**Bash:** `${filename%.*}` (Extension removal)
**sh2 Workaround:**
```sh2
let filename = "image.png"
# No native .trim_ext() yet
let base = sh($"echo \"{filename%.*}\"")
```

#### 4. Array Mutation
**Bash:** `arr[0]="val"` or `arr+=("val")`
**sh2 Workaround:**
`sh2` arrays are currently effectively immutable/reassigned as whole. You cannot modify a single index or push efficiently without `raw`.
```sh2
let list = ["a", "b"]
set list = ["a", "b", "c"] # Reassign
```

#### 5. Advanced Pattern Matching
**Bash:** `[[ $v =~ ^[0-9]+$ ]]`
**sh2 Workaround:**
Use `raw` or external tools like `grep`.
```sh2
if sh("[[ $v =~ ^[0-9]+$ ]]") { ... }
```

## 4. Portability Assessment

| Target | Description | Reliability |
| :--- | :--- | :--- |
| **`--target posix`** | Generates strict `#`/bin/sh` code. Avoids arrays, `local`, `[[`. | **High**. Compiler errors if you use Bash features (like arrays). **Note**: Function parameters are assigned to global variables (scope leakage) as POSIX lacks `local`. |
| **`--target bash`** | Generates `#!/bin/bash`. Uses `local`, arrays, `[[`. | **High**. Standard compile target. |

**Conclusion**: `sh2` is ready to replace the "Skeleton and Muscle" of your scripts (logic, flow, command orchestration). The "Nerves" (complex distinct bashisms) can be handled via escape hatches, allowing for gradual migration.
