mod common;
use common::*;
use sh2c::ast::{Stmt, Expr};

#[test]
fn parse_fs_readable_writable_basic() {
    let program = parse_fixture("fs_readable_writable_basic");
    let func = &program.functions[0];
    assert_eq!(func.body.len(), 2);

    assert!(matches!(&func.body[0], Stmt::Run(_)));

    if let Stmt::If { cond, .. } = &func.body[1] {
        fn has_rw(e: &Expr) -> (bool,bool) {
            match e {
                Expr::IsReadable(_) => (true,false),
                Expr::IsWritable(_) => (false,true),
                Expr::And(a,b) | Expr::Or(a,b) => {
                    let (a1,a2) = has_rw(a);
                    let (b1,b2) = has_rw(b);
                    (a1||b1, a2||b2)
                }
                Expr::Not(x) => has_rw(x),
                _ => (false,false),
            }
        }
        let (r,w) = has_rw(cond);
        assert!(r && w);
    } else {
        panic!("Expected If");
    }
}

#[test]
fn codegen_fs_readable_writable_basic() {
    assert_codegen_matches_snapshot("fs_readable_writable_basic");
}

#[test]
fn exec_fs_readable_writable_basic() {
    assert_exec_matches_fixture("fs_readable_writable_basic");
}
