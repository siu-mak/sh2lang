use sh2c::ast::StmtKind;
use sh2c::ast::ExprKind;
use sh2c::ast::{Stmt, Expr, ArithOp, CompareOp};
mod common;
use common::*;

#[test]
fn parse_fs_predicates_cwd() {
    let program = parse_fixture("fs_predicates_cwd");
    let func = &program.functions[0];

    // stmt0: let d = capture("mktemp","-d")  => Expr { kind: ExprKind::Command([...]), .. }
    if let Stmt { kind: StmtKind::Let { name, value }, .. } = &func.body[0] {
        assert_eq!(name, "d");
        if let Expr { kind: ExprKind::Command(args), .. } = value {
            assert_eq!(args.len(), 2);
            assert!(matches!(args[0], Expr { kind: ExprKind::Literal(ref s), .. } if s == "mktemp"));
            assert!(matches!(args[1], Expr { kind: ExprKind::Literal(ref s), .. } if s == "-d"));
        } else {
            panic!("Expected Expr::Command for capture(\"mktemp\", \"-d\")");
        }
    } else {
        panic!("Expected let d = capture(...)");
    }

    // stmt1: with cwd d { sh("touch f.txt") }
    if let Stmt { kind: StmtKind::WithCwd { path, body }, .. } = &func.body[1] {
        if let Expr { kind: ExprKind::Var(v), .. } = path {
            assert_eq!(v, "d");
        } else { panic!("Expected Var(d)"); }
        assert_eq!(body.len(), 1);
        assert!(matches!(body[0], Stmt { kind: StmtKind::Sh(ref s), .. } if s == "touch f.txt"));
    } else {
        panic!("Expected WithCwd");
    }

    // stmt2: let f = d + "/f.txt"  (parses as Arith(Add); lower turns into Concat)
    if let Stmt { kind: StmtKind::Let { name, value }, .. } = &func.body[2] {
        assert_eq!(name, "f");
        if let Expr { kind: ExprKind::Arith { left, op, right }, .. } = value {
            if let Expr { kind: ExprKind::Var(v), .. } = &**left {
                assert_eq!(v, "d");
            } else { panic!("Expected Var(d)"); }
            assert_eq!(*op, ArithOp::Add);
            if let Expr { kind: ExprKind::Literal(s), .. } = &**right {
                assert_eq!(s, "/f.txt");
            } else { panic!("Expected Literal(/f.txt)"); }
        } else {
            panic!("Expected Expr::Arith(Add) for d + \"/f.txt\"");
        }
    } else {
        panic!("Expected let f = ...");
    }

    // stmt3: if exists(f) && is_file(f) && !is_dir(f) { ... } else { ... }
    if let Stmt { kind: StmtKind::If { cond, .. }, .. } = &func.body[3] {
        // We expect some nesting of Expr::And / Expr::Not
        assert!(matches!(cond, Expr { kind: ExprKind::And(_, _), .. }));
    } else {
        panic!("Expected If for filesystem predicate check");
    }

    // stmt4: cd(d)
    assert!(matches!(func.body[4], Stmt { kind: StmtKind::Cd { .. }, .. }));

    // stmt5: if pwd() == d { ... }
    if let Stmt { kind: StmtKind::If { cond, .. }, .. } = &func.body[5] {
        if let Expr { kind: ExprKind::Compare { left, op, right }, .. } = cond {
            assert!(matches!(**left, Expr { kind: ExprKind::Pwd, .. }));
            assert_eq!(*op, CompareOp::Eq);
            if let Expr { kind: ExprKind::Var(v), .. } = &**right {
                assert_eq!(v, "d");
            } else { panic!("Expected Var(d)"); }
        } else {
            panic!("Expected Compare in pwd() == d");
        }
    } else {
        panic!("Expected If for cd/pwd check");
    }

    // stmt6: run("rm","-rf", d)
    assert!(matches!(func.body[6], Stmt { kind: StmtKind::Run(_), .. }));
}

#[test]
fn codegen_fs_predicates_cwd() {
    assert_codegen_matches_snapshot("fs_predicates_cwd");
}

#[test]
fn exec_fs_predicates_cwd() {
    assert_exec_matches_fixture("fs_predicates_cwd");
}