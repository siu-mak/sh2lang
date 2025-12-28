fn main() {
    use std::path::{Path, PathBuf};
    use sh2c::loader;
    use sh2c::lower;
    use sh2c::codegen::{emit_with_target, TargetShell};
    use std::fs;
    use std::panic;

    let dir = fs::read_dir("tests/fixtures").expect("Failed to read fixtures dir");
    let mut files: Vec<PathBuf> = dir.filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map_or(false, |ext| ext == "sh2"))
        .collect();
    files.sort();

    for path in files {
        println!("Processing {}", path.display());
        let result = panic::catch_unwind(|| {
             let program = loader::load_program_with_imports(&path);
             let ir = lower::lower(program);
             let bash_code = emit_with_target(&ir, TargetShell::Bash);
             let posix_code = emit_with_target(&ir, TargetShell::Posix);
             (bash_code, posix_code)
        });

        match result {
             Ok((bash, posix)) => {
                 let bash_path = path.with_extension("sh.expected");
                 fs::write(&bash_path, bash).expect("Failed to write .sh.expected");
                 
                 let posix_path = path.with_extension("posix.sh.expected");
                 fs::write(&posix_path, posix).expect("Failed to write .posix.sh.expected");
             },
             Err(_) => {
                 println!("Skipping {} (compilation/lowering/codegen failed)", path.display());
             }
        }
    }
}
