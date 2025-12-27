use crate::ast::{Program, Function};
use crate::lexer;
use crate::parser;
use std::collections::{HashSet, HashMap};
use std::path::{Path, PathBuf};
use std::fs;

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
}

impl Loader {
    fn new() -> Self {
        Loader {
            visiting: HashSet::new(),
            stack: Vec::new(),
            loaded: HashSet::new(),
            functions: HashMap::new(),
            function_order: Vec::new(),
        }
    }
}

// Clean up: make load return void and logic cleaner
impl Loader {
    // ... (rest is fine)
}

// Re-write load_program_with_imports to separate the recursive updating from final construction
fn load_program_with_imports_impl(loader: &mut Loader, entry_path: &Path) {
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

    let tokens = lexer::lex(&src);
    let program = parser::parse(&tokens);

    let base_dir = canonical_path.parent().unwrap_or(Path::new("."));
    for import_str in program.imports {
        let mut import_path = base_dir.join(&import_str);
        if import_path.extension().is_none() {
            import_path.set_extension("sh2");
        }
        load_program_with_imports_impl(loader, &import_path);
    }

    for func in program.functions {
        if let Some((_, defined_at)) = loader.functions.get(&func.name) {
            panic!("Function '{}' is already defined in {}", func.name, defined_at.display());
        }
        // Extract function to own it
        let name = func.name.clone();
        loader.function_order.push(name.clone());
        loader.functions.insert(name, (func, canonical_path.clone()));
    }
    
    loader.visiting.remove(&canonical_path);
    loader.stack.pop();
    loader.loaded.insert(canonical_path);
}

// Final wrapper
pub fn load(entry_path: &Path) -> Program {
    let mut loader = Loader::new();
    load_program_with_imports_impl(&mut loader, entry_path);
    
    // Construct final program in deterministic order
    let mut functions = Vec::new();
    for name in loader.function_order {
        let (func, _) = loader.functions.remove(&name).unwrap();
        functions.push(func);
    }
    
    Program { imports: vec![], functions }
}
