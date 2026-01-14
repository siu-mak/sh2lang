use sh2c::ast::ExprKind;
use sh2c::ast::StmtKind;
use sh2c::ast::{Expr, RedirectTarget, Stmt};
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
        match stdout {
            Some(RedirectTarget::File { path, append }) => {
                assert!(!append);
                assert!(matches!(path, Expr { node: ExprKind::Var(v), .. } if v == "f"));
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
            Some(RedirectTarget::File { path, append }) => {
                assert!(*append);
                assert!(matches!(path, Expr { node: ExprKind::Var(v), .. } if v == "f"));
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
            Some(RedirectTarget::Stdout) => {}
            _ => panic!("Expected stderr: stdout"),
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
            Some(RedirectTarget::File { path, append }) => {
                assert!(!append);
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
            Some(RedirectTarget::Stderr) => {}
            _ => panic!("Expected stdout: stderr"),
        }
        match stderr {
            Some(RedirectTarget::File { path, append }) => {
                assert!(!append);
                assert!(matches!(path, Expr { node: ExprKind::Var(v), .. } if v == "f2"));
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
