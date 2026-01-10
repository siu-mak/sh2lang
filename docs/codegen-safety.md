# Shell Code Generation Safety Guidelines

This document provides safety guidelines for contributors working on the sh2c compiler's code generation (`src/codegen.rs`).

## Rule: Use Positional Parameters for Dynamic Data in -c Scripts

When the compiler generates `bash -c` or `sh -c` invocations for **internal helpers**, dynamic values MUST be passed as positional parameters, never interpolated into the script string.

**This rule allows `-c` usage** when necessary, but requires the safe positional-parameter pattern.

### ✅ CORRECT Pattern - Positional Parameters

```rust
// Script is a constant literal, dynamic values passed as separate arguments
let script = sh_single_quote("echo \"$1\" \"$2\"");
let arg1 = emit_val(value_a, target)?;
let arg2 = emit_val(value_b, target)?;
let cmd = format!("bash -c {} _ {} {}", script, arg1, arg2);
```

Generated shell code:
```bash
bash -c 'echo "$1" "$2"' _ "value_a" "value_b"
```

**Key elements**:
- Script literal is constant (no dynamic content)
- Dummy `_` placeholder for `$0`
- Dynamic values passed as separate arguments
- Script references them via `"$1"`, `"$2"`, etc.


### ❌ WRONG Pattern - Interpolation

```rust
// NEVER DO THIS: Dynamic values embedded in script string
let cmd = format!("bash -c 'echo {} {}'", emit_val(a, target)?, emit_val(b, target)?);
```

This creates quoting bugs and potential injection vulnerabilities.

### Exception: User-Facing sh(expr)

The `sh(expr)` feature (`Cmd::Raw` in `src/codegen.rs`) is **intentionally unsafe** and explicitly documented as an escape hatch (see `docs/language.md` section 6.3 and ticket S1). It is the ONLY acceptable use of dynamic shell code execution because:

1. It's user-facing (users write `sh("...")` in their code)
2. It's documented with prominent warnings about injection risks
3. The dynamic content comes from user code, not compiler logic

### Rationale

**Why positional parameters?**
- Shell properly quotes each argument
- No risk of word splitting or glob expansion
- No quoting edge cases to handle
- Immune to injection attacks

**Why avoid interpolation?**
- Easy to introduce quoting bugs
- Difficult to audit for safety
- Fragile under edge cases (quotes, newlines, special chars)

### Future Development

If you need to add compiler-internal features that execute shell code:

1. **Prefer direct shell syntax** over `-c` when possible
2. **If `-c` is necessary**, use the positional parameter pattern
3. **Add tests** with hostile inputs (spaces, quotes, `$`, `;`, newlines)
4. **Document** why `-c` is needed vs direct syntax

### Regression Protection

The test `tests/codegen_shell_c_safety.rs` enforces that any compiler-generated `-c` usage follows the safe positional-parameter pattern. 

**For non-sh() features**:
- If no `-c` usage → PASS
- If `-c` present → REQUIRE safe pattern:
  - Single-quoted script literal
  - Dummy `_` placeholder for `$0`
  - Positional parameter references (`"$1"`, `"$2"`)
  - NO variable interpolation (`$var`, `${var}`)

**For sh(expr) (user-facing escape hatch)**:
- Detected by signature (`__sh2_cmd=` variable + execution pattern)
- Allowed to use `-c` unsafely (intentional, documented in S1)
- This is the ONLY exception to the positional-parameter requirement

This test will fail if future changes introduce compiler-internal `-c` patterns that interpolate dynamic values into script strings.
