use sh2c::ast;
use sh2c::span::Span;

pub fn strip_spans_program(p: &mut ast::Program) {
    p.span = Span::new(0, 0);
    p.source_maps.clear();
    p.entry_file = String::new(); // Clear file path for comparison
    for f in &mut p.functions {
        strip_spans_fn(f);
    }
}

pub fn strip_spans_fn(f: &mut ast::Function) {
    f.span = Span::new(0, 0);
    f.file = String::new(); // Clear file path for comparison
    for s in &mut f.body {
        strip_spans_stmt(s);
    }
}

pub fn strip_spans_stmt(s: &mut ast::Stmt) {
    s.span = Span::new(0, 0);
    match &mut s.node {
        ast::StmtKind::Let { name, value, .. } => {
            name.span = Span::new(0, 0);
            strip_spans_expr(value)
        }
        ast::StmtKind::Run(call) => strip_spans_run_call(call),
        ast::StmtKind::Exec(args) => for a in args { strip_spans_expr(a); },
        ast::StmtKind::Print(e) => strip_spans_expr(e),
        ast::StmtKind::PrintErr(e) => strip_spans_expr(e),
        ast::StmtKind::If { cond, then_body, elifs, else_body } => {
            strip_spans_expr(cond);
            for s in then_body { strip_spans_stmt(s); }
            for e in elifs {
                strip_spans_expr(&mut e.cond);
                for s in &mut e.body { strip_spans_stmt(s); }
            }
            if let Some(body) = else_body {
                for s in body { strip_spans_stmt(s); }
            }
        }
        ast::StmtKind::While { cond, body } => {
            strip_spans_expr(cond);
            for s in body { strip_spans_stmt(s); }
        }
        ast::StmtKind::For { var, iterable, body, .. } => {
            var.span = Span::new(0, 0);
            match iterable {
                ast::ForIterable::List(items) => {
                     for i in items {
                         strip_spans_expr(i);
                     }
                }
                ast::ForIterable::Range(start, end) => {
                    strip_spans_expr(start);
                    strip_spans_expr(end);
                }
                ast::ForIterable::StdinLines => {}
                ast::ForIterable::Find0(spec) => {
                    if let Some(ref mut d) = spec.dir { strip_spans_expr(d); }
                    if let Some(ref mut n) = spec.name { strip_spans_expr(n); }
                    if let Some(ref mut t) = spec.type_filter { strip_spans_expr(t); }
                    if let Some(ref mut m) = spec.maxdepth { strip_spans_expr(m); }
                }
            }
            for s in body {
                strip_spans_stmt(s);
            }
        }
        ast::StmtKind::ForMap { key_var, val_var, body, .. } => {
             key_var.span = Span::new(0, 0);
             val_var.span = Span::new(0, 0);
             for s in body { strip_spans_stmt(s); }
        }
        ast::StmtKind::TryCatch { try_body, catch_body } => {
            for s in try_body { strip_spans_stmt(s); }
            for s in catch_body { strip_spans_stmt(s); }
        }
        ast::StmtKind::Pipe(segments) => {
            for seg in segments {
                seg.span = Span::new(0, 0);
                match &mut seg.node {
                    ast::PipeSegment::Run(call) => strip_spans_run_call(call),
                    ast::PipeSegment::Sudo(call) => strip_spans_run_call(call),
                    ast::PipeSegment::Block(stmts) => for s in stmts { strip_spans_stmt(s); },
                    ast::PipeSegment::EachLine(ident, body) => {
                        ident.span = Span::new(0, 0);
                        for s in body { strip_spans_stmt(s); }
                    }
                }
            }
        }
        ast::StmtKind::Return(Some(e)) => strip_spans_expr(e),
        ast::StmtKind::Exit(Some(e)) => strip_spans_expr(e),
        ast::StmtKind::Cd { path } => strip_spans_expr(path),
        ast::StmtKind::Export { value: Some(v), .. } => strip_spans_expr(v),
        ast::StmtKind::Source { path } => strip_spans_expr(path),
        ast::StmtKind::Call { args, .. } => for a in args { strip_spans_expr(a); },
        ast::StmtKind::AndThen { left, right } => {
            for s in left { strip_spans_stmt(s); }
            for s in right { strip_spans_stmt(s); }
        }
        ast::StmtKind::OrElse { left, right } => {
            for s in left { strip_spans_stmt(s); }
            for s in right { strip_spans_stmt(s); }
        }
        ast::StmtKind::WithEnv { bindings, body } => {
             for (_, v) in bindings { strip_spans_expr(v); }
             for s in body { strip_spans_stmt(s); }
        }
        ast::StmtKind::WithCwd { path, body } => {
             strip_spans_expr(path);
             for s in body { strip_spans_stmt(s); }
        }
        ast::StmtKind::WithLog { path, body, .. } => {
             strip_spans_expr(path);
             for s in body { strip_spans_stmt(s); }
        }
        ast::StmtKind::WithRedirect { stdout, stderr, stdin, body } => {
            // stdout/stderr are now Option<Vec<Spanned<RedirectOutputTarget>>>
            if let Some(targets) = stdout {
                for t in targets {
                    t.node.strip_spans();
                }
            }
            if let Some(targets) = stderr {
                for t in targets {
                    t.node.strip_spans();
                }
            }
            if let Some(t) = stdin { t.strip_spans(); }
            for stmt in body {
                strip_spans_stmt(stmt);
            }
        }
        ast::StmtKind::Subshell { body } => {
             for s in body { strip_spans_stmt(s); }
        }
        ast::StmtKind::Group { body } => {
             for s in body { strip_spans_stmt(s); }
        }
        ast::StmtKind::Spawn { stmt } => strip_spans_stmt(stmt),
        ast::StmtKind::Wait(Some(e)) => strip_spans_expr(e),
        ast::StmtKind::Set { target, value, .. } => {
             target.strip_spans();
             strip_spans_expr(value)
        },
        ast::StmtKind::Case { expr, arms } => {
            strip_spans_expr(expr);
            for arm in arms {
                for s in &mut arm.body { strip_spans_stmt(s); }
            }
        },
        _ => {}
    }
}

pub fn strip_spans_expr(e: &mut ast::Expr) {
    e.span = Span::new(0, 0);
    match &mut e.node {
        ast::ExprKind::Command(args) => for a in args { strip_spans_expr(a); },
        ast::ExprKind::CommandPipe(segs) => for s in segs { for a in s { strip_spans_expr(a); } },
        ast::ExprKind::Concat(l, r) => { strip_spans_expr(l); strip_spans_expr(r); },
        ast::ExprKind::Arith { left, right, .. } => { strip_spans_expr(left); strip_spans_expr(right); },
        ast::ExprKind::Compare { left, right, .. } => { strip_spans_expr(left); strip_spans_expr(right); },
        ast::ExprKind::And(l, r) => { strip_spans_expr(l); strip_spans_expr(r); },
        ast::ExprKind::Or(l, r) => { strip_spans_expr(l); strip_spans_expr(r); },
        ast::ExprKind::Not(e) => strip_spans_expr(e),
        ast::ExprKind::Exists(e) => strip_spans_expr(e),
        ast::ExprKind::IsDir(e) => strip_spans_expr(e),
        ast::ExprKind::IsFile(e) => strip_spans_expr(e),
        ast::ExprKind::IsSymlink(e) => strip_spans_expr(e),
        ast::ExprKind::IsExec(e) => strip_spans_expr(e),
        ast::ExprKind::IsReadable(e) => strip_spans_expr(e),
        ast::ExprKind::IsWritable(e) => strip_spans_expr(e),
        ast::ExprKind::IsNonEmpty(e) => strip_spans_expr(e),
        ast::ExprKind::BoolStr(e) => strip_spans_expr(e),
        ast::ExprKind::Len(e) => strip_spans_expr(e),
        ast::ExprKind::Index { list, index } => { strip_spans_expr(list); strip_spans_expr(index); },
        ast::ExprKind::Field { base, .. } => strip_spans_expr(base),
        ast::ExprKind::Join { list, sep } => { strip_spans_expr(list); strip_spans_expr(sep); },
        ast::ExprKind::Count(e) => strip_spans_expr(e),
        ast::ExprKind::List(items) => for i in items { strip_spans_expr(i); },
        ast::ExprKind::Env(e) => strip_spans_expr(e),
        ast::ExprKind::Input(e) => strip_spans_expr(e),
        ast::ExprKind::Confirm { prompt, default } => {
            strip_spans_expr(prompt);
            if let Some(d) = default { strip_spans_expr(d); }
        }
        ast::ExprKind::Call { args, .. } => for a in args { strip_spans_expr(a); },
        ast::ExprKind::MapLiteral(entries) => for (_, v) in entries { strip_spans_expr(v); },
        _ => {}
    }
}

pub fn strip_spans_run_call(c: &mut ast::RunCall) {
     for a in &mut c.args { strip_spans_expr(a); }
     for o in &mut c.options {
         o.span = Span::new(0, 0);
         strip_spans_expr(&mut o.value);
     }
}
