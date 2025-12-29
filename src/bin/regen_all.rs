fn main() {
    use sh2c::codegen::{TargetShell, emit_with_target};
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
        let program = loader::load_program_with_imports(&path);
        let opts = lower::LowerOptions {
            include_diagnostics: true,
            diag_base_dir: Some(fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR"))).unwrap()),
        };
        
        // Lowering (might panic, but we deal with it mostly working for valid fixtures)
        // If this panicked before, it would be caught. Now it propagates? 
        // No, let's keep a catch_unwind wrapper for safety if needed, or just let it crash (regen usually shouldn't crash on frontend).
        // To respect existing logic, let's wrap lowering too but separately.
        
        let ir_res = panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
             lower::lower_with_options(program, &opts)
        }));

        match ir_res {
            Ok(ir) => {
                // Try Bash
                let bash_res = panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    emit_with_target(&ir, TargetShell::Bash)
                }));
                if let Ok(bash) = bash_res {
                     let bash_path = path.with_extension("sh.expected");
                     if let Err(e) = fs::write(&bash_path, bash) {
                         eprintln!("Failed to write {}: {}", bash_path.display(), e);
                     }
                }
                
                // Try Posix
                let posix_res = panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    emit_with_target(&ir, TargetShell::Posix)
                }));
                 if let Ok(posix) = posix_res {
                     let posix_path = path.with_extension("posix.sh.expected");
                     if let Err(e) = fs::write(&posix_path, posix) {
                         eprintln!("Failed to write {}: {}", posix_path.display(), e);
                     }
                } else {
                     // print failure for posix only if needed, but silence is golden for non-portable tests
                }
            }
            Err(_) => {
                println!("Skipping {} (loading/lowering failed)", path.display());
            }
        }

    }
}
