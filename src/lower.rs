use crate::ast;
use crate::ir;

/// Lower a whole program (AST) into IR
pub fn lower(p: ast::Program) -> Vec<ir::Function> {
    p.functions
        .into_iter()
        .map(lower_function)
        .collect()
}

/// Lower a single function
fn lower_function(f: ast::Function) -> ir::Function {
    let mut commands = Vec::new();

    for stmt in f.body {
        lower_stmt(stmt, &mut commands);
    }

    ir::Function {
        name: f.name,
        commands,
    }
}

/// Lower one AST statement into IR commands
fn lower_stmt(stmt: ast::Stmt, out: &mut Vec<ir::Cmd>) {
    match stmt {
        ast::Stmt::Let { name, value } => {
            out.push(ir::Cmd::Assign(name, lower_expr(value)));
        }

        ast::Stmt::Run(args) => {
            let ir_args = args.into_iter().map(lower_expr).collect();
            out.push(ir::Cmd::Exec(ir_args));
        }

        ast::Stmt::Print(e) => {
            out.push(ir::Cmd::Print(lower_expr(e)));
        }

        ast::Stmt::PrintErr(e) => {
            out.push(ir::Cmd::PrintErr(lower_expr(e)));
        }
        ast::Stmt::If { cond, then_body, else_body } => {
            let mut t_cmds = Vec::new();
            for s in then_body {
                lower_stmt(s, &mut t_cmds);
            }
            
            let mut e_cmds = Vec::new();
            if let Some(body) = else_body {
                for s in body {
                    lower_stmt(s, &mut e_cmds);
                }
            }

            out.push(ir::Cmd::If {
                cond: lower_expr(cond),
                then_body: t_cmds,
                else_body: e_cmds,
            });
        }

    }
}

fn lower_expr(e: ast::Expr) -> ir::Val {
    match e {
        ast::Expr::Literal(s) => ir::Val::Literal(s),
        ast::Expr::Var(s) => ir::Val::Var(s),
        ast::Expr::Concat(l, r) => ir::Val::Concat(Box::new(lower_expr(*l)), Box::new(lower_expr(*r))),
        ast::Expr::Compare { left, op, right } => {
            let op = match op {
                ast::CompareOp::Eq => ir::CompareOp::Eq,
                ast::CompareOp::NotEq => ir::CompareOp::NotEq,
            };
            ir::Val::Compare {
                left: Box::new(lower_expr(*left)),
                op,
                right: Box::new(lower_expr(*right)),
            }
        }
    }
}
