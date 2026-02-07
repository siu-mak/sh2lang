use crate::codegen::{self, TargetShell};
use crate::loader;
use crate::lower;
use crate::semantics;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Default,
    Check,
    EmitAst,
    EmitIr,
    EmitSh,
}

#[derive(Debug)]
pub struct CompileOptions {
    pub target: TargetShell,
    pub include_diagnostics: bool,
    pub out_path: Option<PathBuf>,
    pub chmod_x: bool,
    pub mode: Mode,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            target: TargetShell::Bash,
            include_diagnostics: true,
            out_path: None,
            chmod_x: false, // Default: false (library hygiene)
            mode: Mode::Default, // Default: EmitSh behavior
        }
    }
}

pub struct DriverError {
    pub code: i32,
    pub msg: String,
}

impl DriverError {
    fn compile(msg: String) -> Self {
        Self { code: 2, msg }
    }
    
    fn io(msg: String) -> Self {
        Self { code: 1, msg }
    }
}

pub fn compile_file(path: &Path, options: CompileOptions) -> Result<String, DriverError> {
    let diag_base_dir = path.parent()
        .map(|p| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf()));
        
    // IO Check: Ensure file exists and is readable to return correct exit code (1) vs compile error (2)
    if !path.exists() {
        return Err(DriverError::io(format!("File not found: {}", path.display())));
    }
    if let Err(e) = std::fs::File::open(path) {
        return Err(DriverError::io(format!("Unable to read file: {} ({})", path.display(), e)));
    }
        
    let mut ast = loader::load_program_with_imports(path)
        .map_err(|d| DriverError::compile(d.format(diag_base_dir.as_deref())))?;

    if let Mode::EmitAst = options.mode {
        ast.strip_spans();
        return Ok(format!("{:#?}", ast));
    }

    // Semantic analysis: check variable declarations before lowering
    semantics::check_semantics(&ast, &semantics::SemanticOptions {
        diag_base_dir: diag_base_dir.clone(),
    }).map_err(|e| DriverError::compile(e.to_string()))?;

    let ir = lower::lower_with_options(
        ast,
        &lower::LowerOptions {
            include_diagnostics: options.include_diagnostics,
            diag_base_dir: diag_base_dir.clone(),
        },
    ).map_err(|e| DriverError::compile(e.to_string()))?;


    if let Mode::EmitIr = options.mode {
        let mut ir_stripped = ir;
        for f in &mut ir_stripped {
             f.strip_spans();
        }
        return Ok(format!("{:#?}", ir_stripped));
    }

    if let Mode::Check = options.mode {
        codegen::emit_with_options_checked(
            &ir,
            codegen::CodegenOptions {
                target: options.target,
                include_diagnostics: options.include_diagnostics,
            },
        ).map_err(|e| DriverError::compile(e.to_string()))?;
        return Ok("OK".to_string());
    }

    // Default or EmitSh
    let out = codegen::emit_with_options_checked(
        &ir,
        codegen::CodegenOptions {
            target: options.target,
            include_diagnostics: options.include_diagnostics,
        },
    ).map_err(|e| DriverError::compile(e.to_string()))?;
    
    if let Some(out_path) = &options.out_path {
        std::fs::write(out_path, &out)
            .map_err(|e| DriverError::io(format!("Failed to write to {}: {}", out_path.display(), e)))?;
        
        #[cfg(unix)]
        {
            if options.chmod_x {
                if let Ok(metadata) = std::fs::metadata(out_path) {
                    let mut perms = metadata.permissions();
                    perms.set_mode(perms.mode() | 0o111);
                    let _ = std::fs::set_permissions(out_path, perms);
                }
            }
        }
    }
    
    Ok(out)
}
