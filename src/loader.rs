use crate::ast::{Function, Program};
use crate::lexer;
use crate::parser;
use crate::span::SourceMap;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub fn load_program_with_imports(entry_path: &Path) -> Program {
    load(entry_path)
}

struct Loader {
    // Canonical paths currently being visited (for cycle detection)
    visiting: HashSet<PathBuf>,
    // Stack of paths being visited (for error reporting)
    stack: Vec<PathBuf>,
    // Canonical paths already fully loaded (idempotency)
    loaded: HashSet<PathBuf>,
    // All loaded functions, keyed by name (for conflict detection)
    // Value is (Function definition, Source file path)
    functions: HashMap<String, (Function, PathBuf)>,
    // To preserve deterministic order of functions: store names in order of definition/loading
    function_order: Vec<String>,
    // Top level statements from the entry file
    entry_top_level: Vec<crate::ast::Stmt>,
    // Collected source maps
    source_maps: HashMap<String, SourceMap>,
}

impl Loader {
    fn new() -> Self {
        Loader {
            visiting: HashSet::new(),
            stack: Vec::new(),
            loaded: HashSet::new(),
            functions: HashMap::new(),
            function_order: Vec::new(),
            entry_top_level: Vec::new(),
            source_maps: HashMap::new(),
        }
    }
}

fn load_program_with_imports_impl(loader: &mut Loader, entry_path: &Path, is_entry: bool) {
    let canonical_path = match fs::canonicalize(entry_path) {
        Ok(p) => p,
        Err(e) => panic!("Failed to resolve path {}: {}", entry_path.display(), e),
    };

    // Idempotency: If already loaded, do nothing (no-op).
    if loader.loaded.contains(&canonical_path) {
        return;
    }

    // Cycle detection
    if loader.visiting.contains(&canonical_path) {
        // Construct detailed cycle path
        let mut cycle_msg = String::new();
        for p in &loader.stack {
            cycle_msg.push_str(&format!("{} -> ", p.display()));
        }
        cycle_msg.push_str(&format!("{}", canonical_path.display()));

        panic!("Import cycle detected: {}", cycle_msg);
    }

    loader.visiting.insert(canonical_path.clone());
    loader.stack.push(canonical_path.clone());

    let src = match fs::read_to_string(&canonical_path) {
        Ok(s) => s,
        Err(e) => panic!("Failed to read {}: {}", canonical_path.display(), e),
    };

    let file_str = canonical_path.to_string_lossy().to_string();
    let sm = SourceMap::new(src);
    loader.source_maps.insert(file_str.clone(), sm);

    let sm_ref = loader.source_maps.get(&file_str).unwrap();

    let tokens = lexer::lex(sm_ref, &file_str);
    let program = parser::parse(&tokens, sm_ref, &file_str);

    let base_dir = canonical_path.parent().unwrap_or(Path::new("."));
    for import_str in program.imports {
        let mut import_path = base_dir.join(&import_str);
        if import_path.extension().is_none() {
            import_path.set_extension("sh2");
        }
        load_program_with_imports_impl(loader, &import_path, false);
    }

    for func in program.functions {
        // Reserved name check
        if matches!(
            func.name.as_str(),
            "trim" | "before" | "after" | "replace" | "split"
        ) {
            panic!(
                "Function name '{}' is reserved (prelude helper); choose a different name.",
                func.name
            );
        }

        if let Some((_, defined_at)) = loader.functions.get(&func.name) {
            panic!(
                "Function '{}' is already defined in {}",
                func.name,
                defined_at.display()
            );
        }
        // Extract function to own it
        let name = func.name.clone();
        loader.function_order.push(name.clone());
        loader
            .functions
            .insert(name, (func, canonical_path.clone()));
    }

    if is_entry {
        loader.entry_top_level = program.top_level;
        // Also capture the program span if needed? Program span is just file span.
    } else {
        if !program.top_level.is_empty() {
            panic!(
                "Top-level statements are only allowed in the entry file (found in {})",
                canonical_path.display()
            );
        }
    }

    loader.visiting.remove(&canonical_path);
    loader.stack.pop();
    loader.loaded.insert(canonical_path);
}

pub fn load(entry_path: &Path) -> Program {
    let mut loader = Loader::new();
    load_program_with_imports_impl(&mut loader, entry_path, true);

    // Construct final program in deterministic order
    let mut functions = Vec::new();
    for name in loader.function_order {
        let (func, _) = loader.functions.remove(&name).unwrap();
        functions.push(func);
    }

    // We need a span for the final merged program.
    // It's conceptually the entry file's span, or a synthetic one.
    // Using 0..0 is fine as Program span isn't used much?
    // Or we should grab it from loader.
    // But we don't store the entry program struct.
    // Let's just make a dummy span 0..0 for now, or use empty.
    let span = crate::span::Span { start: 0, end: 0 };

    // We need the entry file name.
    // Since load_program_with_imports_impl canonicalizes, we should too to match keys.
    let entry_file = fs::canonicalize(entry_path)
        .unwrap()
        .to_string_lossy()
        .to_string();

    Program {
        imports: vec![],
        functions,
        top_level: loader.entry_top_level,
        span,
        source_maps: loader.source_maps,
        entry_file,
    }
}
