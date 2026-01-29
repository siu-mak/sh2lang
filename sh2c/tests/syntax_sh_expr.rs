mod common;

#[test]
fn test_sh_expr_compiles() {
    let fixture_path = common::repo_root().join("sh2c/tests/fixtures/check_sh_is_expr.sh2");
    let src = std::fs::read_to_string(fixture_path).unwrap();
    let res = common::try_compile_to_shell(&src, common::TargetShell::Bash);
    assert!(res.is_ok(), "sh() as expr failed to compile: {:?}", res.err());
}
