use sh2c::codegen::{self, TargetShell};
use sh2c::{lexer, lower, parser};
use sh2c::loader;
use std::path::Path;

pub fn try_compile_to_shell(src: &str, target: TargetShell) -> Result<String, String> {
    let sm = sh2c::span::SourceMap::new(src.to_string());
    let tokens = lexer::lex(&sm, src).map_err(|d| d.format(None))?;
    let mut program = parser::parse(&tokens, &sm, "inline_test").map_err(|d| d.format(None))?;
    program.source_maps.insert("inline_test".to_string(), sm);
    
    // Semantic analysis: check variable declarations
    sh2c::semantics::check_semantics(&program, &sh2c::semantics::SemanticOptions::default())
        .map_err(|e| e.message)?;
    
    let opts = sh2c::lower::LowerOptions {
        include_diagnostics: true,
        diag_base_dir: Some(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))),
        target,
    };

    // Use formatted diagnostics for better error messages
    let ir = lower::lower_with_options(program, &opts).map_err(|e| e.message)?; 
    
    codegen::emit_with_options(&ir, codegen::CodegenOptions { target, include_diagnostics: true }).map_err(|e| e.message)
}

pub fn compile_path_to_shell(path: &Path, target: TargetShell) -> String {
    try_compile_path_to_shell(path, target).unwrap_or_else(|e| panic!("{}", e))
}

pub fn try_compile_path_to_shell(path: &Path, target: TargetShell) -> Result<String, String> {
    let program = loader::load_program_with_imports(path)
        .map_err(|d| d.format(path.parent()))?;
        
    let opts = sh2c::lower::LowerOptions {
        include_diagnostics: true,
        diag_base_dir: Some(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))),
        target,
    };
    
    let ir = lower::lower_with_options(program, &opts).map_err(|e| e.message)?;
    codegen::emit_with_options(&ir, codegen::CodegenOptions { target, include_diagnostics: true }).map_err(|e| e.message)
}

pub fn compile_to_bash(src: &str) -> String {
    // Legacy support for string-based tests if any exist (e.g. unit tests not from fixtures)
    // But they won't support imports.
    compile_to_shell(src, TargetShell::Bash)
}
pub fn compile_to_shell(src: &str, target: TargetShell) -> String {
    let sm = sh2c::span::SourceMap::new(src.to_string());
    let tokens = lexer::lex(&sm, src);
    let tokens = tokens.unwrap_or_else(|d| panic!("{}", d.format(None))); 

    let mut program = parser::parse(&tokens, &sm, "inline_test")
        .unwrap_or_else(|d| panic!("{}", d.format(None)));
    program.source_maps.insert("inline_test".to_string(), sm);
    
    // Semantic analysis: check variable declarations
    sh2c::semantics::check_semantics(&program, &sh2c::semantics::SemanticOptions::default())
        .expect("Semantic analysis failed");
    
    // Note: lower calls generally require accurate file info but here we use "inline_test"
    let opts = sh2c::lower::LowerOptions {
        include_diagnostics: true,
        diag_base_dir: Some(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))),
        target,
    };
    let ir = lower::lower_with_options(program, &opts).expect("Lowering failed");
    codegen::emit_with_options(&ir, codegen::CodegenOptions { target, include_diagnostics: true }).expect("Codegen failed")
}
