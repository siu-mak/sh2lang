use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::ast::{Expr, ExprKind, Program, Stmt, StmtKind};
use crate::names;
use crate::span::{Diagnostic, SourceMap, Span};

pub struct ImportIndex<'a> {
    pub alias_map: &'a HashMap<String, PathBuf>,
    pub file_functions: &'a HashMap<PathBuf, HashSet<String>>,
    pub file: &'a str,
    pub sm: Option<&'a SourceMap>,
}

fn diag_unknown_alias(ns: &str, ns_span: Span, index: &ImportIndex) -> Diagnostic {
    Diagnostic {
        msg: format!("unknown import alias '{}'", ns),
        span: ns_span,
        sm: index.sm.cloned(),
        file: Some(index.file.to_string()),
    }
}

fn diag_unknown_func(ns: &str, name: &str, name_span: Span, index: &ImportIndex) -> Diagnostic {
    Diagnostic {
        msg: format!("unknown function '{}.{}'", ns, name),
        span: name_span,
        sm: index.sm.cloned(),
        file: Some(index.file.to_string()),
    }
}

pub fn resolve_qualified_calls(
    program: &mut Program,
    index: &ImportIndex<'_>,
) -> Result<(), Diagnostic> {
    for func in &mut program.functions {
        for stmt in &mut func.body {
            resolve_in_stmt(stmt, index)?;
        }
    }
    Ok(())
}

// SYNC WITH: debug_assert_stmt_resolved below.
// When adding new StmtKind variants that contain Expr/Stmt, update both walkers.
fn resolve_in_stmt(stmt: &mut Stmt, index: &ImportIndex) -> Result<(), Diagnostic> {
    match &mut stmt.node {
        StmtKind::QualifiedCall { ns, ns_span, name, name_span, args, resolved_path, resolved_mangled } => {
            let target_path = match index.alias_map.get(ns) {
                Some(p) => p.clone(),
                None => return Err(diag_unknown_alias(ns, *ns_span, index)),
            };
            let funcs = match index.file_functions.get(&target_path) {
                Some(f) => f,
                None => return Err(diag_unknown_func(ns, name, *name_span, index)),
            };
            if !funcs.contains(name) {
                return Err(diag_unknown_func(ns, name, *name_span, index));
            }
            *resolved_path = Some(target_path);
            *resolved_mangled = Some(names::mangle(ns, name));

            for a in args {
                resolve_in_expr(a, index)?;
            }
        }
        StmtKind::Let { value, .. } | StmtKind::Set { value, .. } => {
            resolve_in_expr(value, index)?;
        }
        StmtKind::Run(call) => {
            for a in &mut call.args {
                resolve_in_expr(a, index)?;
            }
            for o in &mut call.options {
                resolve_in_expr(&mut o.value, index)?;
            }
        }
        StmtKind::Print(e)
        | StmtKind::PrintErr(e)
        | StmtKind::Exit(Some(e))
        | StmtKind::Return(Some(e))
        | StmtKind::Wait(Some(e))
        | StmtKind::Sh(e)
        | StmtKind::Cd { path: e }
        | StmtKind::Export { value: Some(e), .. }
        | StmtKind::Source { path: e } => {
            resolve_in_expr(e, index)?;
        }
        StmtKind::If { cond, then_body, elifs, else_body } => {
            resolve_in_expr(cond, index)?;
            for s in then_body {
                resolve_in_stmt(s, index)?;
            }
            for e in elifs {
                resolve_in_expr(&mut e.cond, index)?;
                for s in &mut e.body {
                    resolve_in_stmt(s, index)?;
                }
            }
            if let Some(body) = else_body {
                for s in body {
                    resolve_in_stmt(s, index)?;
                }
            }
        }
        StmtKind::For { iterable, body, .. } => {
            match iterable {
                crate::ast::ForIterable::List(items) => {
                    for i in items { resolve_in_expr(i, index)?; }
                }
                crate::ast::ForIterable::Range(start, end) => {
                    resolve_in_expr(start, index)?;
                    resolve_in_expr(end, index)?;
                }
                crate::ast::ForIterable::Find0(spec) => {
                    if let Some(e) = &mut spec.dir { resolve_in_expr(e, index)?; }
                    if let Some(e) = &mut spec.name { resolve_in_expr(e, index)?; }
                    if let Some(e) = &mut spec.type_filter { resolve_in_expr(e, index)?; }
                    if let Some(e) = &mut spec.maxdepth { resolve_in_expr(e, index)?; }
                }
                _ => {}
            }
            for s in body { resolve_in_stmt(s, index)?; }
        }
        StmtKind::ForMap { body, .. } => {
            for s in body { resolve_in_stmt(s, index)?; }
        }
        StmtKind::TryCatch { try_body, catch_body } => {
            for s in try_body { resolve_in_stmt(s, index)?; }
            for s in catch_body { resolve_in_stmt(s, index)?; }
        }
        StmtKind::Pipe(segments) => {
            for seg in segments {
                match &mut seg.node {
                    crate::ast::PipeSegment::Run(call) | crate::ast::PipeSegment::Sudo(call) => {
                        for a in &mut call.args { resolve_in_expr(a, index)?; }
                        for o in &mut call.options { resolve_in_expr(&mut o.value, index)?; }
                    }
                    crate::ast::PipeSegment::Block(body) | crate::ast::PipeSegment::EachLine(_, body) => {
                        for s in body { resolve_in_stmt(s, index)?; }
                    }
                }
            }
        }
        StmtKind::Exec(args) => {
            for a in args { resolve_in_expr(a, index)?; }
        }
        StmtKind::AndThen { left, right } | StmtKind::OrElse { left, right } => {
            for s in left {
                resolve_in_stmt(s, index)?;
            }
            for s in right {
                resolve_in_stmt(s, index)?;
            }
        }
        StmtKind::WithEnv { bindings, body } => {
            for (_, v) in bindings {
                resolve_in_expr(v, index)?;
            }
            for s in body {
                resolve_in_stmt(s, index)?;
            }
        }
        StmtKind::WithCwd { path, body } => {
            resolve_in_expr(path, index)?;
            for s in body {
                resolve_in_stmt(s, index)?;
            }
        }
        StmtKind::WithLog { path, body, .. } => {
            resolve_in_expr(path, index)?;
            for s in body {
                resolve_in_stmt(s, index)?;
            }
        }
        StmtKind::WithRedirect { stdout, stderr, stdin, body } => {
            if let Some(targets) = stdout {
                for t in targets {
                    if let crate::ast::RedirectOutputTarget::File { path, .. } = &mut t.node {
                        resolve_in_expr(path, index)?;
                    }
                }
            }
            if let Some(targets) = stderr {
                for t in targets {
                    if let crate::ast::RedirectOutputTarget::File { path, .. } = &mut t.node {
                        resolve_in_expr(path, index)?;
                    }
                }
            }
            if let Some(crate::ast::RedirectInputTarget::File { path }) = stdin {
                resolve_in_expr(path, index)?;
            }
            for s in body {
                resolve_in_stmt(s, index)?;
            }
        }
        StmtKind::Subshell { body } | StmtKind::Group { body } => {
            for s in body {
                resolve_in_stmt(s, index)?;
            }
        }
        StmtKind::Spawn { stmt } => {
            resolve_in_stmt(stmt, index)?;
        }
        StmtKind::Case { expr, arms } => {
            resolve_in_expr(expr, index)?;
            for a in arms {
                for s in &mut a.body {
                    resolve_in_stmt(s, index)?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

// SYNC WITH: debug_assert_expr_resolved below.
// When adding new ExprKind variants that contain Expr, update both walkers.
fn resolve_in_expr(expr: &mut Expr, index: &ImportIndex) -> Result<(), Diagnostic> {
    match &mut expr.node {
        ExprKind::QualifiedCall { ns, ns_span, name, name_span, args, resolved_path, resolved_mangled } => {
            let target_path = match index.alias_map.get(ns) {
                Some(p) => p.clone(),
                None => return Err(diag_unknown_alias(ns, *ns_span, index)),
            };
            let funcs = match index.file_functions.get(&target_path) {
                Some(f) => f,
                None => return Err(diag_unknown_func(ns, name, *name_span, index)),
            };
            if !funcs.contains(name) {
                return Err(diag_unknown_func(ns, name, *name_span, index));
            }
            *resolved_path = Some(target_path);
            *resolved_mangled = Some(names::mangle(ns, name));

            for a in args {
                resolve_in_expr(a, index)?;
            }
        }
        ExprKind::QualifiedCommandWord { ns, ns_span, name, name_span, resolved_path, resolved_mangled } => {
            let target_path = match index.alias_map.get(ns) {
                Some(p) => p.clone(),
                None => return Err(diag_unknown_alias(ns, *ns_span, index)),
            };
            let funcs = match index.file_functions.get(&target_path) {
                Some(f) => f,
                None => return Err(diag_unknown_func(ns, name, *name_span, index)),
            };
            if !funcs.contains(name) {
                return Err(diag_unknown_func(ns, name, *name_span, index));
            }
            *resolved_path = Some(target_path);
            *resolved_mangled = Some(names::mangle(ns, name));
        }
        ExprKind::Command(args) => {
            for a in args {
                resolve_in_expr(a, index)?;
            }
        }
        ExprKind::CommandPipe(pipeline) => {
            for block in pipeline {
                for a in block {
                    resolve_in_expr(a, index)?;
                }
            }
        }
        ExprKind::Concat(a, b) | ExprKind::And(a, b) | ExprKind::Or(a, b) | ExprKind::Join { list: a, sep: b } | ExprKind::Index { list: a, index: b } => {
            resolve_in_expr(a, index)?;
            resolve_in_expr(b, index)?;
        }
        ExprKind::Arith { left, right, .. } | ExprKind::Compare { left, right, .. } => {
            resolve_in_expr(left, index)?;
            resolve_in_expr(right, index)?;
        }
        ExprKind::Not(e) | ExprKind::Exists(e) | ExprKind::IsDir(e) | ExprKind::IsFile(e) | ExprKind::IsSymlink(e) | ExprKind::IsExec(e) | ExprKind::IsReadable(e) | ExprKind::IsWritable(e) | ExprKind::IsNonEmpty(e) | ExprKind::BoolStr(e) | ExprKind::Len(e) | ExprKind::Count(e) | ExprKind::Arg(e) | ExprKind::Env(e) | ExprKind::Input(e) | ExprKind::Field { base: e, .. } => {
            resolve_in_expr(e, index)?;
        }
        ExprKind::List(items) => {
            for i in items {
                resolve_in_expr(i, index)?;
            }
        }

        ExprKind::Call { args, options, .. } => {
            for a in args {
                resolve_in_expr(a, index)?;
            }
            for o in options {
                resolve_in_expr(&mut o.value, index)?;
            }
        }
        ExprKind::MapLiteral(entries) => {
            for (_, v) in entries {
                resolve_in_expr(v, index)?;
            }
        }
        ExprKind::Capture { expr, options } => {
            resolve_in_expr(expr, index)?;
            for o in options {
                resolve_in_expr(&mut o.value, index)?;
            }
        }
        ExprKind::Sh { cmd, options } => {
            resolve_in_expr(cmd, index)?;
            for o in options {
                resolve_in_expr(&mut o.value, index)?;
            }
        }
        ExprKind::Confirm { prompt, default } => {
            resolve_in_expr(prompt, index)?;
            if let Some(d) = default {
                resolve_in_expr(d, index)?;
            }
        }
        ExprKind::Sudo { args, options } => {
            for a in args {
                resolve_in_expr(a, index)?;
            }
            for o in options {
                resolve_in_expr(&mut o.value, index)?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(debug_assertions)]
pub fn debug_assert_program_resolved(program: &Program) {
    for func in &program.functions {
        for stmt in &func.body {
            debug_assert_stmt_resolved(stmt);
        }
    }
}

#[cfg(debug_assertions)]
// SYNC WITH: resolve_in_stmt above.
fn debug_assert_stmt_resolved(stmt: &Stmt) {
    match &stmt.node {
        StmtKind::QualifiedCall { resolved_path, resolved_mangled, args, .. } => {
            debug_assert!(resolved_path.is_some());
            debug_assert!(resolved_mangled.is_some());
            for a in args {
                debug_assert_expr_resolved(a);
            }
        }
        StmtKind::Let { value, .. } | StmtKind::Set { value, .. } => {
            debug_assert_expr_resolved(value);
        }
        StmtKind::Run(call) => {
            for a in &call.args {
                debug_assert_expr_resolved(a);
            }
            for o in &call.options {
                debug_assert_expr_resolved(&o.value);
            }
        }
        StmtKind::Print(e)
        | StmtKind::PrintErr(e)
        | StmtKind::Exit(Some(e))
        | StmtKind::Return(Some(e))
        | StmtKind::Wait(Some(e))
        | StmtKind::Sh(e)
        | StmtKind::Cd { path: e }
        | StmtKind::Export { value: Some(e), .. }
        | StmtKind::Source { path: e } => {
            debug_assert_expr_resolved(e);
        }
        StmtKind::If { cond, then_body, elifs, else_body } => {
            debug_assert_expr_resolved(cond);
            for s in then_body {
                debug_assert_stmt_resolved(s);
            }
            for e in elifs {
                debug_assert_expr_resolved(&e.cond);
                for s in &e.body {
                    debug_assert_stmt_resolved(s);
                }
            }
            if let Some(body) = else_body {
                for s in body {
                    debug_assert_stmt_resolved(s);
                }
            }
        }
        StmtKind::For { iterable, body, .. } => {
            match iterable {
                crate::ast::ForIterable::List(items) => {
                    for i in items { debug_assert_expr_resolved(i); }
                }
                crate::ast::ForIterable::Range(start, end) => {
                    debug_assert_expr_resolved(start);
                    debug_assert_expr_resolved(end);
                }
                crate::ast::ForIterable::Find0(spec) => {
                    if let Some(e) = &spec.dir { debug_assert_expr_resolved(e); }
                    if let Some(e) = &spec.name { debug_assert_expr_resolved(e); }
                    if let Some(e) = &spec.type_filter { debug_assert_expr_resolved(e); }
                    if let Some(e) = &spec.maxdepth { debug_assert_expr_resolved(e); }
                }
                _ => {}
            }
            for s in body { debug_assert_stmt_resolved(s); }
        }
        StmtKind::ForMap { body, .. } => {
            for s in body { debug_assert_stmt_resolved(s); }
        }
        StmtKind::TryCatch { try_body, catch_body } => {
            for s in try_body { debug_assert_stmt_resolved(s); }
            for s in catch_body { debug_assert_stmt_resolved(s); }
        }
        StmtKind::Pipe(segments) => {
            for seg in segments {
                match &seg.node {
                    crate::ast::PipeSegment::Run(call) | crate::ast::PipeSegment::Sudo(call) => {
                        for a in &call.args { debug_assert_expr_resolved(a); }
                        for o in &call.options { debug_assert_expr_resolved(&o.value); }
                    }
                    crate::ast::PipeSegment::Block(body) | crate::ast::PipeSegment::EachLine(_, body) => {
                        for s in body { debug_assert_stmt_resolved(s); }
                    }
                }
            }
        }
        StmtKind::Exec(args) => {
            for a in args { debug_assert_expr_resolved(a); }
        }
        StmtKind::AndThen { left, right } | StmtKind::OrElse { left, right } => {
            for s in left {
                debug_assert_stmt_resolved(s);
            }
            for s in right {
                debug_assert_stmt_resolved(s);
            }
        }
        StmtKind::WithEnv { bindings, body } => {
            for (_, v) in bindings {
                debug_assert_expr_resolved(v);
            }
            for s in body {
                debug_assert_stmt_resolved(s);
            }
        }
        StmtKind::WithCwd { path, body } => {
            debug_assert_expr_resolved(path);
            for s in body {
                debug_assert_stmt_resolved(s);
            }
        }
        StmtKind::WithLog { path, body, .. } => {
            debug_assert_expr_resolved(path);
            for s in body {
                debug_assert_stmt_resolved(s);
            }
        }
        StmtKind::WithRedirect { stdout, stderr, stdin, body } => {
            if let Some(targets) = stdout {
                for t in targets {
                    if let crate::ast::RedirectOutputTarget::File { path, .. } = &t.node {
                        debug_assert_expr_resolved(path);
                    }
                }
            }
            if let Some(targets) = stderr {
                for t in targets {
                    if let crate::ast::RedirectOutputTarget::File { path, .. } = &t.node {
                        debug_assert_expr_resolved(path);
                    }
                }
            }
            if let Some(crate::ast::RedirectInputTarget::File { path }) = stdin {
                debug_assert_expr_resolved(path);
            }
            for s in body {
                debug_assert_stmt_resolved(s);
            }
        }
        StmtKind::Subshell { body } | StmtKind::Group { body } => {
            for s in body {
                debug_assert_stmt_resolved(s);
            }
        }
        StmtKind::Spawn { stmt } => {
            debug_assert_stmt_resolved(stmt);
        }
        StmtKind::Case { expr, arms } => {
            debug_assert_expr_resolved(expr);
            for a in arms {
                for s in &a.body {
                    debug_assert_stmt_resolved(s);
                }
            }
        }
        _ => {}
    }
}

#[cfg(debug_assertions)]
// SYNC WITH: resolve_in_expr above.
fn debug_assert_expr_resolved(expr: &Expr) {
    match &expr.node {
        ExprKind::QualifiedCall { resolved_path, resolved_mangled, args, .. } => {
            debug_assert!(resolved_path.is_some());
            debug_assert!(resolved_mangled.is_some());
            for a in args {
                debug_assert_expr_resolved(a);
            }
        }
        ExprKind::QualifiedCommandWord { resolved_path, resolved_mangled, .. } => {
            debug_assert!(resolved_path.is_some());
            debug_assert!(resolved_mangled.is_some());
        }
        ExprKind::Command(args) => {
            for a in args {
                debug_assert_expr_resolved(a);
            }
        }
        ExprKind::CommandPipe(pipeline) => {
            for block in pipeline {
                for a in block {
                    debug_assert_expr_resolved(a);
                }
            }
        }
        ExprKind::Concat(a, b) | ExprKind::And(a, b) | ExprKind::Or(a, b) | ExprKind::Join { list: a, sep: b } | ExprKind::Index { list: a, index: b } => {
            debug_assert_expr_resolved(a);
            debug_assert_expr_resolved(b);
        }
        ExprKind::Arith { left, right, .. } | ExprKind::Compare { left, right, .. } => {
            debug_assert_expr_resolved(left);
            debug_assert_expr_resolved(right);
        }
        ExprKind::Not(e) | ExprKind::Exists(e) | ExprKind::IsDir(e) | ExprKind::IsFile(e) | ExprKind::IsSymlink(e) | ExprKind::IsExec(e) | ExprKind::IsReadable(e) | ExprKind::IsWritable(e) | ExprKind::IsNonEmpty(e) | ExprKind::BoolStr(e) | ExprKind::Len(e) | ExprKind::Count(e) | ExprKind::Arg(e) | ExprKind::Env(e) | ExprKind::Input(e) | ExprKind::Field { base: e, .. } => {
            debug_assert_expr_resolved(e);
        }
        ExprKind::List(items) => {
            for i in items {
                debug_assert_expr_resolved(i);
            }
        }

        ExprKind::Call { args, options, .. } => {
            for a in args {
                debug_assert_expr_resolved(a);
            }
            for o in options {
                debug_assert_expr_resolved(&o.value);
            }
        }
        ExprKind::MapLiteral(entries) => {
            for (_, v) in entries {
                debug_assert_expr_resolved(v);
            }
        }
        ExprKind::Capture { expr, options } => {
            debug_assert_expr_resolved(expr);
            for o in options {
                debug_assert_expr_resolved(&o.value);
            }
        }
        ExprKind::Sh { cmd, options } => {
            debug_assert_expr_resolved(cmd);
            for o in options {
                debug_assert_expr_resolved(&o.value);
            }
        }
        ExprKind::Confirm { prompt, default } => {
            debug_assert_expr_resolved(prompt);
            if let Some(d) = default {
                debug_assert_expr_resolved(d);
            }
        }
        ExprKind::Sudo { args, options } => {
            for a in args {
                debug_assert_expr_resolved(a);
            }
            for o in options {
                debug_assert_expr_resolved(&o.value);
            }
        }
        _ => {}
    }
}
