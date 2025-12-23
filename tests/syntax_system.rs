mod common;
use sh2c::ast::{Stmt, Expr};
use common::{parse_fixture, assert_codegen_matches_snapshot, assert_exec_matches_fixture};

#[test]
fn parse_exit_basic() {
    let program = parse_fixture("exit_basic");
    let func = &program.functions[0];
    assert!(matches!(func.body[1], Stmt::Exit(_)));
}

#[test]
fn parse_with_env() {
    let program = parse_fixture("with_env");
    let func = &program.functions[0];
    assert!(matches!(func.body[0], Stmt::WithEnv { .. }));
}

#[test]
fn parse_cd_basic() {
    let program = parse_fixture("cd_basic");
    let func = &program.functions[0];
    assert!(matches!(func.body[0], Stmt::Cd { .. }));
}

#[test]
fn parse_spawn_run() {
    let program = parse_fixture("spawn_run");
    let func = &program.functions[0];
    assert!(matches!(func.body[0], Stmt::Spawn { .. }));
}

#[test]
fn parse_wait_all() {
    let program = parse_fixture("wait_all");
    let func = &program.functions[0];
    assert!(matches!(func.body[2], Stmt::Wait(_)));
}

#[test]
fn parse_export_unset() {
    let program = parse_fixture("export_unset");
    let func = &program.functions[0];
    assert!(matches!(func.body[0], Stmt::Export { .. }));
    assert!(matches!(func.body[2], Stmt::Unset { .. }));
}

#[test]
fn parse_source_basic() {
    let program = parse_fixture("source_basic");
    let func = &program.functions[0];
    assert!(matches!(func.body[1], Stmt::Source { .. }));
}

#[test]
fn parse_exists_check() {
    let program = parse_fixture("exists_check");
    let func = &program.functions[0];
    if let Stmt::If { cond, .. } = &func.body[0] {
        assert!(matches!(cond, Expr::Exists(..)));
    } else { panic!("Expected If(Exists)"); }
}

#[test]
fn codegen_exit_basic() { assert_codegen_matches_snapshot("exit_basic"); }
#[test]
fn codegen_exit_arg() { assert_codegen_matches_snapshot("exit_arg"); }
#[test]
fn codegen_export_unset() { assert_codegen_matches_snapshot("export_unset"); }
#[test]
fn codegen_source_basic() { assert_codegen_matches_snapshot("source_basic"); }
#[test]
fn codegen_spawn_run() { assert_codegen_matches_snapshot("spawn_run"); }
#[test]
fn codegen_wait_all() { assert_codegen_matches_snapshot("wait_all"); }
#[test]
fn codegen_wait_pid_var() { assert_codegen_matches_snapshot("wait_pid_var"); }
#[test]
fn codegen_with_env() { assert_codegen_matches_snapshot("with_env"); }
#[test]
fn codegen_with_cwd() { assert_codegen_matches_snapshot("with_cwd"); }
#[test]
fn codegen_with_cwd_check() { assert_codegen_matches_snapshot("with_cwd_check"); }
#[test]
fn codegen_cd_basic() { assert_codegen_matches_snapshot("cd_basic"); }
#[test]
fn codegen_exists_check() { assert_codegen_matches_snapshot("exists_check"); }

#[test]
fn exec_exists_check() { assert_exec_matches_fixture("exists_check"); }
#[test]
fn exec_with_cwd_check() { assert_exec_matches_fixture("with_cwd_check"); }

#[test]
fn codegen_sh_raw() { assert_codegen_matches_snapshot("sh_raw"); }

#[test]
fn codegen_sh_block() { assert_codegen_matches_snapshot("sh_block"); }

// --- Exit Status Backfill ---

#[test]
fn parse_exit_status() {
    let program = parse_fixture("exit_status");
    let func = &program.functions[0];
    if let Stmt::Exit(Some(val)) = &func.body[0] {
        if let Expr::Number(n) = val {
            assert_eq!(*n, 7);
        } else { panic!("Expected Exit(Number)"); }
    } else { panic!("Expected Exit(Some)"); }
}

#[test]
fn codegen_exit_status() { assert_codegen_matches_snapshot("exit_status"); }

#[test]
fn exec_exit_status() { assert_exec_matches_fixture("exit_status"); }

// --- Backfill: Process & Grouping ---

#[test]
fn codegen_subshell_basic() { assert_codegen_matches_snapshot("subshell_basic"); }
#[test]
fn codegen_group_basic() { assert_codegen_matches_snapshot("group_basic"); }
#[test]
fn codegen_spawn_group() { assert_codegen_matches_snapshot("spawn_group"); }
#[test]
fn codegen_spawn_sh_block() { assert_codegen_matches_snapshot("spawn_sh_block"); }

// --- Backfill: Wait Variants ---

#[test]
fn codegen_wait_no_arg() { assert_codegen_matches_snapshot("wait_no_arg"); }
#[test]
fn codegen_wait_complex() { assert_codegen_matches_snapshot("wait_complex"); }
#[test]
fn codegen_wait_arg() { assert_codegen_matches_snapshot("wait_arg"); }
#[test]
fn codegen_wait_list_literal() { assert_codegen_matches_snapshot("wait_list_literal"); }
#[test]
fn codegen_wait_list_var() { assert_codegen_matches_snapshot("wait_list_var"); }
#[test]
fn codegen_wait_list_complex() { assert_codegen_matches_snapshot("wait_list_complex"); }
#[test]
fn codegen_wait_pid_literal() { assert_codegen_matches_snapshot("wait_pid_literal"); }
#[test]
fn exec_wait_pid_var() { assert_exec_matches_fixture("wait_pid_var"); }

// --- Parse Coverage for Process ---

#[test]
fn parse_subshell_basic() {
    let program = parse_fixture("subshell_basic");
    let func = &program.functions[0];
    assert!(matches!(func.body[1], Stmt::Subshell { .. }));
}

#[test]
fn parse_wait_complex() {
    let program = parse_fixture("wait_complex");
    let func = &program.functions[0];
    // wait($(run...))
    if let Stmt::Wait(Some(Expr::Command(_))) = &func.body[0] {
        // ok
    } else { panic!("Expected Wait(Command)"); }
}

#[test]
fn exec_subshell_basic() { assert_exec_matches_fixture("subshell_basic"); }
#[test]
fn exec_group_basic() { assert_exec_matches_fixture("group_basic"); }
