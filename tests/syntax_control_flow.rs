mod common;
use sh2c::ast::{Stmt, Expr};
use common::{parse_fixture, assert_codegen_matches_snapshot, assert_exec_matches_fixture};

// --- Parsers ---

#[test]
fn parse_if_bool_and() {
    let program = parse_fixture("if_bool_and");
    let func = &program.functions[0];
    if let Stmt::If { cond, .. } = &func.body[0] {
        assert!(matches!(cond, Expr::And(..)));
    } else {
        panic!("Expected If");
    }
}

#[test]
fn parse_if_bool_literals() {
    let program = parse_fixture("if_true_literal");
    let func = &program.functions[0];
    if let Stmt::If { cond, .. } = &func.body[0] {
        assert!(matches!(cond, Expr::Bool(true)));
    } else {
        panic!("Expected If(Bool(true))");
    }
}

#[test]
fn parse_while_basic() {
    let program = parse_fixture("while_basic");
    let func = &program.functions[0];
    assert!(matches!(func.body[1], Stmt::While { .. }));
}

#[test]
fn parse_for_list_var() {
    let program = parse_fixture("for_list_var");
    let func = &program.functions[0];
    // Check main structure roughly to ensure it parsed For
    assert_eq!(func.name, "main");
    match &func.body[1] {
        Stmt::For { var, .. } => assert_eq!(var, "x"),
        _ => panic!("Expected For stmt"),
    }
}

// --- Codegen ---

#[test]
fn codegen_if_basic() { assert_codegen_matches_snapshot("if_basic"); }
#[test]
fn codegen_if_elif() { assert_codegen_matches_snapshot("if_elif"); }
#[test]
fn codegen_if_else_if() { assert_codegen_matches_snapshot("if_else_if"); }
#[test]
fn codegen_if_bool_and() { assert_codegen_matches_snapshot("if_bool_and"); }
#[test]
fn codegen_if_true_literal() { assert_codegen_matches_snapshot("if_true_literal"); }
#[test]
fn codegen_if_false_literal() { assert_codegen_matches_snapshot("if_false_literal"); }
#[test]
fn codegen_if_bool_literals_combo() { assert_codegen_matches_snapshot("if_bool_literals_combo"); }
#[test]
fn codegen_while_basic() { assert_codegen_matches_snapshot("while_basic"); }
#[test]
fn codegen_for_basic() { assert_codegen_matches_snapshot("for_basic"); }
#[test]
fn codegen_for_list() { assert_codegen_matches_snapshot("for_list"); }
#[test]
fn codegen_for_list_var() { assert_codegen_matches_snapshot("for_list_var"); }
#[test]
fn codegen_for_args() { assert_codegen_matches_snapshot("for_args"); }

// --- Exec ---

#[test]
fn exec_if_env_true() { assert_exec_matches_fixture("if_env_true"); }
#[test]
fn exec_if_env_false() { assert_exec_matches_fixture("if_env_false"); }
#[test]
fn exec_if_true_literal() { assert_exec_matches_fixture("if_true_literal"); }
#[test]
fn exec_if_bool_and() { assert_exec_matches_fixture("if_bool_and"); }
#[test]
fn exec_while_basic() { assert_exec_matches_fixture("while_basic"); }
#[test]
fn exec_for_list() { assert_exec_matches_fixture("for_list"); }
#[test]
fn exec_for_list_var() { assert_exec_matches_fixture("for_list_var"); }
#[test]
fn exec_for_args() { assert_exec_matches_fixture("for_args"); }

#[test]
fn codegen_if_paren_cond() { assert_codegen_matches_snapshot("if_paren_cond"); }

#[test]
fn codegen_if_exists() { assert_codegen_matches_snapshot("if_exists"); }
#[test]
fn codegen_if_is_dir() { assert_codegen_matches_snapshot("if_is_dir"); }
#[test]
fn codegen_if_is_file() { assert_codegen_matches_snapshot("if_is_file"); }
#[test]
fn codegen_if_number_compare() { assert_codegen_matches_snapshot("if_number_compare"); }

#[test]
fn parse_if_statement_inline() {
    let src = r#"
        func main() {
            if registry {
                print("configured")
            }
        }
    "#;
    use sh2c::{lexer, parser};
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    match &ast.functions[0].body[0] {
        sh2c::ast::Stmt::If { cond, then_body, else_body, .. } => {
            if let sh2c::ast::Expr::Var(name) = cond {
                assert_eq!(name, "registry");
            } else { panic!("Expected Var"); }
            assert_eq!(then_body.len(), 1);
            assert!(else_body.is_none());
        }
        _ => panic!("Expected if"),
    }
}

#[test]
fn parse_nested_if_inline() {
    let src = r#"
        func main() {
            if a {
                if b {
                    print("x")
                }
            }
        }
    "#;
    use sh2c::{lexer, parser};
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    match &ast.functions[0].body[0] {
        sh2c::ast::Stmt::If { then_body, .. } => {
            assert!(matches!(then_body[0], sh2c::ast::Stmt::If { .. }));
        }
        _ => panic!("Expected outer if"),
    }
}

// --- Break / Continue / Short-circuit Backfill ---

#[test]
fn parse_break_basic() {
    let program = parse_fixture("break_basic");
    let func = &program.functions[0];
    // func main -> For -> If -> Break
    if let Stmt::For { body, .. } = &func.body[1] {
        if let Stmt::If { then_body, .. } = &body[0] {
             assert!(matches!(then_body[0], Stmt::Break));
        } else { panic!("Expected If in loop"); }
    } else { panic!("Expected For loop"); }
}

#[test]
fn parse_continue_basic() {
    let program = parse_fixture("continue_basic");
    let func = &program.functions[0];
    if let Stmt::For { body, .. } = &func.body[1] {
        if let Stmt::If { then_body, .. } = &body[0] {
             assert!(matches!(then_body[0], Stmt::Continue));
        } else { panic!("Expected If in loop"); }
    } else { panic!("Expected For loop"); }
}

#[test]
fn parse_andthen_short_circuit() {
    let program = parse_fixture("andthen_short_circuit");
    let func = &program.functions[0];
    assert!(matches!(func.body[0], Stmt::AndThen { .. }));
}

#[test]
fn parse_orelse_short_circuit() {
    let program = parse_fixture("orelse_short_circuit");
    let func = &program.functions[0];
    assert!(matches!(func.body[0], Stmt::OrElse { .. }));
}

#[test]
fn codegen_break_basic() { assert_codegen_matches_snapshot("break_basic"); }
#[test]
fn codegen_continue_basic() { assert_codegen_matches_snapshot("continue_basic"); }
#[test]
fn codegen_andthen_short_circuit() { assert_codegen_matches_snapshot("andthen_short_circuit"); }
#[test]
fn codegen_orelse_short_circuit() { assert_codegen_matches_snapshot("orelse_short_circuit"); }

#[test]
fn exec_break_basic() { assert_exec_matches_fixture("break_basic"); }
#[test]
fn exec_continue_basic() { assert_exec_matches_fixture("continue_basic"); }
#[test]
fn exec_andthen_short_circuit() { assert_exec_matches_fixture("andthen_short_circuit"); }
#[test]
fn exec_orelse_short_circuit() { assert_exec_matches_fixture("orelse_short_circuit"); }

// --- Backfill: And/Or Chain ---

#[test]
fn codegen_and_or_chain() { assert_codegen_matches_snapshot("and_or_chain"); }

#[test]
fn codegen_if_bool() { assert_codegen_matches_snapshot("if_bool"); }
#[test]
fn codegen_if_else() { assert_codegen_matches_snapshot("if_else"); }
#[test]
fn codegen_compare() { assert_codegen_matches_snapshot("compare"); }

#[test]
fn parse_if_bool_or() {
    let program = parse_fixture("if_bool_or");
    let func = &program.functions[0];
    if let Stmt::If { cond, .. } = &func.body[0] {
        assert!(matches!(cond, Expr::Or(..)));
    } else {
        panic!("Expected If");
    }
}
#[test]
fn codegen_if_bool_or() { assert_codegen_matches_snapshot("if_bool_or"); }
#[test]
fn exec_if_bool_or() { assert_exec_matches_fixture("if_bool_or"); }

#[test]
fn parse_if_bool_precedence_and_over_or() {
    let program = parse_fixture("if_bool_precedence_and_over_or");
    let func = &program.functions[0];
    if let Stmt::If { cond, .. } = &func.body[0] {
        // A || B && C -> Or(A, And(B, C))
        if let Expr::Or(left, right) = cond {
             assert!(matches!(**left, Expr::Compare{..}));
             assert!(matches!(**right, Expr::And(..)));
        } else { panic!("Expected Or(Compare, And)"); }
    } else { panic!("Expected If"); }
}
#[test]
fn codegen_if_bool_precedence_and_over_or() { assert_codegen_matches_snapshot("if_bool_precedence_and_over_or"); }
#[test]
fn exec_if_bool_precedence_and_over_or() { assert_exec_matches_fixture("if_bool_precedence_and_over_or"); }

#[test]
fn parse_if_bool_paren_overrides_precedence() {
    let program = parse_fixture("if_bool_paren_overrides_precedence");
    let func = &program.functions[0];
    if let Stmt::If { cond, .. } = &func.body[0] {
        // (A || B) && C -> And(Or(A, B), C)
        if let Expr::And(left, right) = cond {
             assert!(matches!(**left, Expr::Or(..)));
             assert!(matches!(**right, Expr::Compare{..}));
        } else { panic!("Expected And(Or, Compare)"); }
    } else { panic!("Expected If"); }
}
#[test]
fn codegen_if_bool_paren_overrides_precedence() { assert_codegen_matches_snapshot("if_bool_paren_overrides_precedence"); }
#[test]
fn exec_if_bool_paren_overrides_precedence() { assert_exec_matches_fixture("if_bool_paren_overrides_precedence"); }

#[test]
fn parse_if_bool_not_basic() {
    let program = parse_fixture("if_bool_not_basic");
    if let Stmt::If { cond, .. } = &program.functions[0].body[0] {
        assert!(matches!(cond, Expr::Not(..)));
    } else { panic!("Expected If"); }
}
#[test]
fn codegen_if_bool_not_basic() { assert_codegen_matches_snapshot("if_bool_not_basic"); }
#[test]
fn exec_if_bool_not_basic() { assert_exec_matches_fixture("if_bool_not_basic"); }

#[test]
fn parse_arith_comparison() {
    let program = parse_fixture("arith_comparison");
    let func = &program.functions[0];
    if let Stmt::If { cond, .. } = &func.body[0] {
        // 5 > 3
        if let Expr::Compare { left, op, right } = cond {
            assert_eq!(*op, sh2c::ast::CompareOp::Gt);
            assert!(matches!(**left, Expr::Number(5)));
            assert!(matches!(**right, Expr::Number(3)));
        } else { panic!("Expected Compare Gt"); }
    } else { panic!("Expected If"); }
}
#[test]
fn codegen_arith_comparison() { assert_codegen_matches_snapshot("arith_comparison"); }
#[test]
fn exec_arith_comparison() { assert_exec_matches_fixture("arith_comparison"); }
