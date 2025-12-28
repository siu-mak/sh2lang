use sh2c::ast::{Stmt, StmtKind, Expr, ExprKind};
mod common;
use common::*;

#[test]
fn parse_capture_pipe() {
    let program = parse_fixture("capture_pipe");
    let func = &program.functions[0];

    // stmt0: let out = capture(run(...) | run(...)) => Expr { kind: ExprKind::CommandPipe([...]), .. }
    if let Stmt { kind: StmtKind::Let { name, value }, .. } = &func.body[0] {
        assert_eq!(name, "out");
        if let Expr { kind: ExprKind::CommandPipe(segments), .. } = value {
            assert_eq!(segments.len(), 2);

            // seg0
            if let Expr { kind: ExprKind::Literal(ref s), .. } = segments[0][0] {
                 assert_eq!(s, "printf");
            } else { panic!("Expected printf"); }

            // seg1
            if let Expr { kind: ExprKind::Literal(ref s), .. } = segments[1][0] {
                 assert_eq!(s, "sed");
            } else { panic!("Expected sed"); }
        } else {
            panic!("Expected Expr::CommandPipe for capture(... | ...)");
        }
    } else {
        panic!("Expected let out = capture(...)");
    }

    // stmt1: print(out)
    assert!(matches!(func.body[1], Stmt { kind: StmtKind::Print(Expr { kind: ExprKind::Var(ref v), .. }), .. } if v == "out"));
}

#[test]
fn codegen_capture_pipe() {
    assert_codegen_matches_snapshot("capture_pipe");
}

#[test]
fn exec_capture_pipe() {
    assert_exec_matches_fixture("capture_pipe");
}