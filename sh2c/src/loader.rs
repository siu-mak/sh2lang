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
    file_functions: HashMap<PathBuf, HashSet<String>>,
    /// Per-file function store for robust lazy clone on demand.
    /// Avoids relying on global name uniqueness when registering mangled functions.
    file_defined_funcs: HashMap<PathBuf, HashMap<String, Function>>,
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
            file_functions: HashMap::new(),
            file_defined_funcs: HashMap::new(),
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
                help: None,
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
            help: None,
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
                help: None,
            });
        }
    };

    let file_str = canonical_path.to_string_lossy().to_string();
    let sm = SourceMap::new(src);
    // Invariant: source_maps is keyed by canonical-path string (file_str).
    // ImportIndex.sm retrieval below must use the same key.
    loader.source_maps.insert(file_str.clone(), sm.clone());

    let tokens = lexer::lex(&sm, &file_str)?;
    let mut program = parser::parse(&tokens, &sm, &file_str)?;

    let base_dir = canonical_path.parent().unwrap_or(Path::new("."));
    
    // 1. Build local alias_map for this file
    let mut alias_map: HashMap<String, PathBuf> = HashMap::new();

    for import in &program.imports {
        let mut import_path = base_dir.join(&import.path);
        if import_path.extension().is_none() {
            import_path.set_extension("sh2");
        }
        
        let import_canonical = match fs::canonicalize(&import_path) {
            Ok(p) => p,
            Err(e) => {
                return Err(Diagnostic {
                    msg: format!("Failed to resolve path {}: {}", import_path.display(), e),
                    span: import.span,
                    sm: loader.source_maps.get(&file_str).cloned(),
                    file: Some(file_str.clone()),
                    help: None,
                });
            }
        };
        
        if let Some(ref alias) = import.alias {
            alias_map.insert(alias.clone(), import_canonical.clone());
        }
        
        load_program_with_imports_impl(loader, &import_path)?;
    }
    
    // 2. Populate file_functions for this file (before rewrite, so cross-file validation works)
    let func_names: HashSet<String> = program.functions.iter().map(|f| f.name.clone()).collect();
    loader.file_functions.insert(canonical_path.clone(), func_names);

    // 3. Resolve Pass: Validate qualified calls and fill resolved_path/resolved_mangled
    // Invariant: file_str is the canonical-path string used as source_maps key above.
    let index = crate::resolver::ImportIndex {
        alias_map: &alias_map,
        file_functions: &loader.file_functions,
        file: &file_str,
        sm: loader.source_maps.get(&file_str),
    };
    crate::resolver::resolve_qualified_calls(&mut program, &index)?;

    #[cfg(debug_assertions)]
    crate::resolver::debug_assert_program_resolved(&program);

    // 4. Rewrite Pass: Mechanically replace QualifiedCall with Call, and QualifiedCommandWord with Literal
    let mut all_needed: Vec<(String, String, PathBuf)> = Vec::new();
    let mut needed_set: HashSet<(String, String, PathBuf)> = HashSet::new();
    for func in &mut program.functions {
        rewrite_qualified_calls(func, &mut all_needed, &mut needed_set);
    }

    // Populate file_defined_funcs AFTER rewrite so cloned functions have no QualifiedCall nodes.
    // D1 lazy registration clones from here, so clones must already be rewritten.
    let func_map: HashMap<String, Function> = program.functions.iter()
        .map(|f| (f.name.clone(), f.clone())).collect();
    loader.file_defined_funcs.insert(canonical_path.clone(), func_map);

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
                help: None,
            });
        }

        if let Some((_, defined_at)) = loader.functions.get(&func.name) {
            // It could be a mangled function defined multiple times via alias aliasing?
            // Wait, if it's already defined, we don't need to panic if it's exactly the same exported function.
            // Actually, mangled functions are already deduped during rewrite if imported.
            // But if it's a regular function collision, we error.
            if !func.name.starts_with("__imp_") {
                return Err(Diagnostic {
                    msg: format!(
                        "Function '{}' is already defined in {}",
                        func.name,
                        defined_at.display()
                    ),
                    span: func.span,
                    sm: loader.source_maps.get(&func.file).cloned(),
                    file: Some(func.file.clone()),
                    help: None,
                });
            }
        }
        
        // Extract function to own it
        let name = func.name.clone();
        if !loader.functions.contains_key(&name) {
            loader.function_order.push(name.clone());
            loader.functions.insert(name, (func, canonical_path.clone()));
        }
    }

    // 5. D1 Lazy: Register only the mangled functions that were actually referenced
    //    Uses file_defined_funcs for precise per-file lookup (avoids global name uniqueness assumption)
    for (alias, func_name, target_path) in all_needed {
        let mangled = crate::names::mangle(&alias, &func_name);
        if !loader.functions.contains_key(&mangled) {
            if let Some(func_map) = loader.file_defined_funcs.get(&target_path) {
                if let Some(original_func) = func_map.get(&func_name) {
                    let mut cloned = original_func.clone();
                    cloned.name = mangled.clone();
                    loader.functions.insert(mangled.clone(), (cloned, target_path.clone()));
                    loader.function_order.push(mangled);
                }
            }
        }
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
            help: None,
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


fn rewrite_qualified_calls(
    func: &mut Function,
    needed: &mut Vec<(String, String, PathBuf)>,
    needed_set: &mut HashSet<(String, String, PathBuf)>,
) {
    for stmt in &mut func.body {
        rewrite_stmt(stmt, needed, needed_set);
    }
}

fn rewrite_stmt(
    stmt: &mut crate::ast::Stmt,
    needed: &mut Vec<(String, String, PathBuf)>,
    needed_set: &mut HashSet<(String, String, PathBuf)>,
) {
    use crate::ast::StmtKind;
    
    match &mut stmt.node {
        StmtKind::QualifiedCall { .. } => {
            let old = std::mem::replace(&mut stmt.node, StmtKind::Break);
            if let StmtKind::QualifiedCall { ns, name, args, resolved_path, resolved_mangled, .. } = old {
                let path = resolved_path.expect("resolver must run before rewrite");
                let mangled = resolved_mangled.expect("resolver must run before rewrite");
                
                let entry = (ns, name, path);
                if needed_set.insert(entry.clone()) { needed.push(entry); }
                
                stmt.node = StmtKind::Call { name: mangled, args };
            } else { unreachable!() }
        }

        StmtKind::Let { value, .. } | StmtKind::Set { value, .. } => {
            rewrite_expr(value, needed, needed_set);
        }
        StmtKind::Run(call) => {
            for a in call.args.iter_mut() { rewrite_expr(a, needed, needed_set); }
            for o in call.options.iter_mut() { rewrite_expr(&mut o.value, needed, needed_set); }
        }
        StmtKind::Print(e) | StmtKind::PrintErr(e) | StmtKind::Exit(Some(e)) | StmtKind::Return(Some(e)) | StmtKind::Wait(Some(e)) | StmtKind::Sh(e) | StmtKind::Cd { path: e } | StmtKind::Export { value: Some(e), .. } | StmtKind::Source { path: e } => {
            rewrite_expr(e, needed, needed_set);
        }
        StmtKind::If { cond, then_body, elifs, else_body } => {
            rewrite_expr(cond, needed, needed_set);
            for s in then_body.iter_mut() { rewrite_stmt(s, needed, needed_set); }
            for elif in elifs.iter_mut() {
                rewrite_expr(&mut elif.cond, needed, needed_set);
                for s in elif.body.iter_mut() { rewrite_stmt(s, needed, needed_set); }
            }
            if let Some(body) = else_body {
                for s in body.iter_mut() { rewrite_stmt(s, needed, needed_set); }
            }
        }
        StmtKind::While { cond, body } => {
            rewrite_expr(cond, needed, needed_set);
            for s in body.iter_mut() { rewrite_stmt(s, needed, needed_set); }
        }
        StmtKind::For { iterable, body, .. } => {
            match iterable {
                crate::ast::ForIterable::List(items) => {
                    for i in items.iter_mut() { rewrite_expr(i, needed, needed_set); }
                }
                crate::ast::ForIterable::Range(start, end) => {
                    rewrite_expr(start.as_mut(), needed, needed_set);
                    rewrite_expr(end.as_mut(), needed, needed_set);
                }
                crate::ast::ForIterable::Find0(spec) => {
                    if let Some(e) = &mut spec.dir { rewrite_expr(e, needed, needed_set); }
                    if let Some(e) = &mut spec.name { rewrite_expr(e, needed, needed_set); }
                    if let Some(e) = &mut spec.type_filter { rewrite_expr(e, needed, needed_set); }
                    if let Some(e) = &mut spec.maxdepth { rewrite_expr(e, needed, needed_set); }
                }
                _ => {}
            }
            for s in body.iter_mut() { rewrite_stmt(s, needed, needed_set); }
        }
        StmtKind::ForMap { body, .. } => {
            for s in body.iter_mut() { rewrite_stmt(s, needed, needed_set); }
        }
        StmtKind::TryCatch { try_body, catch_body } => {
            for s in try_body.iter_mut() { rewrite_stmt(s, needed, needed_set); }
            for s in catch_body.iter_mut() { rewrite_stmt(s, needed, needed_set); }
        }
        StmtKind::Pipe(segments) => {
            for seg in segments.iter_mut() {
                match &mut seg.node {
                    crate::ast::PipeSegment::Run(call) | crate::ast::PipeSegment::Sudo(call) => {
                        for a in call.args.iter_mut() { rewrite_expr(a, needed, needed_set); }
                        for o in call.options.iter_mut() { rewrite_expr(&mut o.value, needed, needed_set); }
                    }
                    crate::ast::PipeSegment::Block(body) | crate::ast::PipeSegment::EachLine(_, body) => {
                        for s in body.iter_mut() { rewrite_stmt(s, needed, needed_set); }
                    }
                }
            }
        }
        StmtKind::Exec(args) => {
            for a in args.iter_mut() { rewrite_expr(a, needed, needed_set); }
        }
        StmtKind::WithEnv { bindings, body } => {
            for (_, v) in bindings.iter_mut() { rewrite_expr(v, needed, needed_set); }
            for s in body.iter_mut() { rewrite_stmt(s, needed, needed_set); }
        }
        StmtKind::WithCwd { path, body } => {
            rewrite_expr(path, needed, needed_set);
            for s in body.iter_mut() { rewrite_stmt(s, needed, needed_set); }
        }
        StmtKind::WithLog { path, body, .. } => {
            rewrite_expr(path, needed, needed_set);
            for s in body.iter_mut() { rewrite_stmt(s, needed, needed_set); }
        }
        StmtKind::WithRedirect { stdout, stderr, stdin, body } => {
            if let Some(targets) = stdout {
                for t in targets.iter_mut() {
                    if let crate::ast::RedirectOutputTarget::File { path, .. } = &mut t.node {
                        rewrite_expr(path, needed, needed_set);
                    }
                }
            }
            if let Some(targets) = stderr {
                for t in targets.iter_mut() {
                    if let crate::ast::RedirectOutputTarget::File { path, .. } = &mut t.node {
                        rewrite_expr(path, needed, needed_set);
                    }
                }
            }
            if let Some(tgt) = stdin {
                if let crate::ast::RedirectInputTarget::File { path } = tgt {
                    rewrite_expr(path, needed, needed_set);
                }
            }
            for s in body.iter_mut() { rewrite_stmt(s, needed, needed_set); }
        }
        StmtKind::Case { expr, arms } => {
            rewrite_expr(expr, needed, needed_set);
            for arm in arms.iter_mut() {
                for s in arm.body.iter_mut() { rewrite_stmt(s, needed, needed_set); }
            }
        }
        StmtKind::Call { args, .. } => {
            for a in args.iter_mut() { rewrite_expr(a, needed, needed_set); }
        }
        StmtKind::AndThen { left, right } | StmtKind::OrElse { left, right } => {
            for s in left.iter_mut() { rewrite_stmt(s, needed, needed_set); }
            for s in right.iter_mut() { rewrite_stmt(s, needed, needed_set); }
        }
        StmtKind::Subshell { body } | StmtKind::Group { body } => {
            for s in body.iter_mut() { rewrite_stmt(s, needed, needed_set); }
        }
        StmtKind::Spawn { stmt: inner } => {
            rewrite_stmt(inner, needed, needed_set);
        }
        _ => {}
    }
}

fn rewrite_expr(
    expr: &mut crate::ast::Expr,
    needed: &mut Vec<(String, String, PathBuf)>,
    needed_set: &mut HashSet<(String, String, PathBuf)>,
) {
    use crate::ast::ExprKind;
    
    match &mut expr.node {
        ExprKind::QualifiedCall { .. } => {
            let old = std::mem::replace(&mut expr.node, ExprKind::Bool(false));
            if let ExprKind::QualifiedCall { ns, name, args, resolved_path, resolved_mangled, .. } = old {
                let path = resolved_path.expect("resolver must run before rewrite");
                let mangled = resolved_mangled.expect("resolver must run before rewrite");
                
                let entry = (ns, name, path);
                if needed_set.insert(entry.clone()) { needed.push(entry); }
                
                expr.node = ExprKind::Call { name: mangled, args, options: vec![] };
            } else { unreachable!() }
        }
        ExprKind::QualifiedCommandWord { .. } => {
            let old = std::mem::replace(&mut expr.node, ExprKind::Bool(false));
            if let ExprKind::QualifiedCommandWord { ns, name, resolved_path, resolved_mangled, .. } = old {
                let path = resolved_path.expect("resolver must run before rewrite");
                let mangled = resolved_mangled.expect("resolver must run before rewrite");
                
                let entry = (ns, name, path);
                if needed_set.insert(entry.clone()) { needed.push(entry); }
                
                expr.node = ExprKind::Literal(mangled);
            } else { unreachable!() }
        }
        
        ExprKind::Command(args) => {
            for a in args.iter_mut() { rewrite_expr(a, needed, needed_set); }
        }
        ExprKind::CommandPipe(pipeline) => {
            for block in pipeline.iter_mut() {
                for a in block.iter_mut() { rewrite_expr(a, needed, needed_set); }
            }
        }
        ExprKind::Concat(l, r) | ExprKind::And(l, r) | ExprKind::Or(l, r) | ExprKind::Join { list: l, sep: r } | ExprKind::Index { list: l, index: r } => {
            rewrite_expr(l, needed, needed_set);
            rewrite_expr(r, needed, needed_set);
        }
        ExprKind::Arith { left, right, .. } | ExprKind::Compare { left, right, .. } => {
            rewrite_expr(left, needed, needed_set);
            rewrite_expr(right, needed, needed_set);
        }
        ExprKind::Not(e) | ExprKind::Exists(e) | ExprKind::IsDir(e) | ExprKind::IsFile(e) | ExprKind::IsSymlink(e) | ExprKind::IsExec(e) | ExprKind::IsReadable(e) | ExprKind::IsWritable(e) | ExprKind::IsNonEmpty(e) | ExprKind::BoolStr(e) | ExprKind::Len(e) | ExprKind::Count(e) | ExprKind::Arg(e) | ExprKind::Env(e) | ExprKind::Input(e) | ExprKind::Field { base: e, .. } => {
            rewrite_expr(e, needed, needed_set);
        }
        ExprKind::MapLiteral(entries) => {
            for (_, v) in entries.iter_mut() { rewrite_expr(v, needed, needed_set); }
        }
        ExprKind::Call { args, options, .. } | ExprKind::Sudo { args, options } => {
            for a in args.iter_mut() { rewrite_expr(a, needed, needed_set); }
            for o in options.iter_mut() { rewrite_expr(&mut o.value, needed, needed_set); }
        }
        ExprKind::Run(call) => {
            for a in call.args.iter_mut() { rewrite_expr(a, needed, needed_set); }
            for o in call.options.iter_mut() { rewrite_expr(&mut o.value, needed, needed_set); }
        }
        ExprKind::Capture { expr: inner, options } | ExprKind::Sh { cmd: inner, options } => {
            rewrite_expr(inner, needed, needed_set);
            for o in options.iter_mut() { rewrite_expr(&mut o.value, needed, needed_set); }
        }
        ExprKind::Confirm { prompt, default } => {
            rewrite_expr(prompt, needed, needed_set);
            if let Some(d) = default { rewrite_expr(d, needed, needed_set); }
        }
        _ => {}
    }
}

