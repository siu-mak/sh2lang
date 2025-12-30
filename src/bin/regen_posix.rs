fn main() {
    use sh2c::codegen::{TargetShell, emit_with_target};
    use sh2c::loader;
    use sh2c::lower;
    use std::fs;
    use std::path::Path;

    let fixtures = vec![
        "tests/fixtures/call_user_func_expr",
        "tests/fixtures/imports/double_import/main",
        "tests/fixtures/imports/diamond/main",
        "tests/fixtures/toplevel_print",
        "tests/fixtures/toplevel_let_and_run",
    ];

    for fixture_base in fixtures {
        // Handle .sh2 vs directory
        let sh2_path = if Path::new(&format!("{}.sh2", fixture_base)).exists() {
            format!("{}.sh2", fixture_base)
        } else {
            format!("{}.sh2", fixture_base)
        };

        let sh2_path_obj = Path::new(&sh2_path);
        // Special case for directories (imports) - the loader expects the entry file
        let entry_path = if sh2_path_obj.exists() {
            sh2_path_obj.to_path_buf()
        } else {
            // Try without extension if it was a directory path in the list
            Path::new(fixture_base).join("main.sh2")
        };

        // Actually, for imports/double_import/main, the fixture list string is the base name.
        // For `tests/fixtures/imports/double_import/main`, valid path is `tests/fixtures/imports/double_import/main.sh2`

        let final_path = if entry_path.exists() {
            entry_path
        } else {
            // Fallback for directory/file ambiguity in my simple script
            Path::new(fixture_base).with_extension("sh2")
        };

        println!("Regenerating: {}", final_path.display());
        let program = loader::load_program_with_imports(&final_path).unwrap();
        let ir = lower::lower(program);
        let posix_code = emit_with_target(&ir, TargetShell::Posix);

        let expected_path = format!("{}.posix.sh.expected", fixture_base);
        fs::write(&expected_path, posix_code).expect("Failed to write expected file");
    }
}
