use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use sh2c::{lexer, parser, lower, codegen};
use sh2c::ast;

pub fn compile_to_bash(src: &str) -> String {
    let tokens = lexer::lex(src);
    let program = parser::parse(&tokens);
    let ir = lower::lower(program);
    codegen::emit(&ir)
}

pub fn parse_fixture(fixture_name: &str) -> ast::Program {
    let sh2_path = format!("tests/fixtures/{}.sh2", fixture_name);
    let src = fs::read_to_string(&sh2_path).expect("Failed to read fixture");
    let tokens = lexer::lex(&src);
    parser::parse(&tokens)
}

pub fn assert_codegen_matches_snapshot(fixture_name: &str) {
    let sh2_path = format!("tests/fixtures/{}.sh2", fixture_name);
    let expected_path = format!("tests/fixtures/{}.sh.expected", fixture_name);
    
    let src = fs::read_to_string(&sh2_path).expect("Failed to read source fixture");
    let expected = fs::read_to_string(&expected_path).expect("Failed to read expected codegen fixture");
    
    let output = compile_to_bash(&src);
    assert_eq!(output.trim(), expected.trim(), "Codegen mismatch for {}", fixture_name);
}

pub fn assert_codegen_panics(fixture_name: &str, expected_msg_part: &str) {
    let sh2_path = format!("tests/fixtures/{}.sh2", fixture_name);
    let src = fs::read_to_string(&sh2_path).expect("Failed to read source fixture");
    
    // We need to catch unwind, so we can verify the panic message
    let result = std::panic::catch_unwind(|| {
        compile_to_bash(&src)
    });

    match result {
        Ok(_) => panic!("Expected panic during codegen for {}, but it succeeded", fixture_name),
        Err(err) => {
            let msg = if let Some(s) = err.downcast_ref::<&str>() {
                *s
            } else if let Some(s) = err.downcast_ref::<String>() {
                s.as_str()
            } else {
                "Unknown panic message"
            };
            assert!(msg.contains(expected_msg_part), "Expected panic message containing '{}', got '{}'", expected_msg_part, msg);
        }
    }
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

pub fn run_bash_script(bash: &str, env: &[(&str, &str)], args: &[&str]) -> (String, String, i32) {
    let pid = std::process::id();
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir_name = format!("sh2_test_{}_{}", pid, nanos);
    let mut temp_dir = std::env::temp_dir();
    temp_dir.push(dir_name);
    
    fs::create_dir(&temp_dir).expect("Failed to create temp dir");

    let script_path = temp_dir.join("script.sh");
    fs::write(&script_path, bash).expect("Failed to write temp script");
    
    let mut cmd = Command::new("bash");
    cmd.current_dir(&temp_dir);
    for (k, v) in env {
        cmd.env(k, v);
    }
    cmd.arg(&script_path);
    for arg in args {
        cmd.arg(arg);
    }

    let output = cmd.output().expect("Failed to execute bash");
    
    // Best-effort cleanup
    let _ = fs::remove_dir_all(&temp_dir);

    let stdout = String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n");
    let stderr = String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n");
    
    (stdout, stderr, output.status.code().unwrap_or(0))
}

pub fn assert_exec_matches_fixture(fixture_name: &str) {
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
    let env_refs: Vec<(&str, &str)> = env_vars.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();

    let mut args = Vec::new();
    if Path::new(&args_path).exists() {
        let args_content = fs::read_to_string(&args_path).expect("Failed to read args fixture");
        for arg in args_content.split_whitespace() {
            args.push(arg.to_string());
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
