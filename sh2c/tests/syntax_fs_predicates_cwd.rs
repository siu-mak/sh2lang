use sh2c::ast::ExprKind;
use sh2c::ast::StmtKind;
use sh2c::ast::{ArithOp, CompareOp, Expr, Stmt, RunCall};
mod common;
use common::*;

#[test]
fn parse_fs_predicates_cwd() {
    let program = parse_fixture("fs_predicates_cwd");
    let func = &program.functions[0];

    // stmt0: run("mkdir", "-p", "scratch")
    if let Stmt {
        node: StmtKind::Run(RunCall { args, .. }),
        ..
    } = &func.body[0]
    {
        assert_eq!(args.len(), 3);
        assert!(matches!(args[0], Expr { node: ExprKind::Literal(ref s), .. } if s == "mkdir"));
    } else {
        panic!("Expected run(mkdir, ...)");
    }

    // stmt1: with cwd "sh2_test_scratch_fs_predicates_cwd" { sh("touch f.txt") }
    if let Stmt {
        node: StmtKind::WithCwd { path, body },
        ..
    } = &func.body[1]
    {
        if let Expr {
            node: ExprKind::Literal(s), // Now literal
            ..
        } = path
        {
            assert_eq!(s, "sh2_test_scratch_fs_predicates_cwd");
        } else {
            panic!("Expected Literal(\"sh2_test_scratch_fs_predicates_cwd\")");
        }
        assert_eq!(body.len(), 1);
        assert!(matches!(body[0], Stmt { node: StmtKind::Sh(ref e), .. } if matches!(e.node, ExprKind::Literal(ref s) if s == "touch f.txt")));
    } else {
        panic!("Expected WithCwd");
    }

    // stmt2: let f = "sh2_test_scratch_fs_predicates_cwd/f.txt"
    if let Stmt {
        node: StmtKind::Let { name, value },
        ..
    } = &func.body[2]
    {
        assert_eq!(name, "f");
        if let Expr {
            node: ExprKind::Literal(s),
            ..
        } = value
        {
            assert_eq!(s, "sh2_test_scratch_fs_predicates_cwd/f.txt");
        } else {
            panic!("Expected Literal(\"sh2_test_scratch_fs_predicates_cwd/f.txt\")");
        }
    } else {
        panic!("Expected let f = ...");
    }

    // stmt3: if exists(f) ...
    // ... same assert as before ...
    if let Stmt {
        node: StmtKind::If { cond, .. },
        ..
    } = &func.body[3]
    {
        assert!(matches!(
            cond,
            Expr {
                node: ExprKind::And(_, _),
                ..
            }
        ));
    } else {
        panic!("Expected If for filesystem predicate check");
    }

    // stmt4: let d = capture("mktemp", "-d")
    if let Stmt {
        node: StmtKind::Let { name, value },
        ..
    } = &func.body[4]
    {
        assert_eq!(name, "d");
        assert!(matches!(value.node, ExprKind::Command(_)));
    } else {
        panic!("Expected let d = capture(...)");
    }

    // stmt5: cd(d)
    assert!(matches!(
        func.body[5],
        Stmt {
            node: StmtKind::Cd { .. },
            ..
        }
    ));

    // stmt6: if pwd() == d ...
    // ...
    if let Stmt {
        node: StmtKind::If { cond, .. },
        ..
    } = &func.body[6]
    {
        assert!(matches!(cond.node, ExprKind::Compare { .. }));
    } else {
        panic!("Expected If for pwd check");
    }

    // cleanup stmts follow
    // Expect: run("rm", ... d)
    // Expect: run("rm", ... "sh2_test_scratch_fs_predicates_cwd")
    assert!(func.body.len() >= 9); 
    
    // Check last stmt is removing the known scratch dir
    let last = func.body.last().unwrap();
    if let Stmt { node: StmtKind::Run(RunCall { args, .. }), .. } = last {
        // args: rm, -rf, LITERAL
        assert!(args.len() >= 3);
        // We can just check it contains the literal
        let has_scratch = args.iter().any(|a| matches!(&a.node, ExprKind::Literal(s) if s == "sh2_test_scratch_fs_predicates_cwd"));
        assert!(has_scratch, "Cleanup should remove scratch dir");
    } else {
        panic!("Expected cleanup Run call at end");
    }
}

#[test]
fn codegen_fs_predicates_cwd() {
    assert_codegen_matches_snapshot("fs_predicates_cwd");
}

#[test]
fn exec_fs_predicates_cwd() {
    assert_exec_matches_fixture("fs_predicates_cwd");
}
