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
            out.push(ir::Cmd::Assign(name, value));
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
        ast::Stmt::If { var, then_body, else_body } => {
            let mut then_cmds = Vec::new();
            for s in then_body {
                lower_stmt(s, &mut then_cmds);
            }

            let mut else_cmds = Vec::new();
            if let Some(body) = else_body {
                for s in body {
                    lower_stmt(s, &mut else_cmds);
                }
            }

            out.push(ir::Cmd::IfNonEmpty {
                var,
                then_body: then_cmds,
                else_body: else_cmds,
            });
        }

    }
}

fn lower_expr(e: ast::Expr) -> ir::Val {
    match e {
        ast::Expr::Literal(s) => ir::Val::Literal(s),
        ast::Expr::Var(s) => ir::Val::Var(s),
    }
}
