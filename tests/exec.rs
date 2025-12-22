use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use sh2c::{lexer, parser, lower, codegen};

fn compile_to_bash(src: &str) -> String {
    let tokens = lexer::lex(src);
    let program = parser::parse(&tokens);
    let ir = lower::lower(program);
    codegen::emit(&ir)
}

fn write_temp_script(prefix: &str, bash: &str) -> std::path::PathBuf {
    let pid = std::process::id();
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let filename = format!("{}_{}_{}.sh", prefix, pid, nanos);
    let mut path = std::env::temp_dir();
    path.push(filename);
    fs::write(&path, bash).expect("Failed to write temp script");
    path
}

fn run_bash_script(bash: &str, env: &[(&str, &str)], args: &[&str]) -> (String, String, i32) {
    let path = write_temp_script("sh2_test", bash);
    
    let mut cmd = Command::new("bash");
    for (k, v) in env {
        cmd.env(k, v);
    }
    cmd.arg(&path);
    for arg in args {
        cmd.arg(arg);
    }

    let output = cmd.output().expect("Failed to execute bash");
    
    // Best-effort cleanup
    let _ = fs::remove_file(&path);

    let stdout = String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n");
    let stderr = String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n");
    
    (stdout, stderr, output.status.code().unwrap_or(0))
}

fn assert_exec_matches_fixture(fixture_name: &str) {
    let sh2_path = format!("tests/fixtures/{}.sh2", fixture_name);
    let stdout_path = format!("tests/fixtures/{}.stdout", fixture_name);
    let stderr_path = format!("tests/fixtures/{}.stderr", fixture_name);
    let status_path = format!("tests/fixtures/{}.status", fixture_name);
    let args_path = format!("tests/fixtures/{}.args", fixture_name);
    let env_path = format!("tests/fixtures/{}.env", fixture_name);

    if !Path::new(&sh2_path).exists() {
        panic!("Fixture {} does not exist", sh2_path);
    }

    // Only run if at least one expectation file exists
    if !Path::new(&stdout_path).exists() 
       && !Path::new(&stderr_path).exists() 
       && !Path::new(&status_path).exists() {
        return; 
    }

    let src = fs::read_to_string(&sh2_path).expect("Failed to read fixture");
    let bash = compile_to_bash(&src);

    let mut env_vars = Vec::new();
    if Path::new(&env_path).exists() {
        let env_content = fs::read_to_string(&env_path).expect("Failed to read env fixture");
        for line in env_content.lines() {
            if let Some((k, v)) = line.split_once('=') {
                env_vars.push((k.to_string(), v.to_string()));
            }
        }
    }
    // We need env_vars references for run_bash_script
    let env_refs: Vec<(&str, &str)> = env_vars.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();

    let mut args = Vec::new();
    if Path::new(&args_path).exists() {
        let args_content = fs::read_to_string(&args_path).expect("Failed to read args fixture");
        for line in args_content.lines() {
            if !line.trim().is_empty() {
                args.push(line.to_string());
            }
        }
    }
    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let (stdout, stderr, status) = run_bash_script(&bash, &env_refs, &args_refs);

    if Path::new(&stdout_path).exists() {
        let expected_stdout = fs::read_to_string(&stdout_path).expect("Failed to read stdout fixture")
            .replace("\r\n", "\n");
        assert_eq!(stdout.trim(), expected_stdout.trim(), "Stdout mismatch for {}", fixture_name);
    }

    if Path::new(&stderr_path).exists() {
        let expected_stderr = fs::read_to_string(&stderr_path).expect("Failed to read stderr fixture")
            .replace("\r\n", "\n");
        assert_eq!(stderr.trim(), expected_stderr.trim(), "Stderr mismatch for {}", fixture_name);
    }

    if Path::new(&status_path).exists() {
        let expected_status: i32 = fs::read_to_string(&status_path).expect("Failed to read status fixture")
            .trim().parse().expect("Invalid status fixture content");
        assert_eq!(status, expected_status, "Exit code mismatch for {}", fixture_name);
    }
}

// Inline tests refactored or removed in favor of fixture tests where applicable
// We'll keep the specialized inline ones if they test things not covered by fixtures, 
// but mostly we want to move to fixtures.

#[test]
fn exec_for_list_var() {
    assert_exec_matches_fixture("for_list_var");
}

#[test]
fn exec_pipe_basic() { assert_exec_matches_fixture("pipe_basic"); }
#[test]
fn exec_case_wildcard() { assert_exec_matches_fixture("case_wildcard"); }
#[test]
fn exec_while_basic() { assert_exec_matches_fixture("while_basic"); }
#[test]
fn exec_for_list() { assert_exec_matches_fixture("for_list"); }
#[test]
fn exec_if_true_literal() { assert_exec_matches_fixture("if_true_literal"); }
#[test]
fn exec_if_bool_and() { assert_exec_matches_fixture("if_bool_and"); }
#[test]
fn exec_exists_check() { assert_exec_matches_fixture("exists_check"); }
#[test]
fn exec_with_cwd_check() { assert_exec_matches_fixture("with_cwd_check"); }

#[test]
fn exec_hello() { assert_exec_matches_fixture("hello_exec"); }
#[test]
fn exec_if_env_true() { assert_exec_matches_fixture("if_env_true"); }
#[test]
fn exec_if_env_false() { assert_exec_matches_fixture("if_env_false"); }
#[test]
fn exec_print_err() { assert_exec_matches_fixture("print_err_exec"); }
#[test]
fn exec_for_args() { assert_exec_matches_fixture("for_args"); }
#[test]
fn exec_let_args() { assert_exec_matches_fixture("let_args"); }
#[test]
fn exec_run_args() { assert_exec_matches_fixture("run_args"); }
#[test]
fn exec_print_args() { assert_exec_matches_fixture("print_args"); }
#[test]
fn exec_try_catch_basic() { assert_exec_matches_fixture("try_catch_basic"); }
