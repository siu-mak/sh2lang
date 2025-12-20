use crate::ast;
use crate::ir;

pub fn lower(p: ast::Program) -> Vec<ir::Function> {
    p.functions
        .into_iter()
        .map(|f| {
            let mut commands = Vec::new();

            for stmt in f.body {
                match stmt {
                    ast::Stmt::Run(args) => {
                        commands.push(args);
                    }
                }
            }

            ir::Function {
                name: f.name,
                commands,
            }
        })
        .collect()
}
