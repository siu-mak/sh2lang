//! Integration tests for sh2 string expansion semantics (Phase 2 Refinement).
//! enforced semantics:
//! 1. Normal strings "..." are STRICT LITERALS (no implicit expansion).
//! 2. Explicit interpolation $"..." expands {expr} but treats $ as literal text.
//! 3. Raw shell contexts allow inner expansion but must be quoted for outer safety.

use std::process::Command;
use std::fs;

// --- Harness Helper ---

fn compile_and_run_str(name: &str, sh2_code: &str, env_vars: &[(&str, &str)]) -> String {
    let temp_dir = tempfile::Builder::new().prefix("sh2_sem_test").tempdir().expect("tempdir");
    let src_path = temp_dir.path().join(format!("{}.sh2", name));
    let out_path = temp_dir.path().join(format!("{}.sh", name));
    
    fs::write(&src_path, sh2_code).expect("write src");
    
    let sh2c = env!("CARGO_BIN_EXE_sh2c");
    let status = Command::new(sh2c)
        .arg(&src_path)
        .arg("-o")
        .arg(&out_path)
        .status()
        .expect("exec sh2c");
        
    if !status.success() {
        panic!("Compilation failed for {}:\nSource:\n{}", name, sh2_code);
    }
    
    let mut cmd = Command::new("bash");
    cmd.arg(&out_path);
    for (k, v) in env_vars {
        cmd.env(k, v);
    }
    
    let output = cmd.output().expect("exec bash");
    if !output.status.success() {
        // Enforce successful execution unless testing failure
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let script_content = fs::read_to_string(&out_path).unwrap_or_else(|_| "<unreadable>".into());
        panic!("Runtime failed for {} (exit code {:?})\nStderr:\n{}\nStdout:\n{}\nScript Path: {}\nScript Content:\n{}", 
            name, output.status.code(), stderr, stdout, out_path.display(), script_content);
    }
    
    String::from_utf8(output.stdout).expect("utf8")
}

// Overload to get generated script content for safety checks
fn compile_inspect_run(name: &str, sh2_code: &str, env_vars: &[(&str, &str)]) -> (String, String) {
    let temp_dir = tempfile::Builder::new().prefix("sh2_sem_test_insp").tempdir().expect("tempdir");
    let src_path = temp_dir.path().join(format!("{}.sh2", name));
    let out_path = temp_dir.path().join(format!("{}.sh", name));
    
    fs::write(&src_path, sh2_code).expect("write src");
    
    let sh2c = env!("CARGO_BIN_EXE_sh2c");
    let status = Command::new(sh2c)
        .arg(&src_path)
        .arg("-o")
        .arg(&out_path)
        .status()
        .expect("exec sh2c");
        
    if !status.success() {
        panic!("Compilation failed for {}", name);
    }
    
    let gen_script = fs::read_to_string(&out_path).expect("read generated");
    
    let mut cmd = Command::new("bash");
    cmd.arg(&out_path);
    for (k, v) in env_vars {
        cmd.env(k, v);
    }
    let output = cmd.output().expect("exec bash");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!("Runtime failed for {} (exit code {:?})\nStderr:\n{}\nStdout:\n{}\nScript Content:\n{}", 
            name, output.status.code(), stderr, stdout, gen_script);
    }

    let stdout = String::from_utf8(output.stdout).expect("utf8");
    
    (gen_script, stdout)
}

// --- Tests ---

#[test]
fn test_strict_normal_string_print() {
    // 1. Strict Normal String: print("$FOO") -> literal "$FOO"
    let code = r#"
        func main() {
            print("$FOO")
        }
    "#;
    let out = compile_and_run_str("strict_print", code, &[("FOO", "EXPANDED")]);
    assert_eq!(out.trim(), "$FOO", "Normal string must be strict literal (no expansion)");
}

#[test]
fn test_strict_argv() {
    // 2. Argv Strict: run("echo", "$FOO") -> literal "$FOO"
    let code = r#"
        func main() {
            run("echo", "$FOO")
        }
    "#;
    let out = compile_and_run_str("strict_argv", code, &[("FOO", "EXPANDED")]);
    // echo $FOO -> prints $FOO
    assert_eq!(out.trim(), "$FOO", "Argv must be strict literal");
}

#[test]
fn test_dpkg_regression() {
    // 3. Dpkg Regression
    // run("dpkg-query","-W","-f","${Package}\n","bash", allow_fail=true)
    // Check status print.
    
    // Robust availability check (no `which` crate)
    if Command::new("dpkg-query").arg("--version").output().is_err() {
        println!("Skipping dpkg test (dpkg-query not found)");
        return;
    }

    let code = r#"
        func main() {
            run("dpkg-query","-W","-f","${Package}\n","bash", allow_fail=true)
            print("status=" & status())
        }
    "#;
    // Pass Package=BAD ENV to ensure it is NOT expanded
    let out = compile_and_run_str("dpkg_repro", code, &[("Package", "BAD")]);
    
    assert!(!out.contains("BAD"), "Environment variable 'Package' was expanded!");
    // Expect output to contain package name "bash" (from dpkg-query)
    assert!(out.contains("bash"), "Output should contain package name 'bash'");
    assert!(out.contains("status=0"), "Status should be 0 for valid package");
}

#[test]
fn test_explicit_interpolation() {
    // 4. Explicit Interpolation: $"hi {name}"
    // Note: Depends on parser support for $"...".
    let code = r#"
        func main() {
            let name = "world"
            print($"hi {name}")
        }
    "#;
    let out = compile_and_run_str("explicit_interp", code, &[]);
    assert_eq!(out.trim(), "hi world");
}

#[test]
fn test_explicit_interpolation_literal_dollar() {
    // 5. Explicit Interp with Literal $: $"${name}"
    // $ is literal text, {name} is expr.
    // Result should be "$world".
    let code = r#"
        func main() {
            let name = "world"
            print($"${name}")
        }
    "#;
    let out = compile_and_run_str("interp_literal_dollar", code, &[]);
    assert_eq!(out.trim(), "$world", "Expected literal '$' prefix and interpolated '{{name}}'");
}

#[test]
fn test_raw_shell_expansion_safety() {
    // 6. Raw Shell: sh("echo $FOO") -> expanded
    // 7. Raw Shell via Run: run("sh", "-c", "echo $FOO") -> expanded
    // Plus audit generated code safety.
    
    let code = r#"
        func main() {
            sh("echo FROM_SH: $FOO")
            run("sh", "-c", "echo FROM_RUN: $FOO")
        }
    "#;
    let (gen_script, out) = compile_inspect_run("raw_shell", code, &[("FOO", "EXPANDED")]);
    
    // Runtime check
    assert!(out.contains("FROM_SH: EXPANDED"), "sh() should expand in inner shell");
    assert!(out.contains("FROM_RUN: EXPANDED"), "run(sh -c) should expand in inner shell");
    
    // Codegen Safety Check:
    // Outer bash must quote the payload.
    // 1. The payload strings must appear in the script.
    assert!(gen_script.contains("echo FROM_SH: $FOO"), "sh() payload missing from script");
    assert!(gen_script.contains("echo FROM_RUN: $FOO"), "run() payload missing from script");
    
    // 2. The *expanded* value must NO appear in the generated script (outer expansion forbidden).
    // (This ensures $FOO wasn't expanded by sh2c codegen or early bash pass)
    // Actually, checking "EXPANDED" is tricky because we set FOO=EXPANDED in env.
    // But gen_script is static code. It definitively won't contain "EXPANDED".
    // 3. Ensure they appear in quoted context.
    // Heuristic: Ensure no unquoted $FOO instances.
    // Better: Check for single-quoted payload.
    // We allow: 'echo FROM_SH: $FOO' OR similar safe quoting.
    
    fn is_safely_quoted(script: &str, substring: &str) -> bool {
        // Simple heuristic: find substring, check if preceded by ' and followed by '
        // Handling possibly lines or other quoting forms fits the usage "match actual codegen patterns".
        // Current codegen uses sh_single_quote which produces '...'
        // So strict check for '...' is acceptable if we know codegen matches it.
        // User requested: "Or allow either single quotes or an escaping form you actually use... match actual codegen output structure"
        let quoted = format!("'{}'", substring);
        script.contains(&quoted)
    }

    assert!(is_safely_quoted(&gen_script, "echo FROM_SH: $FOO"), "sh() payload valid not single-quoted");
    assert!(is_safely_quoted(&gen_script, "echo FROM_RUN: $FOO"), "run() payload valid not single-quoted");
}

#[test]
fn test_heredoc_literal_safety() {
    // 8. Heredoc Literal
    // Syntax verified: with redirect { stdin: heredoc("...") } { ... }
    
    let code = r#"
        func main() {
            with redirect { stdin: heredoc("$FOO") } {
                run("cat")
            }
        }
    "#;
    
    let (gen_script, out) = compile_inspect_run("heredoc_literal", code, &[("FOO", "EXPANDED")]);
    
    // Runtime check: cat should output "$FOO" literally
    assert_eq!(out.trim(), "$FOO", "Heredoc content must be strict literal");
    
    // Codegen Safety Check:
    // Should use quoted delimiter <<'...'
    // Note: delimiter is dynamic (e.g. __SH2_EOF_1__), so we check for the quoting pattern.
    assert!(gen_script.contains("<<'"), "Generated bash must use quoted heredoc delimiter (e.g. <<'EOF') to prevent expansion");
}
