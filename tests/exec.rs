use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use sh2c::{lexer, parser, lower, codegen};

fn compile_to_bash(src: &str) -> String {
    let tokens = lexer::lex(src);
    let program = parser::parse(&tokens);
    let ir = lower::lower(program);
    codegen::emit(&ir)
}

fn write_temp_script(prefix: &str, bash: &str) -> std::path::PathBuf {
    let pid = std::process::id();
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let filename = format!("{}_{}_{}.sh", prefix, pid, nanos);
    let mut path = std::env::temp_dir();
    path.push(filename);
    fs::write(&path, bash).expect("Failed to write temp script");
    path
}

#[test]
fn hello_executes_correctly() {
    let src = r#"
        func main() {
            run("echo", "hello")
        }
    "#;

    let bash = compile_to_bash(src);
    let path = write_temp_script("sh2_test", &bash);

    let output = Command::new("bash")
        .arg(&path)
        .output()
        .expect("Failed to execute bash");

    let stdout = String::from_utf8_lossy(&output.stdout);
    fs::remove_file(&path).ok();

    assert_eq!(stdout.trim(), "hello");
}

#[test]
fn if_executes_when_var_is_set() {
    let src = r#"
        func main() {
            if TESTVAR {
                run("echo", "yes")
            }
        }
    "#;

    let bash = compile_to_bash(src);
    let path = write_temp_script("sh2_if_test", &bash);

    let output = Command::new("bash")
        .env("TESTVAR", "1")
        .arg(&path)
        .output()
        .expect("Failed to execute bash");

    let stdout = String::from_utf8_lossy(&output.stdout);
    fs::remove_file(&path).ok();
    assert_eq!(stdout.trim(), "yes");
}


#[test]
fn if_does_not_execute_when_var_is_empty() {
    let src = r#"
        func main() {
            if TESTVAR {
                run("echo", "yes")
            }
        }
    "#;

    let bash = compile_to_bash(src);
    let path = write_temp_script("sh2_if_empty_test", &bash);

    let output = Command::new("bash")
        .arg(&path)
        .output()
        .expect("Failed to execute bash");

    let stdout = String::from_utf8_lossy(&output.stdout);
    fs::remove_file(&path).ok();
    assert_eq!(stdout.trim(), "");
}

#[test]
fn print_err_writes_to_stderr() {
    let src = r#"
        func main() {
            print_err("fail")
        }
    "#;

    let bash = compile_to_bash(src);
    let path = write_temp_script("sh2_err", &bash);

    let out = Command::new("bash")
        .arg(&path)
        .output()
        .expect("Failed to execute bash");

    let stderr = String::from_utf8_lossy(&out.stderr);
    fs::remove_file(&path).ok();
    assert!(stderr.contains("fail"));
}

#[test]
fn else_executes_when_var_is_empty() {
    let src = r#"
        func main() {
            if TESTVAR {
                print("yes")
            } else {
                print("no")
            }
        }
    "#;

    let bash = compile_to_bash(src);
    let path = write_temp_script("sh2_else_test", &bash);

    let out = Command::new("bash")
        .arg(&path)
        .output()
        .expect("Failed to execute bash");

    let stdout = String::from_utf8_lossy(&out.stdout);
    fs::remove_file(&path).ok();
    assert_eq!(stdout.trim(), "no");
}

#[test]
fn parses_empty_function_body() {
    let src = r#"
        func main() {
        }
    "#;

    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);

    assert_eq!(ast.functions.len(), 1);
    assert_eq!(ast.functions[0].body.len(), 0);
}

#[test]
fn parses_multiple_functions() {
    let src = r#"
        func a() { print("x") }
        func b() { print("y") }
    "#;

    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);

    assert_eq!(ast.functions.len(), 2);
    assert_eq!(ast.functions[0].name, "a");
    assert_eq!(ast.functions[1].name, "b");
}

#[test]
fn exec_for_list_var() {
    let src = r#"
        func main() {
            let xs = ["a", "b", "c"]
            for x in xs {
                print(x)
            }
        }
    "#;

    let bash = compile_to_bash(src);
    let path = write_temp_script("sh2_for_list", &bash);

    let output = Command::new("bash")
        .arg(&path)
        .output()
        .expect("Failed to execute bash");

    let stdout = String::from_utf8_lossy(&output.stdout);
    fs::remove_file(&path).ok();

    let expected = "a\nb\nc\n";
    assert_eq!(stdout.replace("\r\n", "\n").trim(), expected.trim());
}
