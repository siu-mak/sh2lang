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

        ast::Stmt::If { var, body } => {
            let mut inner = Vec::new();
            for s in body {
                lower_stmt(s, &mut inner);
            }

            out.push(ir::Cmd::IfNonEmpty {
                var,
                body: inner,
            });
        }
    }
}
