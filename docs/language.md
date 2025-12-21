

---

# ✅ `docs/language.md` (clean, render-safe)
# sh2 Language Specification

**Version:** 1.0 (Draft)  
**Status:** Implementer-facing  
**Target:** POSIX shell (default), Bash (optional extensions)

---

## 1. Overview

sh2 is a compiled shell language designed to express shell programs using
**semantic, readable constructs**, compiling them into **portable POSIX shell**.

sh2 exists to address the following problems:

- Traditional shell syntax encodes meaning in punctuation
- Scripts are hard to read, review, and refactor
- Small mistakes often cause silent failures

sh2 replaces symbolic shell syntax with **explicit language constructs**
while preserving compatibility with existing shell environments.

---

## 2. Design Goals

### 2.1 Semantic Explicitness

Language constructs express *intent*, not shell mechanics.

| sh2 | Shell |
|----|------|
| `print_err("msg")` | `echo msg >&2` |
| `if x {}` | `[ -n "$x" ]` |

Shell punctuation is never exposed directly to users.

---

### 2.2 Minimalism

sh2 includes only features required for real shell scripting.
It is **not** a general-purpose programming language.

---

### 2.3 Phase Separation

sh2 is defined in terms of three conceptual layers:

| Layer | Purpose |
|------|---------|
| AST | Syntax as written |
| IR | Semantic meaning |
| Codegen | Shell emission |

Each phase must be independently testable.

---

### 2.4 Portability

Generated output must be valid POSIX shell unless a Bash-only feature is used.

---

## 3. Program Structure

A sh2 program consists of function definitions and optional top-level statements.
```sh2
func main() {
    print("hello")
}
````

### 3.1 Entry Point

* A function named `main` is the program entry point
* The compiler emits:

```sh
main "$@"
```

* Programs without `main` are treated as libraries unless they contain top-level statements

---

## 4. Lexical Elements

### 4.1 Identifiers

Identifiers match:

```
[A-Za-z_][A-Za-z0-9_]*
```

They are case-sensitive and must not conflict with keywords.

---

### 4.2 Strings

Strings are double-quoted.

```sh2
"hello"
"line\nbreak"
```

Strings do **not** perform shell expansion unless explicitly interpolated.

---

### 4.3 Comments

Line comments begin with `#` and continue to end-of-line.

---

## 5. Execution Model

* Statements execute sequentially
* Commands execute synchronously
* sh2 does not implicitly modify environment variables or shell options

---

## 6. Statements

### 6.1 `run`

```sh2
run("command", "arg1", "arg2")
```

Executes an external command with literal arguments.

Generated shell:

```sh
command arg1 arg2
```

---

### 6.2 `print`

```sh2
print("message")
```

Writes to standard output and appends a newline.

---

### 6.3 `print_err`

```sh2
print_err("error")
```

Writes to standard error and appends a newline.

Generated shell:

```sh
echo error >&2
```

---

### 6.4 Variable Assignment

```sh2
let x = "hello"
set env.PATH = "/custom/bin:" + env.PATH
```

Assignments are statements, not expressions.

---

### 6.5 Conditionals

```sh2
if value {
    print("yes")
} else {
    print("no")
}
```

Semantics:

* Empty string → false
* Non-empty string → true

Generated shell:

```sh
if [ -n "$value" ]; then
  ...
else
  ...
fi
```

---

### 6.6 Case Statements

```sh2
case value {
    "a" => { print("A") }
    "b" | "c" => { print("B or C") }
    _ => { print("default") }
}
```

`_` represents the default branch.

---

### 6.7 Loops

#### While

```sh2
while cond {
    run("echo", "looping")
}
```

#### For

```sh2
for x in ["a", "b"] {
    print(x)
}
```

```sh2
for arg in args {
    print(arg)
}
```

---

### 6.8 With Blocks

`with` introduces scoped shell behavior.

#### with env

```sh2
with env { AWS_PROFILE: "prod" } {
    run("aws", "s3", "ls")
}
```

#### with cwd

```sh2
with cwd "/srv/app" {
    run("make")
}
```

#### with redirect

```sh2
with redirect {
    stdout: file("out.log", append=true),
    stderr: stdout
} {
    run("build")
}
```

---

### 6.9 Pipelines

```sh2
run("cat", "file") | run("grep", "foo") | run("wc", "-l")
```

Or structured:

```sh2
pipe {
    run("cat", "file")
} | {
    run("grep", "foo")
}
```

---

### 6.10 Subshells and Groups

```sh2
subshell {
    set env.X = "1"
}
```

```sh2
group {
    run("echo", "no isolation")
}
```

---

### 6.11 Background Execution

```sh2
spawn {
    run("sleep", "5")
}
```

---

### 6.12 Escape Hatches (Bash Completeness)

#### sh()

```sh2
sh("declare -A m; m[x]=1; echo ${m[x]}")
```

#### raw blocks

```sh2
raw {
    trap 'echo bye' EXIT
}
```

At least one escape hatch MUST be supported.

---

## 7. Expressions

Supported expression features:

* string concatenation: `+`
* equality: `==`, `!=`
* boolean logic: `&&`, `||`, `!`
* list literals: `["a", "b"]`

Interpolation:

```sh2
print($"hello {name}")
```

---

## 8. Static Rules

A program is invalid if:

* `main` is missing (for executables)
* `print()` or `print_err()` has incorrect arity
* `else` does not follow `if`
* unsupported features are used for the selected target

---

## 9. Targets

* `posix` (default)
* `bash` (enables Bash-only features)

The compiler must reject unsupported features for the active target.

---

## 10. Conformance

An implementation is conforming if it:

* accepts all valid programs defined here
* rejects invalid programs
* emits behaviorally equivalent shell code

````

---

# ✅ `docs/grammar.ebnf` (clean, render-safe)

```ebnf
program         ::= (toplevel_stmt | function | comment)*

function        ::= "func" identifier "(" params? ")" block

params          ::= param ("," param)*
param           ::= identifier (":" type)?
type            ::= "Str" | "Int" | "Bool"

block           ::= "{" statement* "}"

statement       ::= assign_stmt
                  | run_stmt
                  | print_stmt
                  | print_err_stmt
                  | if_stmt
                  | case_stmt
                  | while_stmt
                  | for_stmt
                  | break_stmt
                  | continue_stmt
                  | return_stmt
                  | exit_stmt
                  | with_stmt
                  | pipe_stmt
                  | spawn_stmt
                  | subshell_stmt
                  | group_stmt
                  | raw_stmt
                  | sh_stmt
                  | comment

assign_stmt     ::= "let" identifier "=" expr
                  | "set" lvalue "=" expr

lvalue          ::= identifier
                  | "env" "." identifier

run_stmt        ::= "run" "(" string ("," string)* ")"

print_stmt      ::= "print" "(" expr ")"
print_err_stmt  ::= "print_err" "(" expr ")"

if_stmt         ::= "if" expr block elif_clause* else_clause?

elif_clause     ::= "elif" expr block
else_clause     ::= "else" block

case_stmt       ::= "case" expr "{" case_arm+ "}"
case_arm        ::= pattern_list "=>" block
pattern_list    ::= pattern ("|" pattern)*
pattern         ::= string | "_"

while_stmt      ::= "while" expr block

for_stmt        ::= "for" identifier "in" iterable block
iterable        ::= "args"
                  | list_lit
                  | "glob" "(" string ")"

with_stmt       ::= "with" with_head block
with_head       ::= "env" "{" env_bind ("," env_bind)* "}"
                  | "cwd" string
                  | "redirect" "{" redirect_bind ("," redirect_bind)* "}"
                  | "options" "{" option_bind ("," option_bind)* "}"

pipe_stmt       ::= pipe_stage ("|" pipe_stage)+
pipe_stage      ::= run_stmt | block

spawn_stmt      ::= "spawn" block
subshell_stmt   ::= "subshell" block
group_stmt      ::= "group" block

raw_stmt        ::= "raw" "{" raw_text "}"
sh_stmt         ::= "sh" "(" string ")"

expr            ::= or_expr
or_expr         ::= and_expr ("||" and_expr)*
and_expr        ::= unary_expr ("&&" unary_expr)*
unary_expr      ::= "!" unary_expr | primary

primary         ::= literal
                  | identifier
                  | "env" "." identifier
                  | "(" expr ")"

literal         ::= string
                  | interpolated_string
                  | int_lit
                  | bool_lit
                  | list_lit

list_lit        ::= "[" (expr ("," expr)*)? "]"

string          ::= "\"" string_char* "\""
interpolated_string ::= "$\"" interp_char* "\""

int_lit         ::= digit+
bool_lit        ::= "true" | "false"

identifier      ::= (letter | "_") (letter | digit | "_")*

comment         ::= "#" comment_char* newline
````

---

## ✅ What changed (important)

* Fixed header hierarchy (no skipped levels)
* Added blank lines before/after code blocks
* Removed inline lists inside paragraphs
* Simplified tables
* Removed nested bullet misuse
* Ensured GitHub-compatible Markdown

This **will render correctly everywhere**.


