mod common;
use common::*;

// ── Compile-time error tests ─────────────────────────────────────────

#[test]
fn find0_compile_fail_expr_context() {
    // find0() must not be usable in expression context
    let src = r#"
func main() {
    let x = find0()
}
"#;
    let res = try_compile_to_shell(src, TargetShell::Bash);
    match res {
        Err(msg) => assert!(
            msg.contains("can only be used as the iterable in a for-loop"),
            "Unexpected error: {}", msg
        ),
        Ok(_) => panic!("Expected compilation failure for find0() in expression context"),
    }
}

#[test]
fn find0_compile_fail_unknown_option() {
    let src = r#"
func main() {
    for f in find0(foo="bar") {
        print(f)
    }
}
"#;
    let res = try_compile_to_shell(src, TargetShell::Bash);
    match res {
        Err(msg) => assert!(
            msg.contains("Unknown argument 'foo'"),
            "Unexpected error: {}", msg
        ),
        Ok(_) => panic!("Expected compilation failure for find0(foo=...)"),
    }
}

#[test]
fn find0_compile_fail_bad_type() {
    let src = r#"
func main() {
    for f in find0(type="x") {
        print(f)
    }
}
"#;
    let res = try_compile_to_shell(src, TargetShell::Bash);
    match res {
        Err(msg) => assert!(
            msg.contains("type must be literal"),
            "Unexpected error: {}", msg
        ),
        Ok(_) => panic!("Expected compilation failure for find0(type=\"x\")"),
    }
}

#[test]
fn find0_compile_fail_posix_target() {
    let src = r#"
func main() {
    for f in find0() {
        print(f)
    }
}
"#;
    let res = try_compile_to_shell(src, TargetShell::Posix);
    match res {
        Err(msg) => assert!(
            msg.contains("find0() requires Bash target"),
            "Unexpected error: {}", msg
        ),
        Ok(_) => panic!("Expected compilation failure for find0() on POSIX target"),
    }
}

#[test]
fn find0_compile_fail_positional_args() {
    let src = r#"
func main() {
    for f in find0("some_dir") {
        print(f)
    }
}
"#;
    let res = try_compile_to_shell(src, TargetShell::Bash);
    match res {
        Err(msg) => assert!(
            msg.contains("does not accept positional arguments"),
            "Unexpected error: {}", msg
        ),
        Ok(_) => panic!("Expected compilation failure for find0 with positional args"),
    }
}

#[test]
fn find0_compile_fail_negative_maxdepth() {
    let src = r#"
func main() {
    for f in find0(maxdepth=(-1)) {
        print(f)
    }
}
"#;
    let res = try_compile_to_shell(src, TargetShell::Bash);
    match res {
        Err(msg) => assert!(
            msg.contains("maxdepth must be a non-negative integer literal"),
            "Unexpected error: {}", msg
        ),
        Ok(_) => panic!("Expected compilation failure for find0(maxdepth=-1)"),
    }
}

#[test]
fn find0_compile_fail_negative_literal_no_parens() {
    let src = r#"
func main() {
    for f in find0(maxdepth=-5) {
        print(f)
    }
}
"#;
    let res = try_compile_to_shell(src, TargetShell::Bash);
    match res {
        Err(msg) => assert!(
            msg.contains("maxdepth must be a non-negative integer literal"),
            "Unexpected error: {}", msg
        ),
        Ok(_) => panic!("Expected compilation failure for find0(maxdepth=-5)"),
    }
}

#[test]
fn find0_compile_success_parens_literal() {
    let src = r#"
func main() {
    for f in find0(maxdepth=(1)) { 
        print(f)
    }
}
"#;
    let res = try_compile_to_shell(src, TargetShell::Bash);
    assert!(res.is_ok(), "Expected success for find0(maxdepth=(1))");
}

// ── Runtime execution tests (Bash-only) ──────────────────────────────

#[test]
fn find0_basic() {
    let src = r#"
func main() {
    run("mkdir", "-p", "test_find0_basic/sub")
    run("touch", "test_find0_basic/a.txt", "test_find0_basic/b.txt", "test_find0_basic/sub/c.txt")
    for f in find0(dir="test_find0_basic", type="f") {
        print(f)
    }
    run("rm", "-rf", "test_find0_basic")
}
"#;
    run_test_bash_only(
        "find0_basic",
        src,
        "test_find0_basic/a.txt\ntest_find0_basic/b.txt\ntest_find0_basic/sub/c.txt",
    );
}

#[test]
fn find0_name_filter() {
    let src = r#"
func main() {
    run("mkdir", "-p", "test_find0_name")
    run("touch", "test_find0_name/a.txt", "test_find0_name/b.log", "test_find0_name/c.txt")
    for f in find0(dir="test_find0_name", name="*.txt") {
        print(f)
    }
    run("rm", "-rf", "test_find0_name")
}
"#;
    run_test_bash_only(
        "find0_name_filter",
        src,
        "test_find0_name/a.txt\ntest_find0_name/c.txt",
    );
}

#[test]
fn find0_type_dir() {
    let src = r#"
func main() {
    run("mkdir", "-p", "test_find0_type/sub1", "test_find0_type/sub2")
    run("touch", "test_find0_type/file.txt")
    for d in find0(dir="test_find0_type", type="d") {
        print(d)
    }
    run("rm", "-rf", "test_find0_type")
}
"#;
    run_test_bash_only(
        "find0_type_dir",
        src,
        "test_find0_type/sub1\ntest_find0_type/sub2",
    );
}

#[test]
fn find0_maxdepth() {
    let src = r#"
func main() {
    run("mkdir", "-p", "test_find0_depth/a/b")
    run("touch", "test_find0_depth/top.txt", "test_find0_depth/a/mid.txt", "test_find0_depth/a/b/deep.txt")
    for f in find0(dir="test_find0_depth", type="f", maxdepth=1) {
        print(f)
    }
    run("rm", "-rf", "test_find0_depth")
}
"#;
    run_test_bash_only(
        "find0_maxdepth",
        src,
        "test_find0_depth/top.txt",
    );
}

#[test]
fn find0_combined() {
    let src = r#"
func main() {
    run("mkdir", "-p", "test_find0_combo/a")
    run("touch", "test_find0_combo/x.txt", "test_find0_combo/y.log", "test_find0_combo/a/z.txt")
    for f in find0(dir="test_find0_combo", name="*.txt", type="f", maxdepth=1) {
        print(f)
    }
    run("rm", "-rf", "test_find0_combo")
}
"#;
    run_test_bash_only(
        "find0_combined",
        src,
        "test_find0_combo/x.txt",
    );
}

#[test]
fn find0_special_chars() {
    // Test filenames with spaces and quotes
    let src = r#"
func main() {
    run("mkdir", "-p", "test_find0_special")
    run("touch", "test_find0_special/hello world.txt", "test_find0_special/it's.txt", "test_find0_special/normal.txt")
    for f in find0(dir="test_find0_special", type="f") {
        print(f)
    }
    run("rm", "-rf", "test_find0_special")
}
"#;
    run_test_bash_only(
        "find0_special_chars",
        src,
        "test_find0_special/hello world.txt\ntest_find0_special/it's.txt\ntest_find0_special/normal.txt",
    );
}

#[test]
fn find0_empty_dir() {
    let src = r#"
func main() {
    run("mkdir", "-p", "test_find0_empty")
    let cnt = "0"
    for f in find0(dir="test_find0_empty", type="f") {
        set cnt = "1"
    }
    print(cnt)
    run("rm", "-rf", "test_find0_empty")
}
"#;
    run_test_bash_only(
        "find0_empty_dir",
        src,
        "0",
    );
}

#[test]
fn find0_defaults() {
    // find0() with name filter on current directory
    let src = r#"
func main() {
    run("mkdir", "-p", "test_find0_defaults")
    run("touch", "test_find0_defaults/one.txt", "test_find0_defaults/two.txt")
    for f in find0(dir="test_find0_defaults", name="*.txt") {
        print(f)
    }
    run("rm", "-rf", "test_find0_defaults")
}
"#;
    run_test_bash_only(
        "find0_defaults",
        src,
        "test_find0_defaults/one.txt\ntest_find0_defaults/two.txt",
    );
}

#[test]
fn find0_newline_filename() {
    let src = r#"
func main() {
    run("mkdir", "-p", "test_find0_newline")
    # Create a file with newline in name
    # We use sh() to ensure exact creation if run() had issues, but run() should be safe.
    # Using python/perl/printf to create it reliably without shell interpretation issues?
    # Actually run("touch", ...) passes arg safely.
    let name = "test_find0_newline/foo\nbar.txt"
    run("touch", name)
    
    for f in find0(dir="test_find0_newline", type="f") {
        print(f)
    }
    run("rm", "-rf", "test_find0_newline")
}
"#;
    run_test_bash_only(
        "find0_newline_filename",
        src,
        "test_find0_newline/foo\nbar.txt",
    );
}

#[test]
fn find0_deterministic_order() {
    let src = r#"
func main() {
    run("mkdir", "-p", "test_find0_order")
    # A vs a vs b. In C locale (bytewise), A(65) < a(97) < b(98).
    run("touch", "test_find0_order/b", "test_find0_order/a", "test_find0_order/A")
    
    for f in find0(dir="test_find0_order", type="f") {
        print(f)
    }
    run("rm", "-rf", "test_find0_order")
}
"#;
    run_test_bash_only(
        "find0_deterministic_order",
        src,
        "test_find0_order/A\ntest_find0_order/a\ntest_find0_order/b",
    );
}

#[test]
fn find0_compile_fail_maxdepth_expression() {
    let src = r#"
func main() {
    for f in find0(maxdepth=(1+1)) {
        print(f)
    }
}
"#;
    let res = try_compile_to_shell(src, TargetShell::Bash);
    match res {
        Err(msg) => assert!(
            msg.contains("maxdepth must be a non-negative integer literal"),
            "Unexpected error: {}", msg
        ),
        Ok(_) => panic!("Expected compilation failure for find0(maxdepth=(1+1))"),
    }
}

#[test]
fn find0_compile_fail_maxdepth_variable() {
    let src = r#"
func main() {
    let n = 1
    for f in find0(maxdepth=n) {
        print(f)
    }
}
"#;
    let res = try_compile_to_shell(src, TargetShell::Bash);
    match res {
        Err(msg) => assert!(
            msg.contains("maxdepth must be a non-negative integer literal"),
            "Unexpected error: {}", msg
        ),
        Ok(_) => panic!("Expected compilation failure for find0(maxdepth=n)"),
    }
}

#[test]
fn find0_runtime_excludes_root_dir() {
    let src = r#"
func main() {
    run("mkdir", "-p", "test_find0_root/sub")
    run("touch", "test_find0_root/file.txt")
    // Should verify that "test_find0_root" (the starting point) is excluded.
    // If we search for type="d", we should find "test_find0_root/sub".
    // But NOT "test_find0_root".
    for f in find0(dir="test_find0_root", type="d") {
        print(f)
    }
    run("rm", "-rf", "test_find0_root")
}
"#;
    run_test_bash_only(
        "find0_runtime_excludes_root_dir",
        src,
        "test_find0_root/sub",
    );
}

#[test]
fn find0_runtime_quoting_safety_dir() {
    let src = r#"
func main() {
    // Directory with special characters: spaces, brackets, dollar
    let bad_dir = "test_find0_bad_[brackets]_$dollar"
    run("mkdir", "-p", bad_dir)
    run("touch", bad_dir & "/file.txt")
    
    for f in find0(dir=bad_dir, type="f") {
        print(f)
    }
    run("rm", "-rf", bad_dir)
}
"#;
    run_test_bash_only(
        "find0_runtime_quoting_safety_dir",
        src,
        "test_find0_bad_[brackets]_$dollar/file.txt",
    );
}

#[test]
fn find0_runtime_dash_leading_dir() {
    let src = r#"
func main() {
    let bad_dir = "-test_find0_dash_leading"
    run("mkdir", "-p", "--", bad_dir)
    run("touch", "--", bad_dir & "/file.txt")
    
    // If find0 codegen is unsafe (e.g. `find $dir`), find will treat `-test...` as an option and fail.
    // It must emit `find -- "$dir"` or similar safe invocation.
    for f in find0(dir=bad_dir, type="f") {
        print(f)
    }
    run("rm", "-rf", "--", bad_dir)
}
"#;
    run_test_bash_only(
        "find0_runtime_dash_leading_dir",
        src,
        "./-test_find0_dash_leading/file.txt",
    );
}

#[test]
fn find0_runtime_name_glob_semantics() {
    let src = r#"
func main() {
    // Test that `name` argument is passed to find without intermediate shell expansion.
    run("mkdir", "-p", "test_find0_glob")
    run("touch", "test_find0_glob/a1.txt", "test_find0_glob/ab.txt")

    // name="a?.txt" is a glob pattern.
    // If the shell expands it early, it might match nothing (and disappear) or match files in CWD.
    // We want `find` to receive 'a?.txt' literally and perform the matching itself.
    // Both a1.txt and ab.txt should match 'a?.txt'
    for f in find0(dir="test_find0_glob", name="a?.txt", type="f") {
        print(f)
    }
    run("rm", "-rf", "test_find0_glob")
}
"#;
    run_test_bash_only(
        "find0_runtime_name_glob_semantics",
        src,
        "test_find0_glob/a1.txt\ntest_find0_glob/ab.txt",
    );
}
