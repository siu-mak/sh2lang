#![allow(dead_code)]
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use sh2c::{lexer, parser, lower, codegen};
use sh2c::ast;
pub use sh2c::codegen::TargetShell;

// Replaces existing compile_to_shell which took string.
// Note: verify if any tests strictly rely on string-only compilation without files.
// Most tests in common/mod.rs read from fixtures.
use sh2c::loader;

pub fn compile_path_to_shell(path: &Path, target: TargetShell) -> String {
    let program = loader::load_program_with_imports(path);
    let ir = lower::lower(program);
    codegen::emit_with_target(&ir, target)
}

pub fn compile_to_bash(src: &str) -> String {
    // Legacy support for string-based tests if any exist (e.g. unit tests not from fixtures)
    // But they won't support imports.
    compile_to_shell(src, TargetShell::Bash)
}

pub fn compile_to_shell(src: &str, target: TargetShell) -> String {
    let sm = sh2c::span::SourceMap::new(src.to_string());
    let tokens = lexer::lex(&sm, src);
    let program = parser::parse(&tokens, &sm, "inline_test");
    // Note: lower calls generally require accurate file info but here we use "inline_test"
    let ir = lower::lower(program);
    codegen::emit_with_target(&ir, target)
}

pub fn parse_fixture(fixture_name: &str) -> ast::Program {
    let sh2_path = format!("tests/fixtures/{}.sh2", fixture_name);
    let src = fs::read_to_string(&sh2_path).expect("Failed to read fixture");
    let sm = sh2c::span::SourceMap::new(src.clone());
    let tokens = lexer::lex(&sm, &src);
    parser::parse(&tokens, &sm, &sh2_path)
}

pub fn assert_codegen_matches_snapshot(fixture_name: &str) {
    let sh2_path = format!("tests/fixtures/{}.sh2", fixture_name);
    let expected_path = format!("tests/fixtures/{}.sh.expected", fixture_name);
    
    let expected = fs::read_to_string(&expected_path).expect("Failed to read expected codegen fixture");
    let output = compile_path_to_shell(Path::new(&sh2_path), TargetShell::Bash);
    assert_eq!(output.trim(), expected.trim(), "Codegen mismatch for {}", fixture_name);
}

pub fn assert_codegen_panics(fixture_name: &str, expected_msg_part: &str) {
    let sh2_path = format!("tests/fixtures/{}.sh2", fixture_name);
    
    // We need to catch unwind, so we can verify the panic message
    let result = std::panic::catch_unwind(|| {
        compile_path_to_shell(Path::new(&sh2_path), TargetShell::Bash)
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
pub fn assert_codegen_matches_snapshot_target(fixture_name: &str, target: TargetShell) {
    let sh2_path = format!("tests/fixtures/{}.sh2", fixture_name);
    let target_str = match target {
        TargetShell::Bash => "bash",
        TargetShell::Posix => "posix",
    };
    let target_expected_path = format!("tests/fixtures/{}.{}.sh.expected", fixture_name, target_str);
    let default_expected_path = format!("tests/fixtures/{}.sh.expected", fixture_name);
    
    let expected_path = if Path::new(&target_expected_path).exists() {
        target_expected_path.clone()
    } else {
        default_expected_path.clone()
    };

    let expected = fs::read_to_string(&expected_path).expect("Failed to read expected codegen fixture");
    
    let shell_script = compile_path_to_shell(Path::new(&sh2_path), target);
    assert_eq!(shell_script.trim(), expected.trim(), "Codegen mismatch for {} (target={:?})", fixture_name, target);
}

pub fn assert_codegen_panics_target(fixture_name: &str, target: TargetShell, expected_msg_part: &str) {
    let sh2_path = format!("tests/fixtures/{}.sh2", fixture_name);
    
    let result = std::panic::catch_unwind(|| {
        compile_path_to_shell(Path::new(&sh2_path), target)
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


pub fn run_bash_script(bash: &str, env: &[(&str, &str)], args: &[&str]) -> (String, String, i32) {
    run_shell_script(bash, "bash", env, args, None, None)
}

fn copy_dir_all(src: &Path, dst: &Path) {
    if !dst.exists() {
        fs::create_dir_all(dst).expect("Failed to create dst dir");
    }
    for entry in fs::read_dir(src).expect("Failed to read src dir") {
        let entry = entry.expect("Failed to read entry");
        let ty = entry.file_type().expect("Failed to get file type");
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path);
        } else {
            fs::copy(entry.path(), &dst_path).expect("Failed to copy file");
        }
    }
}

pub fn run_shell_script(script: &str, shell: &str, env: &[(&str, &str)], args: &[&str], input: Option<&str>, fs_setup: Option<&Path>) -> (String, String, i32) {
    let pid = std::process::id();
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir_name = format!("sh2_test_{}_{}", pid, nanos);
    let mut temp_dir = std::env::temp_dir();
    temp_dir.push(dir_name);
    
    fs::create_dir(&temp_dir).expect("Failed to create temp dir");

    if let Some(src) = fs_setup {
        copy_dir_all(src, &temp_dir);
    }

    let script_path = temp_dir.join("script.sh");
    fs::write(&script_path, script).expect("Failed to write temp script");
    
    let mut cmd = Command::new(shell);
    cmd.current_dir(&temp_dir);
    for (k, v) in env {
        cmd.env(k, v);
    }
    cmd.arg(&script_path);
    for arg in args {
        cmd.arg(arg);
    }
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    if input.is_some() {
        cmd.stdin(std::process::Stdio::piped());
    }

    let mut child = cmd.spawn().expect("Failed to spawn shell");

    if let Some(input_str) = input {
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(input_str.as_bytes()).expect("Failed to write to stdin");
        }
    }

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(_) => {
             // If shell is missing (e.g. dash), return fake error or handle gracefully
             // Tests should check if shell exists before calling?
             // Or we just fail.
             // But plan said "tests should detect if dash exists".
             // We can return -1 status if failed to start?
             // But callers expect output.
             // We'll panic here if fail, but tests should gate.
             let _ = fs::remove_dir_all(&temp_dir);
             panic!("Failed to execute {}", shell);
        }
    };
    
    // Best-effort cleanup
    let _ = fs::remove_dir_all(&temp_dir);

    let stdout = String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n");
    let stderr = String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n");
    
    (stdout, stderr, output.status.code().unwrap_or(0))
}

pub fn assert_exec_matches_fixture(fixture_name: &str) {
    assert_exec_matches_fixture_target(fixture_name, TargetShell::Bash);
}

pub fn assert_exec_matches_fixture_target(fixture_name: &str, target: TargetShell) {
    let sh2_path = format!("tests/fixtures/{}.sh2", fixture_name);
    let stdout_path = format!("tests/fixtures/{}.stdout", fixture_name);
    let stderr_path = format!("tests/fixtures/{}.stderr", fixture_name);
    let status_path = format!("tests/fixtures/{}.status", fixture_name);
    let args_path = format!("tests/fixtures/{}.args", fixture_name);
    let env_path = format!("tests/fixtures/{}.env", fixture_name);
    let stdin_path = format!("tests/fixtures/{}.stdin", fixture_name);
    let fs_path = format!("tests/fixtures/{}.fs", fixture_name);

    if !Path::new(&sh2_path).exists() {
        panic!("Fixture {} does not exist", sh2_path);
    }

    // Only run if at least one expectation file exists
    if !Path::new(&stdout_path).exists() 
       && !Path::new(&stderr_path).exists() 
       && !Path::new(&status_path).exists() {
        return; 
    }

    //     let src = fs::read_to_string(&sh2_path).expect("Failed to read fixture");
    let shell_script = compile_path_to_shell(Path::new(&sh2_path), target);
    
    let shell_bin = match target {
        TargetShell::Bash => "bash".to_string(),
        TargetShell::Posix => {
            // CI Support: Allow strict enforcement of a specific POSIX shell via env var.
            // If SH2C_POSIX_SHELL is set, we use it. Allowed values: "dash", "sh".
            // If set to anything else, or if the requested shell is missing, we PANIC.
            // If unset, we fall back to auto-detection (dash -> sh -> skip).
            if let Ok(strict_shell) = std::env::var("SH2C_POSIX_SHELL") {
                match strict_shell.as_str() {
                    "dash" => {
                        if Command::new("dash").arg("-c").arg("true").status().map(|s| s.success()).unwrap_or(false) {
                            "dash".to_string()
                        } else {
                            panic!("SH2C_POSIX_SHELL=dash but dash is not available");
                        }
                    }
                    "sh" => {
                        if Command::new("sh").arg("-c").arg("true").status().map(|s| s.success()).unwrap_or(false) {
                            "sh".to_string()
                        } else {
                            panic!("SH2C_POSIX_SHELL=sh but sh is not available");
                        }
                    }
                    other => {
                        panic!("Invalid SH2C_POSIX_SHELL='{}'; expected 'dash' or 'sh'", other);
                    }
                }
            } else {
                // Auto-detection
                if Command::new("dash").arg("-c").arg("true").status().map(|s| s.success()).unwrap_or(false) {
                    "dash".to_string()
                } else if Command::new("sh").arg("-c").arg("true").status().map(|s| s.success()).unwrap_or(false) {
                    "sh".to_string()
                } else {
                    eprintln!("Skipping POSIX test for {} because 'dash' and 'sh' are not available", fixture_name);
                    return;
                }
            }
        }
    };

    let mut env_vars = Vec::new();
    if Path::new(&env_path).exists() {
        let env_content = fs::read_to_string(&env_path).expect("Failed to read env fixture");
        for line in env_content.lines() {
            if let Some((k, v)) = line.split_once('=') {
                if k == "PATH" {
                   let current_path = std::env::var("PATH").unwrap_or_default();
                   let new_segment = if v.starts_with("tests/fixtures") {
                        std::env::current_dir().unwrap().join(v).to_string_lossy().to_string()
                   } else {
                        v.to_string()
                   };
                   let new_path = format!("{}:{}", new_segment, current_path);
                   env_vars.push((k.to_string(), new_path));
                } else if v.starts_with("tests/fixtures") {
                   let abs_path = std::env::current_dir().unwrap().join(v);
                   env_vars.push((k.to_string(), abs_path.to_string_lossy().to_string()));
                } else {
                   env_vars.push((k.to_string(), v.to_string()));
                }
            }
        }
    }
    let env_refs: Vec<(&str, &str)> = env_vars.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();

    let mut args = Vec::new();
    if Path::new(&args_path).exists() {
        let args_content = fs::read_to_string(&args_path).expect("Failed to read args fixture");
        for arg in args_content.lines() {
            if !arg.trim().is_empty() {
                args.push(arg.to_string());
            }
        }
    }
    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let stdin_content = if Path::new(&stdin_path).exists() {
        Some(fs::read_to_string(&stdin_path).expect("Failed to read stdin fixture"))
    } else {
        None
    };
    
    let fs_setup = if Path::new(&fs_path).exists() {
        Some(Path::new(&fs_path))
    } else {
        None
    };

    let (stdout, stderr, status) = run_shell_script(&shell_script, &shell_bin, &env_refs, &args_refs, stdin_content.as_deref(), fs_setup);

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

/// Run shell script with additional shell flags (e.g., "-e" for errexit)
pub fn run_shell_script_with_flags(script: &str, shell: &str, flags: &[&str], env: &[(&str, &str)], args: &[&str], input: Option<&str>, fs_setup: Option<&Path>) -> (String, String, i32) {
    let pid = std::process::id();
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir_name = format!("sh2_test_{}_{}", pid, nanos);
    let mut temp_dir = std::env::temp_dir();
    temp_dir.push(dir_name);
    
    fs::create_dir(&temp_dir).expect("Failed to create temp dir");

    if let Some(src) = fs_setup {
        copy_dir_all(src, &temp_dir);
    }

    let script_path = temp_dir.join("script.sh");
    fs::write(&script_path, script).expect("Failed to write temp script");
    
    let mut cmd = Command::new(shell);
    cmd.current_dir(&temp_dir);
    for (k, v) in env {
        cmd.env(k, v);
    }
    // Add flags before the script path
    for flag in flags {
        cmd.arg(flag);
    }
    cmd.arg(&script_path);
    for arg in args {
        cmd.arg(arg);
    }
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    if input.is_some() {
        cmd.stdin(std::process::Stdio::piped());
    }

    let mut child = cmd.spawn().expect("Failed to spawn shell");

    if let Some(input_str) = input {
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(input_str.as_bytes()).expect("Failed to write to stdin");
        }
    }

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(_) => {
             let _ = fs::remove_dir_all(&temp_dir);
             panic!("Failed to execute {}", shell);
        }
    };
    
    let _ = fs::remove_dir_all(&temp_dir);

    let stdout = String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n");
    let stderr = String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n");
    
    (stdout, stderr, output.status.code().unwrap_or(0))
}

/// Like assert_exec_matches_fixture_target but invokes shell with extra flags
pub fn assert_exec_matches_fixture_target_with_flags(fixture_name: &str, target: TargetShell, shell_flags: &[&str]) {
    let sh2_path = format!("tests/fixtures/{}.sh2", fixture_name);
    let stdout_path = format!("tests/fixtures/{}.stdout", fixture_name);
    let stderr_path = format!("tests/fixtures/{}.stderr", fixture_name);
    let status_path = format!("tests/fixtures/{}.status", fixture_name);
    let args_path = format!("tests/fixtures/{}.args", fixture_name);
    let env_path = format!("tests/fixtures/{}.env", fixture_name);
    let stdin_path = format!("tests/fixtures/{}.stdin", fixture_name);
    let fs_path = format!("tests/fixtures/{}.fs", fixture_name);

    if !Path::new(&sh2_path).exists() {
        panic!("Fixture {} does not exist", sh2_path);
    }

    if !Path::new(&stdout_path).exists() 
       && !Path::new(&stderr_path).exists() 
       && !Path::new(&status_path).exists() {
        return; 
    }

    let src = fs::read_to_string(&sh2_path).expect("Failed to read fixture");
    let shell_script = compile_to_shell(&src, target);
    
    let shell_bin = match target {
        TargetShell::Bash => "bash".to_string(),
        TargetShell::Posix => {
            if let Ok(strict_shell) = std::env::var("SH2C_POSIX_SHELL") {
                match strict_shell.as_str() {
                    "dash" => {
                        if Command::new("dash").arg("-c").arg("true").status().map(|s| s.success()).unwrap_or(false) {
                            "dash".to_string()
                        } else {
                            panic!("SH2C_POSIX_SHELL=dash but dash is not available");
                        }
                    }
                    "sh" => {
                        if Command::new("sh").arg("-c").arg("true").status().map(|s| s.success()).unwrap_or(false) {
                            "sh".to_string()
                        } else {
                            panic!("SH2C_POSIX_SHELL=sh but sh is not available");
                        }
                    }
                    other => {
                        panic!("Invalid SH2C_POSIX_SHELL='{}'; expected 'dash' or 'sh'", other);
                    }
                }
            } else {
                if Command::new("dash").arg("-c").arg("true").status().map(|s| s.success()).unwrap_or(false) {
                    "dash".to_string()
                } else if Command::new("sh").arg("-c").arg("true").status().map(|s| s.success()).unwrap_or(false) {
                    "sh".to_string()
                } else {
                    eprintln!("Skipping POSIX test for {} because 'dash' and 'sh' are not available", fixture_name);
                    return;
                }
            }
        }
    };

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
        for arg in args_content.lines() {
            if !arg.trim().is_empty() {
                args.push(arg.to_string());
            }
        }
    }
    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let stdin_content = if Path::new(&stdin_path).exists() {
        Some(fs::read_to_string(&stdin_path).expect("Failed to read stdin fixture"))
    } else {
        None
    };

    let (stdout, stderr, status) = run_shell_script_with_flags(&shell_script, &shell_bin, shell_flags, &env_refs, &args_refs, stdin_content.as_deref(), if Path::new(&fs_path).exists() { Some(Path::new(&fs_path)) } else { None });

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