fn main() {
    use sh2c::codegen::{TargetShell, emit_with_options, CodegenOptions};
    use sh2c::loader;
    use sh2c::lower;
    use std::fs;
    use std::panic;
    use std::path::PathBuf;

    let dir = fs::read_dir("tests/fixtures").expect("Failed to read fixtures dir");
    let mut files: Vec<PathBuf> = dir
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map_or(false, |ext| ext == "sh2"))
        .collect();
    files.sort();

    for path in files {
        println!("Processing {}", path.display());
        let program = match loader::load_program_with_imports(&path) {
            Ok(p) => p,
            Err(e) => {
                println!("Skipping {} (loading failed: {})", path.display(), e.msg);
                continue;
            }
        };
        let opts = lower::LowerOptions {
            include_diagnostics: true,
            diag_base_dir: Some(fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR"))).unwrap()),
        };
        
        // Lowering (might panic, but we deal with it mostly working for valid fixtures)
        // If this panicked before, it would be caught. Now it propagates? 
        // No, let's keep a catch_unwind wrapper for safety if needed, or just let it crash (regen usually shouldn't crash on frontend).
        // To respect existing logic, let's wrap lowering too but separately.
        
        // Lowering
        let ir_res = lower::lower_with_options(program, &opts);

        match ir_res {
            Ok(ir) => {
                // Try Bash
                if let Ok(bash) = emit_with_options(&ir, CodegenOptions { target: TargetShell::Bash, include_diagnostics: true }) {
                     let bash_path = path.with_extension("sh.expected");
                     if let Err(e) = fs::write(&bash_path, bash) {
                         eprintln!("Failed to write {}: {}", bash_path.display(), e);
                     }
                }
                
                // Try Posix
                if let Ok(posix) = emit_with_options(&ir, CodegenOptions { target: TargetShell::Posix, include_diagnostics: true }) {
                     let posix_path = path.with_extension("posix.sh.expected");
                     if let Err(e) = fs::write(&posix_path, posix) {
                         eprintln!("Failed to write {}: {}", posix_path.display(), e);
                     }
                }
            }
            Err(_) => {
                println!("Skipping {} (lowering failed)", path.display());
            }
        }

    }
}
