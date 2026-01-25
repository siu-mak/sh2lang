use sh2c::ast::ExprKind;
use sh2c::ast::StmtKind;
use sh2c::ast::{Expr, RedirectOutputTarget, RedirectInputTarget, Stmt};
use std::path::Path;
mod common;
use common::*;

#[test]
fn parse_redirect_io() {
    let program = parse_fixture("redirect_io");
    let func = &program.functions[0];

    // stmt0: let f = capture("mktemp")
    assert!(matches!(
        func.body[0],
        Stmt {
            node: StmtKind::Let { .. },
            ..
        }
    ));

    // stmt1: with redirect { stdout: file(f) } { print("out") }
    if let Stmt {
        node:
            StmtKind::WithRedirect {
                stdout,
                stderr,
                stdin,
                body,
            },
        ..
    } = &func.body[1]
    {
        assert!(stdin.is_none());
        assert!(stderr.is_none());
        // stdout is now Option<Vec<Spanned<RedirectOutputTarget>>>
        match stdout {
            Some(targets) if targets.len() == 1 => {
                match &targets[0].node {
                    RedirectOutputTarget::File { path, append } => {
                        assert!(!append);
                        assert!(matches!(path, Expr { node: ExprKind::Var(v), .. } if v == "f"));
                    }
                    _ => panic!("Expected stdout: file(f)"),
                }
            }
            _ => panic!("Expected stdout: file(f)"),
        }
        assert!(matches!(
            body[0],
            Stmt {
                node: StmtKind::Print(_),
                ..
            }
        ));
    } else {
        panic!("Expected WithRedirect (stdout -> file)");
    }

    // stmt2: with redirect { stderr: file(f, append=true) } { print_err("err") }
    if let Stmt {
        node:
            StmtKind::WithRedirect {
                stdout,
                stderr,
                stdin,
                body,
            },
        ..
    } = &func.body[2]
    {
        assert!(stdin.is_none());
        assert!(stdout.is_none());
        match stderr {
            Some(targets) if targets.len() == 1 => {
                match &targets[0].node {
                    RedirectOutputTarget::File { path, append } => {
                        assert!(*append);
                        assert!(matches!(path, Expr { node: ExprKind::Var(v), .. } if v == "f"));
                    }
                    _ => panic!("Expected stderr: file(f, append=true)"),
                }
            }
            _ => panic!("Expected stderr: file(f, append=true)"),
        }
        assert!(matches!(
            body[0],
            Stmt {
                node: StmtKind::PrintErr(_),
                ..
            }
        ));
    } else {
        panic!("Expected WithRedirect (stderr append)");
    }

    // stmt3: run("cat", f)
    assert!(matches!(
        func.body[3],
        Stmt {
            node: StmtKind::Run(_),
            ..
        }
    ));

    // stmt4: with redirect { stderr: stdout } { print_err("e2s") }
    if let Stmt {
        node:
            StmtKind::WithRedirect {
                stdout,
                stderr,
                stdin,
                body,
            },
        ..
    } = &func.body[4]
    {
        assert!(stdin.is_none());
        assert!(stdout.is_none());
        match stderr {
            Some(targets) if targets.len() == 1 => {
                match &targets[0].node {
                    RedirectOutputTarget::ToStdout => {}
                    _ => panic!("Expected stderr: to_stdout()"),
                }
            }
            _ => panic!("Expected stderr: to_stdout()"),
        }
        assert!(matches!(
            body[0],
            Stmt {
                node: StmtKind::PrintErr(_),
                ..
            }
        ));
    } else {
        panic!("Expected WithRedirect (stderr -> stdout)");
    }

    // stmt5: let infile = capture("mktemp")
    assert!(matches!(
        func.body[5],
        Stmt {
            node: StmtKind::Let { .. },
            ..
        }
    ));

    // stmt6: sh("echo hello > $infile")
    assert!(matches!(
        func.body[6],
        Stmt {
            node: StmtKind::Sh(_),
            ..
        }
    ));

    // stmt7: with redirect { stdin: file(infile) } { run("cat") }
    if let Stmt {
        node:
            StmtKind::WithRedirect {
                stdout,
                stderr,
                stdin,
                body,
            },
        ..
    } = &func.body[7]
    {
        assert!(stdout.is_none());
        assert!(stderr.is_none());
        match stdin {
            Some(RedirectInputTarget::File { path }) => {
                assert!(matches!(path, Expr { node: ExprKind::Var(v), .. } if v == "infile"));
            }
            _ => panic!("Expected stdin: file(infile)"),
        }
        assert!(matches!(
            body[0],
            Stmt {
                node: StmtKind::Run(_),
                ..
            }
        ));
    } else {
        panic!("Expected WithRedirect (stdin <- file)");
    }

    // stmt8: let f2 = capture("mktemp")
    assert!(matches!(
        func.body[8],
        Stmt {
            node: StmtKind::Let { .. },
            ..
        }
    ));

    // stmt9: with redirect { stdout: stderr, stderr: file(f2) } { print("swap_ok") }
    if let Stmt {
        node:
            StmtKind::WithRedirect {
                stdout,
                stderr,
                stdin,
                body,
            },
        ..
    } = &func.body[9]
    {
        assert!(stdin.is_none());
        match stdout {
            Some(targets) if targets.len() == 1 => {
                match &targets[0].node {
                    RedirectOutputTarget::ToStderr => {}
                    _ => panic!("Expected stdout: to_stderr()"),
                }
            }
            _ => panic!("Expected stdout: to_stderr()"),
        }
        match stderr {
            Some(targets) if targets.len() == 1 => {
                match &targets[0].node {
                    RedirectOutputTarget::File { path, append } => {
                        assert!(!append);
                        assert!(matches!(path, Expr { node: ExprKind::Var(v), .. } if v == "f2"));
                    }
                    _ => panic!("Expected stderr: file(f2)"),
                }
            }
            _ => panic!("Expected stderr: file(f2)"),
        }
        assert!(matches!(
            body[0],
            Stmt {
                node: StmtKind::Print(_),
                ..
            }
        ));
    } else {
        panic!("Expected WithRedirect (ordering sensitive)");
    }

    // stmt10: run("cat", f2)
    assert!(matches!(
        func.body[10],
        Stmt {
            node: StmtKind::Run(_),
            ..
        }
    ));

    // cleanup runs exist
    assert!(matches!(
        func.body[11],
        Stmt {
            node: StmtKind::Run(_),
            ..
        }
    ));
    assert!(matches!(
        func.body[12],
        Stmt {
            node: StmtKind::Run(_),
            ..
        }
    ));
    assert!(matches!(
        func.body[13],
        Stmt {
            node: StmtKind::Run(_),
            ..
        }
    ));
}

#[test]
fn codegen_redirect_io() {
    assert_codegen_matches_snapshot("redirect_io");
}

#[test]
fn exec_redirect_io() {
    assert_exec_matches_fixture("redirect_io");
}

#[test]
fn parse_redirect_list_errors() {
    let cases = vec![
        // Empty list
        ("with redirect { stdout: [] } { print(1) }", "redirect target list cannot be empty"),
        ("with redirect { stderr: [] } { print(1) }", "redirect target list cannot be empty"),
        
        // Stdin list
        ("with redirect { stdin: [file(\"x\")] } { print(1) }", "stdin does not support multi-sink redirect"),
        
        // Inherit in non-list (parser error now)
        ("with redirect { stdout: inherit_stdout() } { print(1) }", "inherit_stdout() is only valid in redirect lists"),
        ("with redirect { stderr: inherit_stderr() } { print(1) }", "inherit_stderr() is only valid in redirect lists"),
        
        // Wrong stream inherit
        ("with redirect { stderr: [inherit_stdout()] } { print(1) }", "inherit_stdout() is only valid for stdout redirects"),
        ("with redirect { stdout: [inherit_stderr()] } { print(1) }", "inherit_stderr() is only valid for stderr redirects"),
        
        // Duplicate inherit
        ("with redirect { stdout: [inherit_stdout(), inherit_stdout()] } { print(1) }", "duplicate inherit_stdout()"),
        ("with redirect { stderr: [inherit_stderr(), inherit_stderr()] } { print(1) }", "duplicate inherit_stderr()"),
        
        // Cross-stream in list (function form)
        ("with redirect { stdout: [to_stderr(), file(\"x\")] } { print(1) }", "cross-stream redirect not allowed in multi-sink list"),
        ("with redirect { stderr: [to_stdout(), file(\"x\")] } { print(1) }", "cross-stream redirect not allowed in multi-sink list"),
        
        // Cross-stream in list (legacy keyword form)
        ("with redirect { stdout: [stdout, file(\"x\")] } { print(1) }", "cross-stream redirect not allowed in multi-sink list"),
        ("with redirect { stdout: [stderr, file(\"x\")] } { print(1) }", "cross-stream redirect not allowed in multi-sink list"),
        ("with redirect { stderr: [stdout, file(\"x\")] } { print(1) }", "cross-stream redirect not allowed in multi-sink list"),
        ("with redirect { stderr: [stderr, file(\"x\")] } { print(1) }", "cross-stream redirect not allowed in multi-sink list"),
    ];

    for (src, err_msg) in cases {
        let full_src = format!("func main() {{ {} }}", src);
        // Using try_compile_to_shell fails at parser stage for these
        let result = try_compile_to_shell(&full_src, TargetShell::Bash);
        match result {
            Ok(_) => panic!("Expected failure for '{}', but it succeeded", src),
            Err(msg) => if !msg.contains(err_msg) {
                panic!("Expected error '{}' for input '{}', but got: {}", err_msg, src, msg);
            }
        }
    }
}


#[test]
fn test_mixed_append_compile_error() {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/redirect_multi_mixed_append_error.sh2");
    
    let result = try_compile_to_shell_path(&fixture_path, TargetShell::Bash);
    match result {
        Ok(_) => panic!("Expected compilation to fail for mixed append modes"),
        Err(msg) => {
            assert!(msg.contains("redirect list cannot mix append and overwrite modes"),
                    "Error should mention mixed append restriction, got: {}", msg);
        }
    }
}

#[test]
fn test_posix_multi_sink_unsupported() {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/redirect_multi_posix_unsupported.sh2");
    
    let result = try_compile_to_shell_path(&fixture_path, TargetShell::Posix);
    match result {
        Ok(_) => panic!("Expected compilation to fail for POSIX multi-sink"),
        Err(msg) => {
            assert!(msg.contains("multi-sink redirect is not supported for POSIX target"),
                    "Error should mention POSIX limitation, got: {}", msg);
            assert!(msg.contains("switch to --target bash"),
                    "Error should suggest bash target, got: {}", msg);
        }
    }
}

#[test]
fn test_posix_single_list_ok() {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/redirect_multi_posix_single_list_ok.sh2");
    
    let result = try_compile_to_shell_path(&fixture_path, TargetShell::Posix);
    assert!(result.is_ok(), 
            "Single-element list should compile for POSIX, got error: {:?}", result.err());
}

fn try_compile_to_shell_path(path: &Path, target: TargetShell) -> Result<String, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    try_compile_to_shell(&content, target)
}
