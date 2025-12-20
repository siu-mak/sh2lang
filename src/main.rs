mod lexer;
mod parser;
mod ast;
mod ir;
mod lower;
mod codegen;

use std::fs;

fn main() {
    let src = fs::read_to_string("example.sh2").unwrap();

    let tokens = lexer::lex(&src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let bash = codegen::emit(&ir);

    println!("{}", bash);
}
