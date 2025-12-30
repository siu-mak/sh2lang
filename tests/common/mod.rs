#![allow(dead_code)]
use sh2c::ast;
pub use sh2c::codegen::TargetShell;
use sh2c::{codegen, lexer, lower, parser};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

// Replaces existing compile_to_shell which took string.
// Note: verify if any tests strictly rely on string-only compilation without files.
// Most tests in common/mod.rs read from fixtures.
use sh2c::loader;

pub fn compile_path_to_shell(path: &Path, target: TargetShell) -> String {
    let program = loader::load_program_with_imports(path)
        .unwrap_or_else(|d| panic!("{}", d.format(path.parent())));
    let opts = sh2c::lower::LowerOptions {
        include_diagnostics: true,
        diag_base_dir: Some(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))),
    };
    let ir = lower::lower_with_options(program, &opts);
    codegen::emit_with_target(&ir, target)
}

pub fn compile_to_bash(src: &str) -> String {
    // Legacy support for string-based tests if any exist (e.g. unit tests not from fixtures)
    // But they won't support imports.
    compile_to_shell(src, TargetShell::Bash)
}
pub fn compile_to_shell(src: &str, target: TargetShell) -> String {
    let sm = sh2c::span::SourceMap::new(src.to_string());
    let tokens = lexer::lex(&sm, src); // lex doesn't return Result in signature? 
    // Wait, I updated lexer::lex to return Result in lexer.rs Step 1905? 
    // Step 1905 updated Lexer::error, but NOT lex function signature explicitly in replacement.
    // Step 1859 updated loader.rs to use `?` on lexer::lex. Implying it returns Result.
    // If lexer::lex was NOT updated to return Result, loader.rs would fail.
    // But loader.rs compiled. So lexer::lex MUST returning Result (or I am confused).
    // Let's check lexer.rs source View 1896 line 44: `pub fn lex... -> Result`.
    // Yes, it returns Result.
    // So tokens line 32 needs unwrap.
    let tokens = tokens.unwrap_or_else(|d| panic!("{}", d.format(None))); 

    let mut program = parser::parse(&tokens, &sm, "inline_test")
        .unwrap_or_else(|d| panic!("{}", d.format(None)));
    program.source_maps.insert("inline_test".to_string(), sm);
    // Note: lower calls generally require accurate file info but here we use "inline_test"
    let opts = sh2c::lower::LowerOptions {
        include_diagnostics: true,
        diag_base_dir: Some(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))),
    };
    let ir = lower::lower_with_options(program, &opts);
    codegen::emit_with_target(&ir, target)
}

pub fn parse_fixture(fixture_name: &str) -> ast::Program {
    let sh2_path = format!("tests/fixtures/{}.sh2", fixture_name);
    let src = fs::read_to_string(&sh2_path).expect("Failed to read fixture");
    let sm = sh2c::span::SourceMap::new(src.clone());
    let tokens = lexer::lex(&sm, &src).unwrap_or_else(|d| panic!("{}", d.format(None)));
    parser::parse(&tokens, &sm, &sh2_path).unwrap_or_else(|d| panic!("{}", d.format(None)))
}

pub fn assert_codegen_matches_snapshot(fixture_name: &str) {
    let sh2_path = format!("tests/fixtures/{}.sh2", fixture_name);
    let expected_path = format!("tests/fixtures/{}.sh.expected", fixture_name);

    let output = compile_path_to_shell(Path::new(&sh2_path), TargetShell::Bash);

    let expected = if std::env::var("SH2C_UPDATE_SNAPSHOTS").is_ok() {
        fs::write(&expected_path, &output).expect("Failed to update snapshot");
        output.clone()
    } else {
        fs::read_to_string(&expected_path).expect("Failed to read expected codegen fixture")
    };

    assert_eq!(
        output.trim(),
        expected.trim(),
        "Codegen mismatch for {}",
        fixture_name
    );
}

pub fn assert_codegen_panics(fixture_name: &str, expected_msg_part: &str) {
    let sh2_path = format!("tests/fixtures/{}.sh2", fixture_name);

    // We need to catch unwind, so we can verify the panic message
    let result =
        std::panic::catch_unwind(|| compile_path_to_shell(Path::new(&sh2_path), TargetShell::Bash));

    match result {
        Ok(_) => panic!(
            "Expected panic during codegen for {}, but it succeeded",
            fixture_name
        ),
        Err(err) => {
            let msg = if let Some(s) = err.downcast_ref::<&str>() {
                *s
            } else if let Some(s) = err.downcast_ref::<String>() {
                s.as_str()
            } else {
                "Unknown panic message"
            };
            assert!(
                msg.contains(expected_msg_part),
                "Expected panic message containing '{}', got '{}'",
                expected_msg_part,
                msg
            );
        }
    }
}
pub fn assert_codegen_matches_snapshot_target(fixture_name: &str, target: TargetShell) {
    let sh2_path = format!("tests/fixtures/{}.sh2", fixture_name);
    let target_str = match target {
        TargetShell::Bash => "bash",
        TargetShell::Posix => "posix",
    };
    let target_expected_path =
        format!("tests/fixtures/{}.{}.sh.expected", fixture_name, target_str);
    let default_expected_path = format!("tests/fixtures/{}.sh.expected", fixture_name);

    let expected_path = if Path::new(&target_expected_path).exists() {
        target_expected_path.clone()
    } else if std::env::var("SH2C_UPDATE_SNAPSHOTS").is_ok() && !Path::new(&default_expected_path).exists() {
         target_expected_path.clone()
    } else {
        default_expected_path.clone()
    };

    let shell_script = compile_path_to_shell(Path::new(&sh2_path), target);

    let expected = if std::env::var("SH2C_UPDATE_SNAPSHOTS").is_ok() {
        if let Some(parent) = Path::new(&expected_path).parent() {
            fs::create_dir_all(parent).expect("Failed to create snapshot dir");
        }
        fs::write(&expected_path, &shell_script).expect("Failed to update snapshot");
        shell_script.clone()
    } else {
        if !Path::new(&expected_path).exists() {
             panic!("Snapshot missing: {}. run with SH2C_UPDATE_SNAPSHOTS=1 to create it.", expected_path);
        }
        fs::read_to_string(&expected_path).expect("Failed to read expected codegen fixture")
    };
    
    // We already have shell_script, no need to call compile_path_to_shell again 
    // (Wait, original code called it at line 121. I moved it up.)
    assert_eq!(
        shell_script.trim(),
        expected.trim(),
        "Codegen mismatch for {} (target={:?})",
        fixture_name,
        target
    );
}

pub fn assert_codegen_panics_target(
    fixture_name: &str,
    target: TargetShell,
    expected_msg_part: &str,
) {
    let sh2_path = format!("tests/fixtures/{}.sh2", fixture_name);

    let result = std::panic::catch_unwind(|| compile_path_to_shell(Path::new(&sh2_path), target));

    match result {
        Ok(_) => panic!(
            "Expected panic during codegen for {}, but it succeeded",
            fixture_name
        ),
        Err(err) => {
            let msg = if let Some(s) = err.downcast_ref::<&str>() {
                *s
            } else if let Some(s) = err.downcast_ref::<String>() {
                s.as_str()
            } else {
                "Unknown panic message"
            };
            assert!(
                msg.contains(expected_msg_part),
                "Expected panic message containing '{}', got '{}'",
                expected_msg_part,
                msg
            );
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

pub fn run_shell_script(
    script: &str,
    shell: &str,
    env: &[(&str, &str)],
    args: &[&str],
    input: Option<&str>,
    fs_setup: Option<&Path>,
) -> (String, String, i32) {
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
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
            stdin
                .write_all(input_str.as_bytes())
                .expect("Failed to write to stdin");
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
    let target_str = match target {
        TargetShell::Bash => "bash",
        TargetShell::Posix => "posix",
    };
    let stdout_path_tgt = format!("tests/fixtures/{}.{}.stdout", fixture_name, target_str);
    let stderr_path_tgt = format!("tests/fixtures/{}.{}.stderr", fixture_name, target_str);
    let status_path_tgt = format!("tests/fixtures/{}.{}.status", fixture_name, target_str);
    
    let stdout_path = if Path::new(&stdout_path_tgt).exists() { stdout_path_tgt } else { format!("tests/fixtures/{}.stdout", fixture_name) };
    let stderr_path = if Path::new(&stderr_path_tgt).exists() { stderr_path_tgt } else { format!("tests/fixtures/{}.stderr", fixture_name) };
    let status_path = if Path::new(&status_path_tgt).exists() { status_path_tgt } else { format!("tests/fixtures/{}.status", fixture_name) };

    let args_path = format!("tests/fixtures/{}.args", fixture_name);
    let env_path = format!("tests/fixtures/{}.env", fixture_name);
    let stdin_path = format!("tests/fixtures/{}.stdin", fixture_name);
    let fs_path = format!("tests/fixtures/{}.fs", fixture_name);

    if !Path::new(&sh2_path).exists() {
        panic!("Fixture {} does not exist", sh2_path);
    }

    // Only run if at least one expectation file exists (checked generically or target specific)
    if !Path::new(&stdout_path).exists()
        && !Path::new(&stderr_path).exists()
        && !Path::new(&status_path).exists()
         // Also allow running if Update is requested, to create new snapshots? 
         // But maybe limit to existing checks to avoid creating garbage for partial tests.
    {
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
                        if Command::new("dash")
                            .arg("-c")
                            .arg("true")
                            .status()
                            .map(|s| s.success())
                            .unwrap_or(false)
                        {
                            "dash".to_string()
                        } else {
                            panic!("SH2C_POSIX_SHELL=dash but dash is not available");
                        }
                    }
                    "sh" => {
                        if Command::new("sh")
                            .arg("-c")
                            .arg("true")
                            .status()
                            .map(|s| s.success())
                            .unwrap_or(false)
                        {
                            "sh".to_string()
                        } else {
                            panic!("SH2C_POSIX_SHELL=sh but sh is not available");
                        }
                    }
                    other => {
                        panic!(
                            "Invalid SH2C_POSIX_SHELL='{}'; expected 'dash' or 'sh'",
                            other
                        );
                    }
                }
            } else {
                // Auto-detection
                if Command::new("dash")
                    .arg("-c")
                    .arg("true")
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
                {
                    "dash".to_string()
                } else if Command::new("sh")
                    .arg("-c")
                    .arg("true")
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
                {
                    "sh".to_string()
                } else {
                    eprintln!(
                        "Skipping POSIX test for {} because 'dash' and 'sh' are not available",
                        fixture_name
                    );
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
                        std::env::current_dir()
                            .unwrap()
                            .join(v)
                            .to_string_lossy()
                            .to_string()
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
    let env_refs: Vec<(&str, &str)> = env_vars
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

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

    let (stdout, stderr, status) = run_shell_script(
        &shell_script,
        &shell_bin,
        &env_refs,
        &args_refs,
        stdin_content.as_deref(),
        fs_setup,
    );

    if Path::new(&stdout_path).exists() || std::env::var("SH2C_UPDATE_SNAPSHOTS").is_ok() {
        if std::env::var("SH2C_UPDATE_SNAPSHOTS").is_ok() {
            fs::write(&stdout_path, &stdout).expect("Failed to update stdout snapshot");
        }
        let expected_stdout = fs::read_to_string(&stdout_path)
            .expect("Failed to read stdout fixture")
            .replace("\r\n", "\n");
        assert_eq!(
            stdout.trim(),
            expected_stdout.trim(),
            "Stdout mismatch for {}.\nStderr:\n{}",
            fixture_name,
            stderr
        );
    }

    if Path::new(&stderr_path).exists() || std::env::var("SH2C_UPDATE_SNAPSHOTS").is_ok() {
        if std::env::var("SH2C_UPDATE_SNAPSHOTS").is_ok() {
            fs::write(&stderr_path, &stderr).expect("Failed to update stderr snapshot");
        }
        let expected_stderr = fs::read_to_string(&stderr_path)
            .expect("Failed to read stderr fixture")
            .replace("\r\n", "\n");
        assert_eq!(
            stderr.trim(),
            expected_stderr.trim(),
            "Stderr mismatch for {}",
            fixture_name
        );
    }

    if Path::new(&status_path).exists() {
        let expected_status: i32 = fs::read_to_string(&status_path)
            .expect("Failed to read status fixture")
            .trim()
            .parse()
            .expect("Invalid status fixture content");
        assert_eq!(
            status, expected_status,
            "Exit code mismatch for {}",
            fixture_name
        );
    }
}

/// Run shell script with additional shell flags (e.g., "-e" for errexit)
pub fn run_shell_script_with_flags(
    script: &str,
    shell: &str,
    flags: &[&str],
    env: &[(&str, &str)],
    args: &[&str],
    input: Option<&str>,
    fs_setup: Option<&Path>,
) -> (String, String, i32) {
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
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
            stdin
                .write_all(input_str.as_bytes())
                .expect("Failed to write to stdin");
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
pub fn assert_exec_matches_fixture_target_with_flags(
    fixture_name: &str,
    target: TargetShell,
    shell_flags: &[&str],
) {
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
        && !Path::new(&status_path).exists()
    {
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
                        if Command::new("dash")
                            .arg("-c")
                            .arg("true")
                            .status()
                            .map(|s| s.success())
                            .unwrap_or(false)
                        {
                            "dash".to_string()
                        } else {
                            panic!("SH2C_POSIX_SHELL=dash but dash is not available");
                        }
                    }
                    "sh" => {
                        if Command::new("sh")
                            .arg("-c")
                            .arg("true")
                            .status()
                            .map(|s| s.success())
                            .unwrap_or(false)
                        {
                            "sh".to_string()
                        } else {
                            panic!("SH2C_POSIX_SHELL=sh but sh is not available");
                        }
                    }
                    other => {
                        panic!(
                            "Invalid SH2C_POSIX_SHELL='{}'; expected 'dash' or 'sh'",
                            other
                        );
                    }
                }
            } else {
                if Command::new("dash")
                    .arg("-c")
                    .arg("true")
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
                {
                    "dash".to_string()
                } else if Command::new("sh")
                    .arg("-c")
                    .arg("true")
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
                {
                    "sh".to_string()
                } else {
                    eprintln!(
                        "Skipping POSIX test for {} because 'dash' and 'sh' are not available",
                        fixture_name
                    );
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
    let env_refs: Vec<(&str, &str)> = env_vars
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

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

    let (stdout, stderr, status) = run_shell_script_with_flags(
        &shell_script,
        &shell_bin,
        shell_flags,
        &env_refs,
        &args_refs,
        stdin_content.as_deref(),
        if Path::new(&fs_path).exists() {
            Some(Path::new(&fs_path))
        } else {
            None
        },
    );

    if Path::new(&stdout_path).exists() {
        let expected_stdout = fs::read_to_string(&stdout_path)
            .expect("Failed to read stdout fixture")
            .replace("\r\n", "\n");
        assert_eq!(
            stdout.trim(),
            expected_stdout.trim(),
            "Stdout mismatch for {}",
            fixture_name
        );
    }

    if Path::new(&stderr_path).exists() {
        let expected_stderr = fs::read_to_string(&stderr_path)
            .expect("Failed to read stderr fixture")
            .replace("\r\n", "\n");
        assert_eq!(
            stderr.trim(),
            expected_stderr.trim(),
            "Stderr mismatch for {}",
            fixture_name
        );
    }

    if Path::new(&status_path).exists() {
        let expected_status: i32 = fs::read_to_string(&status_path)
            .expect("Failed to read status fixture")
            .trim()
            .parse()
            .expect("Invalid status fixture content");
        assert_eq!(
            status, expected_status,
            "Exit code mismatch for {}",
            fixture_name
        );
    }
}

pub fn assert_parse_error_matches_snapshot(fixture_name: &str) {
    let sh2_path = format!("tests/fixtures/{}.sh2", fixture_name);
    let expected_path = format!("tests/fixtures/{}.stderr.expected", fixture_name);

    // Suppress panic printing to stderr during test execution to keep output clean?
    // Rust test harness captures stdout/stderr, but panics print to stderr.
    // We can't easily suppress it without a panic hook, but for now we let it print.
    
    let result = std::panic::catch_unwind(|| {
        // We use compile_path_to_shell which invokes loading/parsing.
        // It should panic on parse error.
        compile_path_to_shell(Path::new(&sh2_path), TargetShell::Bash)
    });

    let err_msg = match result {
        Ok(_) => panic!("Expected parsing/codegen to fail for {}", fixture_name),
        Err(err) => {
             if let Some(s) = err.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = err.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic message".to_string()
            }
        }
    };
    
    // Sanitize panic message (strip thread info/stack trace pointers if present)
    // The test harness or panic handler might erroneously include this info in the payload or we are catching something weird.
    // We want to keep the diagnostic part.
    // Heuristic: remove lines likely to be panic metadata.
    let lines: Vec<&str> = err_msg.lines()
        .filter(|l| {
             !l.contains("thread '") 
             && !l.contains("panicked at") 
             && !l.starts_with("note: run with")
        })
        .collect();
    let output = lines.join("\n").trim().replace("\r\n", "\n");

    let expected = if std::env::var("SH2C_UPDATE_SNAPSHOTS").is_ok() {
        fs::write(&expected_path, &output).expect("Failed to update snapshot");
        output.clone()
    } else {
        fs::read_to_string(&expected_path).unwrap_or_else(|_| "".to_string()) 
    };

    assert_eq!(output, expected.trim(), "Error message mismatch for {}", fixture_name);
}

pub fn strip_spans_program(p: &mut sh2c::ast::Program) {
    p.span = sh2c::span::Span::new(0, 0);
    p.source_maps.clear();
    for f in &mut p.functions {
        strip_spans_fn(f);
    }
    for s in &mut p.top_level {
        strip_spans_stmt(s);
    }
}

pub fn strip_spans_fn(f: &mut sh2c::ast::Function) {
    f.span = sh2c::span::Span::new(0, 0);
    for s in &mut f.body {
        strip_spans_stmt(s);
    }
}

pub fn strip_spans_stmt(s: &mut sh2c::ast::Stmt) {
    s.span = sh2c::span::Span::new(0, 0);
    match &mut s.node {
        sh2c::ast::StmtKind::Let { value, .. } => strip_spans_expr(value),
        sh2c::ast::StmtKind::Run(call) => strip_spans_run_call(call),
        sh2c::ast::StmtKind::Exec(args) => for a in args { strip_spans_expr(a); },
        sh2c::ast::StmtKind::Print(e) => strip_spans_expr(e),
        sh2c::ast::StmtKind::PrintErr(e) => strip_spans_expr(e),
        sh2c::ast::StmtKind::If { cond, then_body, elifs, else_body } => {
            strip_spans_expr(cond);
            for s in then_body { strip_spans_stmt(s); }
            for e in elifs {
                strip_spans_expr(&mut e.cond);
                for s in &mut e.body { strip_spans_stmt(s); }
            }
            if let Some(body) = else_body {
                for s in body { strip_spans_stmt(s); }
            }
        }
        sh2c::ast::StmtKind::While { cond, body } => {
            strip_spans_expr(cond);
            for s in body { strip_spans_stmt(s); }
        }
        sh2c::ast::StmtKind::For { items, body, .. } => {
            for e in items { strip_spans_expr(e); }
            for s in body { strip_spans_stmt(s); }
        }
        sh2c::ast::StmtKind::ForMap { body, .. } => {
            for s in body { strip_spans_stmt(s); }
        }
        sh2c::ast::StmtKind::TryCatch { try_body, catch_body } => {
            for s in try_body { strip_spans_stmt(s); }
            for s in catch_body { strip_spans_stmt(s); }
        }
        sh2c::ast::StmtKind::Pipe(segments) => {
            for c in segments { strip_spans_run_call(c); }
        }
        sh2c::ast::StmtKind::PipeBlocks { segments } => {
            for seg in segments { for s in seg { strip_spans_stmt(s); } }
        }
        sh2c::ast::StmtKind::Return(Some(e)) => strip_spans_expr(e),
        sh2c::ast::StmtKind::Exit(Some(e)) => strip_spans_expr(e),
        sh2c::ast::StmtKind::Cd { path } => strip_spans_expr(path),
        sh2c::ast::StmtKind::Export { value: Some(v), .. } => strip_spans_expr(v),
        sh2c::ast::StmtKind::Source { path } => strip_spans_expr(path),
        sh2c::ast::StmtKind::Call { args, .. } => for a in args { strip_spans_expr(a); },
        sh2c::ast::StmtKind::AndThen { left, right } => {
            for s in left { strip_spans_stmt(s); }
            for s in right { strip_spans_stmt(s); }
        }
        sh2c::ast::StmtKind::OrElse { left, right } => {
            for s in left { strip_spans_stmt(s); }
            for s in right { strip_spans_stmt(s); }
        }
        sh2c::ast::StmtKind::WithEnv { bindings, body } => {
             for (_, v) in bindings { strip_spans_expr(v); }
             for s in body { strip_spans_stmt(s); }
        }
        sh2c::ast::StmtKind::WithCwd { path, body } => {
             strip_spans_expr(path);
             for s in body { strip_spans_stmt(s); }
        }
        sh2c::ast::StmtKind::WithLog { path, body, .. } => {
             strip_spans_expr(path);
             for s in body { strip_spans_stmt(s); }
        }
        sh2c::ast::StmtKind::WithRedirect { stdout, stderr, stdin, body } => {
            if let Some(t) = stdout { strip_spans_redirect(t); }
            if let Some(t) = stderr { strip_spans_redirect(t); }
            if let Some(t) = stdin { strip_spans_redirect(t); }
            for s in body { strip_spans_stmt(s); }
        }
        sh2c::ast::StmtKind::Subshell { body } => {
             for s in body { strip_spans_stmt(s); }
        }
        sh2c::ast::StmtKind::Group { body } => {
             for s in body { strip_spans_stmt(s); }
        }
        sh2c::ast::StmtKind::Spawn { stmt } => strip_spans_stmt(stmt),
        sh2c::ast::StmtKind::Wait(Some(e)) => strip_spans_expr(e),
        sh2c::ast::StmtKind::Set { value, .. } => strip_spans_expr(value),
        sh2c::ast::StmtKind::Case { expr, arms } => {
            strip_spans_expr(expr);
            for arm in arms {
                for s in &mut arm.body { strip_spans_stmt(s); }
            }
        },
        _ => {}
    }
}

pub fn strip_spans_expr(e: &mut sh2c::ast::Expr) {
    e.span = sh2c::span::Span::new(0, 0);
    match &mut e.node {
        sh2c::ast::ExprKind::Command(args) => for a in args { strip_spans_expr(a); },
        sh2c::ast::ExprKind::CommandPipe(segs) => for s in segs { for a in s { strip_spans_expr(a); } },
        sh2c::ast::ExprKind::Concat(l, r) => { strip_spans_expr(l); strip_spans_expr(r); },
        sh2c::ast::ExprKind::Arith { left, right, .. } => { strip_spans_expr(left); strip_spans_expr(right); },
        sh2c::ast::ExprKind::Compare { left, right, .. } => { strip_spans_expr(left); strip_spans_expr(right); },
        sh2c::ast::ExprKind::And(l, r) => { strip_spans_expr(l); strip_spans_expr(r); },
        sh2c::ast::ExprKind::Or(l, r) => { strip_spans_expr(l); strip_spans_expr(r); },
        sh2c::ast::ExprKind::Not(e) => strip_spans_expr(e),
        sh2c::ast::ExprKind::Exists(e) => strip_spans_expr(e),
        sh2c::ast::ExprKind::IsDir(e) => strip_spans_expr(e),
        sh2c::ast::ExprKind::IsFile(e) => strip_spans_expr(e),
        sh2c::ast::ExprKind::IsSymlink(e) => strip_spans_expr(e),
        sh2c::ast::ExprKind::IsExec(e) => strip_spans_expr(e),
        sh2c::ast::ExprKind::IsReadable(e) => strip_spans_expr(e),
        sh2c::ast::ExprKind::IsWritable(e) => strip_spans_expr(e),
        sh2c::ast::ExprKind::IsNonEmpty(e) => strip_spans_expr(e),
        sh2c::ast::ExprKind::BoolStr(e) => strip_spans_expr(e),
        sh2c::ast::ExprKind::Len(e) => strip_spans_expr(e),
        sh2c::ast::ExprKind::Index { list, index } => { strip_spans_expr(list); strip_spans_expr(index); },
        sh2c::ast::ExprKind::Field { base, .. } => strip_spans_expr(base),
        sh2c::ast::ExprKind::Join { list, sep } => { strip_spans_expr(list); strip_spans_expr(sep); },
        sh2c::ast::ExprKind::Count(e) => strip_spans_expr(e),
        sh2c::ast::ExprKind::List(items) => for i in items { strip_spans_expr(i); },
        sh2c::ast::ExprKind::Env(e) => strip_spans_expr(e),
        sh2c::ast::ExprKind::Input(e) => strip_spans_expr(e),
        sh2c::ast::ExprKind::Confirm(e) => strip_spans_expr(e),
        sh2c::ast::ExprKind::Call { args, .. } => for a in args { strip_spans_expr(a); },
        sh2c::ast::ExprKind::MapLiteral(entries) => for (_, v) in entries { strip_spans_expr(v); },
        _ => {}
    }
}

pub fn strip_spans_run_call(c: &mut sh2c::ast::RunCall) {
     for a in &mut c.args { strip_spans_expr(a); }
     for o in &mut c.options { 
         o.span = sh2c::span::Span::new(0, 0);
         strip_spans_expr(&mut o.value);
     }
}

pub fn strip_spans_redirect(t: &mut sh2c::ast::RedirectTarget) {
     match t {
         sh2c::ast::RedirectTarget::File { path, .. } => strip_spans_expr(path),
         _ => {}
     }
}
