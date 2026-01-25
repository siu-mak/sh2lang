use crate::ast::{Function, Program};
use crate::lexer;
use crate::parser;
use crate::span::SourceMap;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::span::Diagnostic;

struct Loader {
    loaded: HashSet<PathBuf>,
    visiting: HashSet<PathBuf>,
    stack: Vec<PathBuf>,
    source_maps: HashMap<String, SourceMap>,
    functions: HashMap<String, (Function, PathBuf)>,
    function_order: Vec<String>,
}

impl Loader {
    fn new() -> Self {
        Loader {
            loaded: HashSet::new(),
            visiting: HashSet::new(),
            stack: Vec::new(),
            source_maps: HashMap::new(),
            functions: HashMap::new(),
            function_order: Vec::new(),

        }
    }
}

pub fn load_program_with_imports(entry_path: &Path) -> Result<Program, Diagnostic> {
    load(entry_path)
}

// ...

fn load_program_with_imports_impl(loader: &mut Loader, entry_path: &Path) -> Result<(), Diagnostic> {
    let canonical_path = match fs::canonicalize(entry_path) {
        Ok(p) => p,
        Err(e) => {
            return Err(Diagnostic {
                msg: {
                    let mut m = format!("Failed to resolve path {}: {}", entry_path.display(), e);
                    let p_str = entry_path.to_string_lossy();
                    if p_str.starts_with("~") {
                        m.push_str("\nhint: '~' is not expanded; use env.HOME & \"/path\" (or an absolute path).");
                    }
                    m
                },
                span: crate::span::Span::new(0, 0),
                sm: None,
                file: Some(entry_path.to_string_lossy().to_string()),
            });
        }
    };

    // Idempotency: If already loaded, do nothing (no-op).
    if loader.loaded.contains(&canonical_path) {
        return Ok(());
    }

    // Cycle detection
    if loader.visiting.contains(&canonical_path) {
        // ... same cycle panic ...
        // Keeping panic for internal invariants/loader errors not related to syntax for now? 
        // Or should I use Diagnostic? 
        // Cycle is a "user error" in imports. 
        // Prompt said "parse/lex errors". Imports are semantic. Keep panic for now to reduce scope.
        // But formatting it as Diagnostic would be nice. 
        // Let's keep panic as it's not strictly "parser".
        let mut cycle_msg = String::new();
        for p in &loader.stack {
            cycle_msg.push_str(&format!("{} -> ", p.display()));
        }
        cycle_msg.push_str(&format!("{}", canonical_path.display()));

        return Err(Diagnostic {
            msg: format!("Import cycle detected: {}", cycle_msg),
            span: crate::span::Span::new(0, 0),
            sm: None,
            file: Some(canonical_path.to_string_lossy().to_string()),
        });
    }

    loader.visiting.insert(canonical_path.clone());
    loader.stack.push(canonical_path.clone());

    let src = match fs::read_to_string(&canonical_path) {
        Ok(s) => s,
        Err(e) => {
            return Err(Diagnostic {
                msg: format!("Failed to read {}: {}", canonical_path.display(), e),
                span: crate::span::Span::new(0, 0),
                sm: None,
                file: Some(canonical_path.to_string_lossy().to_string()),
            });
        }
    };

    let file_str = canonical_path.to_string_lossy().to_string();
    let sm = SourceMap::new(src);
    loader.source_maps.insert(file_str.clone(), sm);

    let sm_ref = loader.source_maps.get(&file_str).unwrap();

    let tokens = lexer::lex(sm_ref, &file_str)?;
    let program = parser::parse(&tokens, sm_ref, &file_str)?;

    let base_dir = canonical_path.parent().unwrap_or(Path::new("."));
    for import_str in program.imports {
        let mut import_path = base_dir.join(&import_str);
        if import_path.extension().is_none() {
            import_path.set_extension("sh2");
        }
        load_program_with_imports_impl(loader, &import_path)?;
    }
    
    // ... rest of loop ...
    for func in program.functions {
        // ... (keep panics for semantics) ...
        if matches!(
            func.name.as_str(),
            "trim" | "before" | "after" | "replace" | "split"
        ) {
            return Err(Diagnostic {
                msg: format!(
                    "Function name '{}' is reserved (prelude helper); choose a different name.",
                    func.name
                ),
                span: func.span, // We have func.span here!
                sm: loader.source_maps.get(&func.file).cloned(),
                file: Some(func.file.clone()),
            });
        }

        if let Some((_, defined_at)) = loader.functions.get(&func.name) {
            return Err(Diagnostic {
                msg: format!(
                    "Function '{}' is already defined in {}",
                    func.name,
                    defined_at.display()
                ),
                span: func.span,
                sm: loader.source_maps.get(&func.file).cloned(),
                file: Some(func.file.clone()),
            });
        }
        // Extract function to own it
        let name = func.name.clone();
        loader.function_order.push(name.clone());
        loader
            .functions
            .insert(name, (func, canonical_path.clone()));
    }



    loader.visiting.remove(&canonical_path);
    loader.stack.pop();
    loader.loaded.insert(canonical_path);
    Ok(())
}

pub fn load(entry_path: &Path) -> Result<Program, Diagnostic> {
    let mut loader = Loader::new();
    load_program_with_imports_impl(&mut loader, entry_path)?;

    // Construct final program in deterministic order
    let mut functions = Vec::new();
    for name in loader.function_order {
        let (func, _) = loader.functions.remove(&name).unwrap();
        functions.push(func);
    }

    // ... same span logic ...
    let span = crate::span::Span { start: 0, end: 0 };
    let entry_file = fs::canonicalize(entry_path)
        .map_err(|e| Diagnostic {
            msg: {
                let mut m = format!("Failed to resolve path {}: {}", entry_path.display(), e);
                let p_str = entry_path.to_string_lossy();
                if p_str.starts_with("~") {
                    m.push_str("\nhint: '~' is not expanded; use env.HOME & \"/path\" (or an absolute path).");
                }
                m
            },
            span: crate::span::Span::new(0, 0),
            sm: None,
            file: Some(entry_path.to_string_lossy().to_string()),
        })?
        .to_string_lossy()
        .to_string();

    Ok(Program {
        imports: vec![],
        functions,

        span,
        source_maps: loader.source_maps,
        entry_file,
    })
}
