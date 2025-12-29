use sh2c::ast::{Expr, ExprKind, Stmt, StmtKind};
mod common;
use common::*;

#[test]
fn parse_env_export_unset_source() {
    let program = parse_fixture("env_export_unset_source");
    let func = &program.functions[0];

    // stmt0: export("X", "hello")
    if let Stmt {
        kind: StmtKind::Export { name, value },
        ..
    } = &func.body[0]
    {
        assert_eq!(name, "X");
        let v = value.as_ref().expect("Expected export to have a value");
        assert!(matches!(v, Expr { kind: ExprKind::Literal(s), .. } if s == "hello"));
    } else {
        panic!("Expected Export with value");
    }

    // stmt1: run("sh","-c","echo \\$X")
    assert!(matches!(
        func.body[1],
        Stmt {
            kind: StmtKind::Run(_),
            ..
        }
    ));

    // stmt2: let y = "yo"
    if let Stmt {
        kind: StmtKind::Let { name, value },
        ..
    } = &func.body[2]
    {
        assert_eq!(name, "y");
        assert!(matches!(value, Expr { kind: ExprKind::Literal(s), .. } if s == "yo"));
    } else {
        panic!("Expected let y = \"yo\"");
    }

    // stmt3: export("y") (no value)
    if let Stmt {
        kind: StmtKind::Export { name, value },
        ..
    } = &func.body[3]
    {
        assert_eq!(name, "y");
        assert!(value.is_none());
    } else {
        panic!("Expected Export without value");
    }

    // stmt4: run("sh","-c","echo \\$y")
    assert!(matches!(
        func.body[4],
        Stmt {
            kind: StmtKind::Run(_),
            ..
        }
    ));

    // stmt5: unset("X")
    if let Stmt {
        kind: StmtKind::Unset { name },
        ..
    } = &func.body[5]
    {
        assert_eq!(name, "X");
    } else {
        panic!("Expected Unset(\"X\")");
    }

    // stmt6: run("sh","-c", ...)
    assert!(matches!(
        func.body[6],
        Stmt {
            kind: StmtKind::Run(_),
            ..
        }
    ));

    // stmt7: let f = capture("mktemp") => Expr { kind: ExprKind::Command([...]), .. }
    if let Stmt {
        kind: StmtKind::Let { name, value },
        ..
    } = &func.body[7]
    {
        assert_eq!(name, "f");
        assert!(matches!(
            value,
            Expr {
                kind: ExprKind::Command(_),
                ..
            }
        ));
    } else {
        panic!("Expected let f = capture(...)");
    }

    // stmt8: sh("echo 'echo sourced_ok' > $f")
    assert!(matches!(
        func.body[8],
        Stmt {
            kind: StmtKind::Sh(_),
            ..
        }
    ));

    // stmt9: source(f)
    if let Stmt {
        kind: StmtKind::Source { path },
        ..
    } = &func.body[9]
    {
        assert!(matches!(path, Expr { kind: ExprKind::Var(v), .. } if v == "f"));
    } else {
        panic!("Expected Source(f)");
    }

    // stmt10: run("rm", "-f", f)
    assert!(matches!(
        func.body[10],
        Stmt {
            kind: StmtKind::Run(_),
            ..
        }
    ));
}

#[test]
fn codegen_env_export_unset_source() {
    assert_codegen_matches_snapshot("env_export_unset_source");
}

#[test]
fn exec_env_export_unset_source() {
    assert_exec_matches_fixture("env_export_unset_source");
}
