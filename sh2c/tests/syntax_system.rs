mod common;
use common::*;
use sh2c::ast::{Expr, ExprKind, Stmt, StmtKind};

#[test]
fn exec_wait_list_literal() {
    // Manually run to ignore path-dependent stderr (Bash wait warning includes script path)
    let fixture_name = "wait_list_literal";
    let sh2_path = format!("tests/fixtures/{}.sh2", fixture_name);
    let stdout_path = format!("tests/fixtures/{}.stdout", fixture_name);
    let status_path = format!("tests/fixtures/{}.status", fixture_name);

    if !std::path::Path::new(&sh2_path).exists() {
        panic!("Fixture {} does not exist", sh2_path);
    }
    
    let src = std::fs::read_to_string(&sh2_path).unwrap();
    let expected_stdout = std::fs::read_to_string(&stdout_path)
        .unwrap_or_default()
        .trim()
        .replace("\r\n", "\n");
    let expected_status: i32 = std::fs::read_to_string(&status_path)
        .expect("Missing status file")
        .trim()
        .parse()
        .unwrap();

    for target in [TargetShell::Bash, TargetShell::Posix] {
        let shell_script = compile_to_shell(&src, target);
        let shell_bin = match target {
            TargetShell::Bash => "bash",
            TargetShell::Posix => {
                 if std::process::Command::new("dash").arg("-c").arg("true").status().map(|s| s.success()).unwrap_or(false) {
                    "dash"
                } else {
                    "sh"
                }
            }
        };

        let (stdout, _stderr, status) = run_shell_script(
            &shell_script,
            shell_bin,
            &[],
            &[],
            None,
            None,
        );

        assert_eq!(stdout.trim(), expected_stdout, "Stdout mismatch for {:?} target", target);
        assert_eq!(status, Some(expected_status), "Status mismatch for {:?} target", target);
    }
}

// --- Parse Coverage for Process ---

#[test]
fn parse_subshell_basic() {
    let program = parse_fixture("subshell_basic");
    let func = &program.functions[0];
    assert!(matches!(
        func.body[1],
        Stmt {
            node: StmtKind::Subshell { .. },
            ..
        }
    ));
}

#[test]
fn parse_wait_complex() {
    let program = parse_fixture("wait_complex");
    let func = &program.functions[0];
    // wait($(run...))
    if let Stmt {
        node:
            StmtKind::Wait(Some(Expr {
                node: ExprKind::Command(_),
                ..
            })),
        ..
    } = &func.body[0]
    {
        // ok
    } else {
        panic!("Expected Wait(Command)");
    }
}

#[test]
fn exec_subshell_basic() {
    assert_exec_matches_fixture("subshell_basic");
}
#[test]
fn exec_group_basic() {
    assert_exec_matches_fixture("group_basic");
}
