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
        ast::Stmt::Run(args) => {
            out.push(ir::Cmd::Exec(args));
        }

        ast::Stmt::Print(s) => {
            out.push(ir::Cmd::Print(s));
        }

        ast::Stmt::PrintErr(s) => {
            out.push(ir::Cmd::PrintErr(s));
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
